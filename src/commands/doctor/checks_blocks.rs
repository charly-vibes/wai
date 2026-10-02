// Block-related doctor checks: agent instructions (managed blocks) and
// managed-block staleness.

use genesis::doctor::CheckStatus;
use genesis::managed_block::content_sha8;

use super::WaiCheckEntry;
use crate::managed_block::InstalledPipeline;
use miette::IntoDiagnostic;
use std::path::Path;

/// Fix closure type carried by [`WaiCheckEntry::fix_fn`].
type FixFn = Box<dyn FnOnce(&Path) -> miette::Result<()>>;

/// Workspace generator state shared by the staleness checks: detected plugin
/// names, installed skill names, and installed pipelines.
struct WorkspaceState {
    plugin_names: Vec<String>,
    skill_names: Vec<String>,
    pipelines: Vec<InstalledPipeline>,
}

impl WorkspaceState {
    fn detect(project_root: &Path) -> Self {
        let plugins = crate::plugin::detect_plugins(project_root);
        let plugin_names: Vec<String> = plugins
            .iter()
            .filter(|p| p.detected)
            .map(|p| p.def.name.clone())
            .collect();
        let skill_names = crate::workspace::detect_installed_skill_names(project_root);
        let pipelines = crate::workspace::detect_installed_pipelines(project_root);
        Self {
            plugin_names,
            skill_names,
            pipelines,
        }
    }

    fn plugin_refs(&self) -> Vec<&str> {
        self.plugin_names.iter().map(|s| s.as_str()).collect()
    }

    fn skill_refs(&self) -> Vec<&str> {
        self.skill_names.iter().map(|s| s.as_str()).collect()
    }
}

/// Build the fix closure that re-injects the managed block into `target`
/// (relative to the workspace root), regenerating plugin/skill/pipeline state
/// at fix time.
fn inject_block_fix(target: &'static str, err_prefix: &'static str) -> FixFn {
    inject_block_fix_owned(target.to_string(), err_prefix)
}

fn inject_block_fix_owned(target: String, err_prefix: &'static str) -> FixFn {
    Box::new(move |root| {
        let state = WorkspaceState::detect(root);
        crate::managed_block::inject_managed_block(
            &root.join(&target),
            &state.plugin_refs(),
            &state.skill_refs(),
            &state.pipelines,
        )
        .map(|_| ())
        .map_err(|e| miette::miette!("{}: {}", err_prefix, e))
    })
}

/// Returns true if the WAI managed block in `path` already mentions the ro5 skill.
/// Used to detect a stale block when the ro5 skill was installed after the last `wai init`.
fn managed_block_mentions_ro5(path: &std::path::Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let wai_start = "<!-- WAI:START -->";
    let wai_end = "<!-- WAI:END -->";
    if let (Some(start), Some(end)) = (content.find(wai_start), content.find(wai_end)) {
        content[start..end].contains("/ro5")
    } else {
        false
    }
}

pub(super) fn check_agent_instructions(project_root: &Path) -> Vec<WaiCheckEntry> {
    let skill_names = crate::workspace::detect_installed_skill_names(project_root);
    let has_ro5_skill = skill_names
        .iter()
        .any(|s| s == "ro5" || s == "rule-of-5" || s == "rule-of-5-universal");

    [("AGENTS.md", "LLMs"), ("CLAUDE.md", "Claude Code")]
        .iter()
        .map(|(filename, reader)| check_agent_file(project_root, filename, reader, has_ro5_skill))
        .collect()
}

/// Check one agent-instructions file (AGENTS.md / CLAUDE.md): present, block
/// present, and ro5-skill freshness.
fn check_agent_file(
    project_root: &Path,
    filename: &str,
    reader: &str,
    has_ro5_skill: bool,
) -> WaiCheckEntry {
    let path = project_root.join(filename);
    let name = format!("Agent instructions: {filename}");
    if !path.exists() {
        return WaiCheckEntry {
            name,
            status: CheckStatus::Warn,
            message: format!("{filename} not found — {reader} won't know to use wai"),
            fix: Some(format!(
                "Run: wai init (to create {filename} with wai instructions)"
            )),
            fix_fn: Some(inject_block_fix_named(filename)),
        };
    }
    if !crate::managed_block::has_managed_block(&path) {
        return WaiCheckEntry {
            name,
            status: CheckStatus::Warn,
            message: "Exists but missing wai managed block".to_string(),
            fix: Some(format!(
                "Run: wai init (to inject wai instructions into {filename})"
            )),
            fix_fn: Some(inject_block_fix_named(filename)),
        };
    }
    if has_ro5_skill && !managed_block_mentions_ro5(&path) {
        return WaiCheckEntry {
            name,
            status: CheckStatus::Warn,
            message: "Managed block is stale: ro5 skill installed but not reflected".to_string(),
            fix: Some("Run: wai init (to regenerate managed block with ro5 reminders)".to_string()),
            fix_fn: Some(inject_block_fix_named(filename)),
        };
    }
    WaiCheckEntry {
        name,
        status: CheckStatus::Pass,
        message: "Contains wai managed block".to_string(),
        fix: None,
        fix_fn: None,
    }
}

/// Fix closure keyed by a runtime filename (the static-target helper cannot
/// express the `Agent instructions` entries, whose fix text is dynamic).
fn inject_block_fix_named(filename: &str) -> FixFn {
    let filename = filename.to_string();
    Box::new(move |root| {
        let state = WorkspaceState::detect(root);
        crate::managed_block::inject_managed_block(
            &root.join(&filename),
            &state.plugin_refs(),
            &state.skill_refs(),
            &state.pipelines,
        )
        .into_diagnostic()?;
        Ok(())
    })
}

/// Check managed block staleness by comparing generated vs actual content.
///
/// Provenance-aware (genesis 0.11 add-artifact-provenance): the footer is
/// stripped before the full-text compare, so footer version-text changes
/// never flag staleness; the footer's recorded sha is then verified against
/// the footer-free body to catch post-init edits.
pub(super) fn check_managed_block_staleness(project_root: &Path) -> Vec<WaiCheckEntry> {
    use crate::managed_block::{read_managed_block_parts, wai_block_content, wai_detailed_content};

    let root = project_root.to_path_buf();
    let state = WorkspaceState::detect(project_root);

    let expected = wai_block_content(
        &root,
        &state.plugin_refs(),
        &state.skill_refs(),
        &state.pipelines,
    );

    let mut results: Vec<WaiCheckEntry> = ["CLAUDE.md", "AGENTS.md"]
        .iter()
        .filter_map(|filename| {
            let read = read_managed_block_parts(&root.join(filename))?;
            let stale = read.content != expected;
            let drifted = !stale
                && read
                    .footer_sha
                    .as_deref()
                    .is_some_and(|sha| sha != content_sha8(&read.body));
            (stale || drifted).then(|| root_block_stale_entry(filename, drifted))
        })
        .collect();

    let expected_detailed = wai_detailed_content(
        &root,
        &state.plugin_refs(),
        &state.skill_refs(),
        &state.pipelines,
    );
    results.extend(detailed_file_staleness(&root, &expected_detailed));

    results
}

/// Staleness entry for an outdated CLAUDE.md / AGENTS.md managed block.
fn root_block_stale_entry(filename: &str, drifted: bool) -> WaiCheckEntry {
    let target = filename.to_string();
    let message = if drifted {
        format!(
            "{filename} managed block provenance drift — footer hash does not match block content (edited after last init?) — run 'wai init' to refresh"
        )
    } else {
        format!("{filename} managed block outdated — run 'wai init' to refresh")
    };
    WaiCheckEntry {
        name: format!("Managed block staleness: {filename}"),
        status: CheckStatus::Warn,
        message,
        fix: Some("Run: wai init".to_string()),
        fix_fn: Some(inject_block_fix_owned(
            target,
            "Failed to inject managed block",
        )),
    }
}

/// Staleness entry for the `.wai/AGENTS.md` detailed reference file.
fn detailed_file_staleness(root: &Path, expected_detailed: &str) -> Option<WaiCheckEntry> {
    let name = "Managed block staleness: .wai/AGENTS.md".to_string();
    let detailed_path = root.join(".wai").join("AGENTS.md");
    if !detailed_path.exists() {
        return Some(WaiCheckEntry {
            name,
            status: CheckStatus::Warn,
            message: ".wai/AGENTS.md missing — run 'wai init' to create it".to_string(),
            fix: Some("Run: wai init".to_string()),
            fix_fn: Some(inject_block_fix(
                ".wai/AGENTS.md",
                "Failed to create .wai/AGENTS.md",
            )),
        });
    }
    let actual = std::fs::read_to_string(&detailed_path).ok()?;
    (actual != expected_detailed).then(|| WaiCheckEntry {
        name,
        status: CheckStatus::Warn,
        message: ".wai/AGENTS.md outdated — run 'wai init' to refresh".to_string(),
        fix: Some("Run: wai init".to_string()),
        fix_fn: Some(inject_block_fix(
            ".wai/AGENTS.md",
            "Failed to fix .wai/AGENTS.md",
        )),
    })
}
