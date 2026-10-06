use genesis::managed_block::{BlockDef, BlockInjector, BlockRegistry, InjectResult};
use std::path::Path;

pub(crate) const WAI_START: &str = "<!-- WAI:START -->";
pub(crate) const WAI_END: &str = "<!-- WAI:END -->";

pub(crate) fn ubiquitous_language_note(repo_root: &Path) -> Option<&'static str> {
    let index = repo_root.join(".wai/resources/ubiquitous-language/README.md");
    index.exists().then_some(
        "## Ubiquitous Language\n\
         \n\
         If `.wai/resources/ubiquitous-language/README.md` exists, read it first as the\n\
         navigation index, then open only the bounded-context files relevant to the task.\n\
         Avoid loading every terminology file unless the work truly spans multiple contexts.\n",
    )
}

/// Info about an installed pipeline with metadata, for managed block generation.
#[derive(Debug, Clone)]
pub struct InstalledPipeline {
    pub name: String,
    pub description: String,
    pub when: String,
    pub step_count: usize,
}

/// Generate the **slim** managed block for CLAUDE.md / AGENTS.md.
///
/// This is Layer 1 of progressive disclosure: orient the agent, surface
/// pipelines, and point to `.wai/AGENTS.md` for the full reference.
///
/// Footer-free generator content: the on-disk block additionally carries a
/// provenance footer inside the markers (genesis `with_provenance`), which
/// the doctor staleness check strips before comparing.
pub fn wai_block_content(
    repo_root: &Path,
    detected_plugins: &[&str],
    installed_skills: &[&str],
    installed_pipelines: &[InstalledPipeline],
) -> String {
    // Footer-free generator form: on-disk blocks written by inject_managed_block
    // carry a provenance footer inside the markers; read_managed_block strips it
    // so the doctor staleness check compares like-for-like.
    let mut block = String::from(WAI_START);
    block.push_str(&wai_block_inner(
        repo_root,
        detected_plugins,
        installed_skills,
        installed_pipelines,
    ));
    block.push_str(WAI_END);
    block
}

/// Generate the inner content of the slim managed block (without WAI:START/WAI:END markers).
///
/// Used by the genesis BlockInjector which adds its own markers.
pub fn wai_block_inner(
    repo_root: &Path,
    detected_plugins: &[&str],
    installed_skills: &[&str],
    installed_pipelines: &[InstalledPipeline],
) -> String {
    let has_beads = detected_plugins.contains(&"beads");
    let has_openspec = detected_plugins.contains(&"openspec");
    let has_ro5 = installed_skills
        .iter()
        .any(|s| *s == "ro5" || *s == "rule-of-5" || *s == "rule-of-5-universal");

    let mut block = String::new();
    block.push_str(&slim_intro());
    block.push_str(&slim_detected_tools(has_beads, has_openspec));
    if has_ro5 {
        block.push_str(&slim_ro5_reminder());
    }
    block.push_str(&slim_quick_start(has_beads));
    block.push_str(&slim_pipelines_section(installed_pipelines));
    if let Some(note) = ubiquitous_language_note(repo_root) {
        block.push('\n');
        block.push_str(note);
    }
    block.push_str(&slim_autonomous_policy());
    block.push_str(&slim_detailed_pointer());
    block
}

/// Slim block header: what wai is and why to run `wai status` first.
fn slim_intro() -> String {
    "# Workflow Tools\n\
     \n\
     This project uses **wai** to track the *why* behind decisions — research,\n\
     reasoning, and design choices that shaped the code. Run `wai status` first\n\
     to orient yourself.\n"
        .to_string()
}

/// Detected companion workflow tools plus the TDD/Tidy-First and
/// search-before-research reminders (only when companions are present).
fn slim_detected_tools(has_beads: bool, has_openspec: bool) -> String {
    let mut section = String::new();
    if has_beads || has_openspec {
        section.push_str(
            "\n\
             Detected workflow tools:\n\
             - **wai** — research, reasoning, and design decisions\n",
        );
        if has_beads {
            section.push_str(
                "- **beads** — issue tracking (tasks, bugs, dependencies). \
                 CLI command: **`bd`** (not `beads`)\n",
            );
        }
        if has_openspec {
            section.push_str(
                "- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)\n",
            );
        }
        section.push_str(
            "\n\
             > **CRITICAL**: Apply TDD and Tidy First throughout — not just when writing code:\n\
             > - **Planning/task creation**: each ticket should map to a red→green→refactor cycle; \
             refactoring tasks must be separate tickets from feature tasks.\n\
             > - **Design**: define the test shape (inputs/outputs) before designing the implementation.\n\
             > - **Implementation**: write the failing test first, then make it pass, then tidy in a separate commit.\n\
             \n\
             > **When beginning research or creating a ticket**: run `wai search \"<topic>\"` \
             to check for existing patterns before writing new content.\n",
        );
    }
    section
}

/// Ro5 skill reminder, present only when the skill is installed.
fn slim_ro5_reminder() -> String {
    "> **Ro5**: The Rule of 5 skill is installed. Run `/ro5` after key phase transitions \
     — implement, research, design — for iterative quality review.\n"
        .to_string()
}

/// Quick start — just the essentials.
fn slim_quick_start(has_beads: bool) -> String {
    let mut section = String::from("\n## Quick Start\n\n");
    section.push_str("1. `wai sync` — ensure agent tools are projected\n");
    section.push_str("2. `wai status` — see active projects, phase, and suggestions\n");
    if has_beads {
        section.push_str("3. `bd ready` — find available work items\n");
    }
    section.push_str(
        "\n\
         When context reaches ~40%: stop and tell the user — responses degrade past\n\
         this point. Recommend `wai close` then `/clear` to resume cleanly.\n\
         Do NOT skip `wai close` — it enables resume detection.\n",
    );
    section
}

/// Available Pipelines — discovery-critical, stays in the slim block.
fn slim_pipelines_section(installed_pipelines: &[InstalledPipeline]) -> String {
    let mut section = String::new();
    if installed_pipelines.is_empty() {
        return section;
    }
    section.push_str(
        "\n\
         ## Available Pipelines\n\
         \n\
         | Pipeline | When to Use | Start |\n\
         |----------|-------------|-------|\n",
    );
    for p in installed_pipelines {
        section.push_str(&format!(
            "| {} | {} | `wai pipeline start {} --topic=<topic>` |\n",
            p.name, p.when, p.name,
        ));
    }
    section.push_str(
        "\n> Pipeline steps may have gates that enforce artifact creation, review \
         coverage, and oracle checks before advancement. \
         Run `wai pipeline gates <name>` for details.\
         \n> **Before any edit**: run `wai pipeline current --json`; if a run is \
         active for this project, resume it (`wai pipeline next`) rather than \
         starting ad-hoc work.\n",
    );
    section
}

/// Autonomous Work Policy — always present in slim block.
fn slim_autonomous_policy() -> String {
    "\n\
     ## Autonomous Work Policy\n\
     \n\
     Proceed without routine confirmation when the next step is clear.\n\
     Do not ask to continue, fix, or commit — just do it.\n\
     \n\
     **Stop and ask** only when:\n\
     - Conflicting requirements or ambiguous intent\n\
     - Destructive actions (data loss, force-push, drop table)\n\
     - Credentials, secrets, or external services not yet authorized\n\
     - Unresolved test failures after two attempts\n\
     - Push, deploy, or release — always get explicit authorization\n\
     - Context approaching 40% — recommend `wai close` then `/clear`\n"
        .to_string()
}

/// Pointer to the detailed instructions in `.wai/AGENTS.md`.
fn slim_detailed_pointer() -> String {
    "\n\
     ## Detailed Instructions\n\
     \n\
     Full workflow reference — session lifecycle, capturing work, command cheat\n\
     sheets, cross-tool sync, and PARA structure — lives in **`.wai/AGENTS.md`**.\n\
     Read it at the start of your first session or when you need detailed guidance.\n\
     \n\
     Keep this managed block so `wai init` can refresh the instructions.\n\
     \n"
    .to_string()
}

// Re-exports: the detailed reference and REFLECT blocks were split into their
// own modules; keep the historical `managed_block::` import paths working.
pub use crate::detailed_agents::{
    DetailedFileResult, wai_detailed_content, write_detailed_agents_file,
};
pub use crate::reflect_block::{
    REFLECT_REF_END, REFLECT_REF_START, has_reflect_block, read_reflect_block,
    wai_reflect_ref_content,
};

pub fn inject_managed_block(
    path: &Path,
    detected_plugins: &[&str],
    installed_skills: &[&str],
    installed_pipelines: &[InstalledPipeline],
) -> Result<InjectResult, std::io::Error> {
    let repo_root = path.parent().unwrap_or(Path::new("."));
    let wai_content = wai_block_inner(
        repo_root,
        detected_plugins,
        installed_skills,
        installed_pipelines,
    );
    let ref_inner = format!("\n{}\n", wai_reflect_ref_content());
    // Full REF block with markers, used for direct file append
    let ref_full = format!(
        "\n{}\n{}{}\n",
        REFLECT_REF_START,
        wai_reflect_ref_content(),
        REFLECT_REF_END
    );

    let mut reg = BlockRegistry::new();
    reg.register(BlockDef::new("WAI"));
    reg.register(BlockDef::with_markers(
        "WAI:REFLECT:REF",
        REFLECT_REF_START,
        REFLECT_REF_END,
    ));
    // Opt-in provenance footer (genesis 0.11 add-artifact-provenance): the
    // footer records generator/version/source plus a content hash covering
    // only the block content, so version bumps never move the hash and
    // doctor can detect post-init edits via the footer-sha fast path.
    let injector = BlockInjector::new(reg).with_provenance("wai");

    let wai_result = injector.inject(path, "WAI", &wai_content)?;

    // For the REFLECT:REF block, handle ordering:
    // - If the file was just created, append REF block after WAI block
    // - If the WAI block was prepended to an existing file, append REF block after WAI block
    // - Otherwise, use BlockInjector (updates in place)
    let ref_result = if wai_result == InjectResult::Created || wai_result == InjectResult::Prepended
    {
        let mut file_content = std::fs::read_to_string(path)?;
        file_content.push_str(&ref_full);
        std::fs::write(path, &file_content)?;
        InjectResult::Created
    } else {
        injector.inject(path, "WAI:REFLECT:REF", &ref_inner)?
    };

    // Return the most significant result
    Ok(match (wai_result, ref_result) {
        (InjectResult::Created, _) | (_, InjectResult::Created) => InjectResult::Created,
        (InjectResult::Prepended, _) | (_, InjectResult::Prepended) => InjectResult::Prepended,
        _ => InjectResult::Updated,
    })
}

/// Extract the actual WAI block content (between WAI:START and WAI:END, inclusive).
/// Returns `None` if the file does not exist or has no block.
///
/// A trailing provenance footer (genesis 0.11 `add-artifact-provenance`) is
/// stripped so the result compares byte-identically to the footer-free
/// generator content ([`wai_block_content`]).
pub fn read_managed_block(path: &Path) -> Option<String> {
    read_managed_block_parts(path).map(|parts| parts.content)
}

/// Provenance-aware read of the WAI managed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedBlockRead {
    /// Footer-free block content (markers inclusive).
    pub content: String,
    /// Footer-free inner body (between the markers).
    pub body: String,
    /// Recorded footer `sha=` value when a provenance footer is present.
    pub footer_sha: Option<String>,
}

/// Read the WAI managed block, splitting the provenance footer off the
/// inner content. Mirrors genesis 0.11 `split_provenance_footer` semantics:
/// the footer is recognized as the last line inside the markers matching
/// `<!-- provenance: ... -->`, and the separating newline is dropped so the
/// body hashes byte-identically to the generator's content.
pub fn read_managed_block_parts(path: &Path) -> Option<ManagedBlockRead> {
    let mut reg = BlockRegistry::new();
    reg.register(BlockDef::new("WAI"));
    let injector = BlockInjector::new(reg);
    let block = injector.read_block(path, "WAI")?;
    let start = block.find(WAI_START)? + WAI_START.len();
    let end = block.rfind(WAI_END)?;
    if start > end {
        return None;
    }
    let inner = &block[start..end];
    let (body, footer_sha) = split_provenance_footer(inner);
    let mut content = String::with_capacity(block.len());
    content.push_str(&block[..start]);
    content.push_str(body);
    content.push_str(&block[end..]);
    Some(ManagedBlockRead {
        content,
        body: body.to_string(),
        footer_sha: footer_sha.map(str::to_string),
    })
}

/// Split a trailing provenance footer off managed-block inner content.
///
/// Returns the footer-free body and, when the last line is a provenance
/// footer, its `sha=` value. Matches the genesis inject format
/// `{content}\n{footer}\n` — the separating newline is dropped too, so the
/// body hashes byte-identically to the generator's content.
pub fn split_provenance_footer(inner: &str) -> (&str, Option<&str>) {
    let trimmed = inner.trim_end_matches('\n');
    let last_line = match trimmed.rfind('\n') {
        Some(idx) => &trimmed[idx + 1..],
        None if !trimmed.is_empty() => trimmed,
        None => return (inner, None),
    };
    match parse_footer_sha(last_line) {
        Some(sha) => {
            let cut = trimmed.len() - last_line.len();
            (&trimmed[..cut.saturating_sub(1)], Some(sha))
        }
        None => (inner, None),
    }
}

/// Extract `sha=<value>` from a provenance footer line, if it is one.
fn parse_footer_sha(line: &str) -> Option<&str> {
    let rest = line
        .strip_prefix("<!-- provenance: ")?
        .strip_suffix(" -->")?;
    let idx = rest.find("sha=")?;
    let sha = &rest[idx + 4..];
    Some(sha.split_whitespace().next().unwrap_or(sha))
}

pub fn has_managed_block(path: &Path) -> bool {
    let mut reg = BlockRegistry::new();
    reg.register(BlockDef::new("WAI"));
    let injector = BlockInjector::new(reg);
    injector.has_block(path, "WAI")
}

/// Describe the result of a managed block injection for user-facing messages.
pub fn describe_inject_result(result: &InjectResult, filename: &str) -> String {
    match result {
        InjectResult::Created => format!("Created {} with wai instructions", filename),
        InjectResult::Prepended => {
            format!("Added wai instructions to existing {}", filename)
        }
        InjectResult::Updated => format!("Updated wai instructions in {}", filename),
    }
}

// ── REFLECT:REF block ────────────────────────────────────────────────────────

#[cfg(test)]
mod wai_block_tests {
    use super::*;

    // ── Pipeline run adoption (wai-vx02.1) ────────────────────────────────

    #[test]
    fn slim_block_instructs_run_adoption_when_pipelines_installed() {
        let pipes = [InstalledPipeline {
            name: "research-flow".into(),
            description: "d".into(),
            when: "research".into(),
            step_count: 2,
        }];
        let output = wai_block_content(Path::new("."), &[], &[], &pipes);
        assert!(
            output.contains("wai pipeline current --json"),
            "expected run-adoption instruction mentioning `wai pipeline current --json` in slim block"
        );
        assert!(
            output.contains("wai pipeline next"),
            "expected resume command in slim block"
        );
    }

    #[test]
    fn slim_block_has_no_adoption_instruction_without_pipelines() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            !output.contains("wai pipeline current --json"),
            "adoption instruction is noise when no pipelines are installed"
        );
    }

    // ── Slim block (Layer 1: CLAUDE.md / AGENTS.md) ──────────────────────

    #[test]
    fn slim_block_contains_wai_sync() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            output.contains("wai sync"),
            "expected 'wai sync' in slim block"
        );
    }

    #[test]
    fn slim_block_contains_wai_status() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            output.contains("wai status"),
            "expected 'wai status' in slim block"
        );
    }

    #[test]
    fn slim_block_points_to_detailed_instructions() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            output.contains(".wai/AGENTS.md"),
            "expected pointer to .wai/AGENTS.md in slim block"
        );
    }

    #[test]
    fn slim_block_does_not_contain_quick_reference() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            !output.contains("## Quick Reference"),
            "slim block should not contain full Quick Reference"
        );
    }

    #[test]
    fn slim_block_does_not_contain_capturing_work() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            !output.contains("## Capturing Work"),
            "slim block should not contain Capturing Work section"
        );
    }

    #[test]
    fn slim_block_does_not_contain_ending_session() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            !output.contains("## Ending a Session"),
            "slim block should not contain Ending a Session section"
        );
    }

    #[test]
    fn tdd_disclaimer_present_with_companion_tools() {
        let output = wai_block_content(Path::new("."), &["beads", "openspec"], &[], &[]);
        assert!(
            output.contains("CRITICAL"),
            "expected CRITICAL disclaimer in slim block with companion tools"
        );
        assert!(output.contains("TDD"), "expected 'TDD' in slim block");
        assert!(
            output.contains("Tidy First"),
            "expected 'Tidy First' in slim block"
        );
    }

    #[test]
    fn tdd_disclaimer_present_with_beads_only() {
        let output = wai_block_content(Path::new("."), &["beads"], &[], &[]);
        assert!(
            output.contains("CRITICAL"),
            "expected CRITICAL disclaimer with beads"
        );
    }

    #[test]
    fn tdd_disclaimer_absent_without_companion_tools() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            !output.contains("Tidy First"),
            "unexpected 'Tidy First' without companion tools"
        );
    }

    #[test]
    fn ro5_reminder_present_when_skill_installed() {
        for name in &["ro5", "rule-of-5", "rule-of-5-universal"] {
            let output = wai_block_content(Path::new("."), &[], &[name], &[]);
            assert!(
                output.contains("/ro5"),
                "expected '/ro5' when skill '{name}' installed"
            );
        }
    }

    #[test]
    fn ro5_reminder_absent_without_skill() {
        let output = wai_block_content(Path::new("."), &["beads", "openspec"], &[], &[]);
        assert!(!output.contains("/ro5"), "unexpected '/ro5' without skill");
    }

    const SEARCH_INSTRUCTION: &str = "before writing new content";

    #[test]
    fn search_before_research_present_with_companions() {
        for plugins in [
            &["beads"][..],
            &["openspec"][..],
            &["beads", "openspec"][..],
        ] {
            let output = wai_block_content(Path::new("."), plugins, &[], &[]);
            assert!(
                output.contains(SEARCH_INSTRUCTION),
                "expected search instruction with plugins {:?}",
                plugins
            );
        }
    }

    #[test]
    fn search_before_research_absent_without_companions() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            !output.contains(SEARCH_INSTRUCTION),
            "unexpected search instruction without companion tools"
        );
    }

    #[test]
    fn search_before_research_after_tdd_disclaimer() {
        let output = wai_block_content(Path::new("."), &["beads"], &[], &[]);
        let tdd_pos = output.find("CRITICAL").expect("CRITICAL not found");
        let search_pos = output
            .find(SEARCH_INSTRUCTION)
            .expect("search instruction not found");
        assert!(search_pos > tdd_pos);
    }

    #[test]
    fn context_pressure_tells_user() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(output.contains("stop and tell the user"));
        assert!(output.contains("responses degrade"));
    }

    #[test]
    fn slim_block_includes_ubiquitous_language_note_when_index_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let note_root = dir.path().join(".wai/resources/ubiquitous-language");
        std::fs::create_dir_all(&note_root).unwrap();
        std::fs::write(note_root.join("README.md"), "# Index\n").unwrap();

        let output = wai_block_content(dir.path(), &[], &[], &[]);
        assert!(output.contains("## Ubiquitous Language"));
        assert!(output.contains("read it first as the"));
        assert!(output.contains("Avoid loading every terminology file"));
    }

    #[test]
    fn slim_block_omits_ubiquitous_language_note_without_index() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output = wai_block_content(dir.path(), &[], &[], &[]);
        assert!(!output.contains("## Ubiquitous Language"));
    }

    #[test]
    fn slim_block_omits_ubiquitous_language_note_for_partial_tree_without_index() {
        let dir = tempfile::tempdir().expect("tempdir");
        let contexts = dir
            .path()
            .join(".wai/resources/ubiquitous-language/contexts");
        std::fs::create_dir_all(&contexts).unwrap();
        std::fs::write(contexts.join("billing.md"), "# Billing\n").unwrap();

        let output = wai_block_content(dir.path(), &[], &[], &[]);
        assert!(!output.contains("## Ubiquitous Language"));
    }

    // Pipeline section (stays in slim block — discovery-critical)

    #[test]
    fn pipeline_section_present_when_pipelines_installed() {
        let pipelines = vec![InstalledPipeline {
            name: "scientific-research".to_string(),
            description: "AI-assisted research".to_string(),
            when: "Frontier-level research requiring systematic validation".to_string(),
            step_count: 8,
        }];
        let output = wai_block_content(Path::new("."), &[], &[], &pipelines);
        assert!(output.contains("## Available Pipelines"));
        assert!(output.contains("scientific-research"));
        assert!(output.contains("Frontier-level research"));
        assert!(output.contains("wai pipeline start scientific-research --topic=<topic>"));
    }

    #[test]
    fn pipeline_section_absent_when_no_pipelines() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(!output.contains("Available Pipelines"));
    }

    #[test]
    fn pipeline_section_includes_gate_note() {
        let pipelines = vec![InstalledPipeline {
            name: "test".to_string(),
            description: "Test pipeline".to_string(),
            when: "Testing".to_string(),
            step_count: 2,
        }];
        let output = wai_block_content(Path::new("."), &[], &[], &pipelines);
        assert!(output.contains("gates"));
    }

    #[test]
    fn pipeline_section_lists_multiple_pipelines() {
        let pipelines = vec![
            InstalledPipeline {
                name: "alpha".to_string(),
                description: "Alpha workflow".to_string(),
                when: "When alpha".to_string(),
                step_count: 3,
            },
            InstalledPipeline {
                name: "beta".to_string(),
                description: "Beta workflow".to_string(),
                when: "When beta".to_string(),
                step_count: 5,
            },
        ];
        let output = wai_block_content(Path::new("."), &[], &[], &pipelines);
        assert!(output.contains("alpha"));
        assert!(output.contains("beta"));
    }

    // ── Autonomous Work Policy (slim block) ────────────────────────────

    #[test]
    fn slim_block_contains_autonomous_work_policy_section() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            output.contains("## Autonomous Work Policy"),
            "expected Autonomous Work Policy section in slim block"
        );
    }

    #[test]
    fn slim_block_policy_says_no_routine_confirmations() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        assert!(
            output.contains("routine confirmation"),
            "policy should mention not asking routine confirmations"
        );
    }

    #[test]
    fn slim_block_policy_lists_stop_conditions() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        for condition in [
            "Conflicting requirements",
            "Destructive action",
            "Credentials",
            "Unresolved test failure",
            "push",
            "context",
        ] {
            assert!(
                output.contains(condition),
                "stop conditions should mention '{condition}'"
            );
        }
    }

    #[test]
    fn slim_block_policy_before_detailed_instructions() {
        let output = wai_block_content(Path::new("."), &[], &[], &[]);
        let policy_pos = output
            .find("Autonomous Work Policy")
            .expect("Autonomous Work Policy not found");
        let detailed_pos = output
            .find("Detailed Instructions")
            .expect("Detailed Instructions not found");
        assert!(
            policy_pos < detailed_pos,
            "Autonomous Work Policy should appear before Detailed Instructions"
        );
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    use genesis::managed_block::content_sha8;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    /// Extract the text between the WAI markers of a file.
    fn wai_inner(path: &Path) -> String {
        let content = std::fs::read_to_string(path).unwrap();
        let start = content.find(WAI_START).unwrap() + WAI_START.len();
        let end = content.find(WAI_END).unwrap();
        content[start..end].to_string()
    }

    #[test]
    fn inject_writes_provenance_footer_inside_markers() {
        let dir = tmp();
        let path = dir.path().join("AGENTS.md");
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let inner = wai_inner(&path);
        let footer_line = inner
            .lines()
            .find(|l| l.contains("<!-- provenance: "))
            .expect("provenance footer present inside WAI markers");
        assert!(footer_line.starts_with("<!-- provenance: "));
        assert!(footer_line.ends_with(" -->"));
        assert!(footer_line.contains("generator=wai"));
        assert!(footer_line.contains("source=WAI"));
        assert!(footer_line.contains("sha="));
    }

    #[test]
    fn footer_sha_covers_content_excluding_footer_line() {
        let dir = tmp();
        let path = dir.path().join("AGENTS.md");
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let inner = wai_inner(&path);
        let (body, sha) = split_provenance_footer(&inner);
        let recorded = sha.expect("footer sha present");
        assert_eq!(recorded, content_sha8(body));
        // Body is byte-identical to the generator content.
        assert_eq!(body, wai_block_inner(path.parent().unwrap(), &[], &[], &[]));
    }

    #[test]
    fn read_managed_block_strips_provenance_footer() {
        let dir = tmp();
        let path = dir.path().join("AGENTS.md");
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let actual = read_managed_block(&path).unwrap();
        let expected = wai_block_content(path.parent().unwrap(), &[], &[], &[]);
        assert_eq!(
            actual, expected,
            "footer-stripped read must equal footer-free generator content"
        );
    }

    #[test]
    fn split_provenance_footer_leaves_footer_less_content_untouched() {
        let inner = "\n# Content\n";
        let (body, sha) = split_provenance_footer(inner);
        assert_eq!(body, inner);
        assert!(sha.is_none());
    }

    #[test]
    fn footer_survives_reinject_with_updated_hash() {
        let dir = tmp();
        let path = dir.path().join("AGENTS.md");
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        inject_managed_block(&path, &[], &["ro5"], &[]).unwrap();
        let inner = wai_inner(&path);
        assert_eq!(inner.matches("<!-- provenance: ").count(), 1);
        let (_, sha) = split_provenance_footer(&inner);
        let expected_sha =
            content_sha8(&wai_block_inner(path.parent().unwrap(), &[], &["ro5"], &[]));
        assert_eq!(sha.unwrap(), expected_sha);
    }
}
