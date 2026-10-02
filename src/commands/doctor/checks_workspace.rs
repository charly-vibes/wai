// Workspace doctor checks: skills directory and agent tool directory coverage.
//
// Extracted from mod.rs verbatim (tidy-first move, no behavior change).

use genesis::doctor::CheckStatus;

use super::WaiCheckEntry;
use crate::config::{SKILLS_DIR, agent_config_dir};
use std::collections::HashSet;
use std::path::Path;

/// Known agent tool directories: (dir name, display name)
const AGENT_TOOL_DIRS: &[(&str, &str)] = &[
    (".agents", "Agents"),
    (".amp", "Amp"),
    (".claude", "Claude Code"),
    (".cursor", "Cursor"),
    (".gemini", "Gemini CLI"),
];

/// Find SKILL.md files outside `.wai/` and agent tool directories, and report any not yet
/// imported into wai. Agent tool directories (.claude, .amp, .gemini, .cursor) are excluded
/// because they hold synced copies of skills, not source definitions.
/// Find SKILL.md files outside `.wai/` and agent tool directories, and report any not yet
/// imported into wai. Agent tool directories (.claude, .amp, .gemini, .cursor) are excluded
/// because they hold synced copies of skills, not source definitions.
pub(super) fn check_skills_in_repo(project_root: &Path) -> Vec<WaiCheckEntry> {
    let external_skills = find_external_skill_files(project_root);
    if external_skills.is_empty() {
        return vec![];
    }

    let imported = imported_skill_dirs(project_root);
    let unimported = unimported_skills(project_root, &external_skills, &imported);

    if unimported.is_empty() {
        vec![WaiCheckEntry {
            name: "Skills import".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "{} SKILL.md file(s) found outside wai — all imported",
                external_skills.len()
            ),
            fix: None,
            fix_fn: None,
        }]
    } else {
        vec![WaiCheckEntry {
            name: "Skills import".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} SKILL.md file(s) found outside wai but not imported: {}",
                unimported.len(),
                unimported.join(", ")
            ),
            fix: Some(
                "Copy each skill to .wai/resources/agent-config/skills/<name>/SKILL.md".to_string(),
            ),
            fix_fn: None,
        }]
    }
}

/// Walk the repo (skipping managed/build/agent-tool dirs) collecting
/// SKILL.md files that live outside wai.
fn find_external_skill_files(project_root: &Path) -> Vec<std::path::PathBuf> {
    use walkdir::WalkDir;

    let excluded: Vec<std::path::PathBuf> = skill_scan_exclusions(project_root);
    WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            !excluded.iter().any(|x| p.starts_with(x))
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "SKILL.md" && e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Directories excluded from the external-skill scan: managed, build, and
/// agent tool dirs (the latter hold synced copies, not source definitions).
fn skill_scan_exclusions(project_root: &Path) -> Vec<std::path::PathBuf> {
    let mut dirs = vec![
        project_root.join(".wai"),
        project_root.join("target"),
        project_root.join(".git"),
    ];
    dirs.extend(
        AGENT_TOOL_DIRS
            .iter()
            .map(|(dir, _)| project_root.join(dir)),
    );
    dirs
}

/// Skill directory names already managed by wai (have a SKILL.md inside).
fn imported_skill_dirs(project_root: &Path) -> HashSet<String> {
    let skills_dir = agent_config_dir(project_root).join(SKILLS_DIR);
    if !skills_dir.exists() {
        return HashSet::new();
    }
    std::fs::read_dir(&skills_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("SKILL.md").exists())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect()
}

/// External SKILL.md paths whose parent dir is not yet imported into wai.
fn unimported_skills(
    project_root: &Path,
    external_skills: &[std::path::PathBuf],
    imported: &HashSet<String>,
) -> Vec<String> {
    let mut unimported: Vec<String> = Vec::new();
    for skill_path in external_skills {
        if let Some(parent) = skill_path.parent()
            && let Some(dir_name) = parent.file_name().and_then(|n| n.to_str())
            && !imported.contains(dir_name)
        {
            let rel = skill_path
                .strip_prefix(project_root)
                .unwrap_or(skill_path)
                .display()
                .to_string();
            unimported.push(rel);
        }
    }
    unimported
}

/// Check that detected agent tool directories (.claude, .amp, .gemini, .cursor) are covered by
/// projections, and that wai skills are synced to them.
/// Check that detected agent tool directories (.claude, .amp, .gemini, .cursor) are covered by
/// projections, and that wai skills are synced to them.
pub(super) fn check_agent_tool_coverage(project_root: &Path) -> Vec<WaiCheckEntry> {
    let config_dir = agent_config_dir(project_root);
    let projections_path = config_dir.join(".projections.yml");
    let has_skills = wai_has_skills(&config_dir);

    // Which known agent tool directories exist at the project root?
    let detected: Vec<(&str, &str)> = AGENT_TOOL_DIRS
        .iter()
        .filter(|(dir, _)| project_root.join(dir).is_dir())
        .copied()
        .collect();

    if detected.is_empty() {
        return vec![];
    }

    let projections_opt = load_projections(&projections_path);

    // If the projections file exists and is explicitly empty, the user has intentionally
    // opted out of projections — skip the per-directory coverage warnings entirely.
    if let Some(ref p) = projections_opt
        && p.is_empty()
    {
        return vec![];
    }

    let projections = projections_opt.unwrap_or_default();

    detected
        .iter()
        .map(|(tool_dir, tool_name)| {
            tool_projection_entry(tool_dir, tool_name, &projections, has_skills)
        })
        .collect()
}

/// Whether wai manages any skills (skills dir exists and is non-empty).
fn wai_has_skills(config_dir: &Path) -> bool {
    let skills_dir = config_dir.join(SKILLS_DIR);
    skills_dir.exists()
        && std::fs::read_dir(&skills_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.path().join("SKILL.md").exists())
            })
            .unwrap_or(false)
}

/// Load the projections config.
///
///   None  → file missing or unreadable/unparseable (coverage check should warn)
///   Some(vec) with items → projections configured (coverage check applies)
///   Some(empty vec) → user explicitly set projections: [] (suppress coverage warnings)
fn load_projections(projections_path: &Path) -> Option<Vec<super::checks_sync::ProjectionEntry>> {
    if !projections_path.exists() {
        return None;
    }
    std::fs::read_to_string(projections_path)
        .ok()
        .and_then(|c| serde_yml::from_str::<super::checks_sync::ProjectionsConfig>(&c).ok())
        .map(|cfg| cfg.projections)
}

/// Coverage entry for one detected agent tool dir: unprojected, projected
/// without skills sync, projected fine, or projected with no skills to sync.
fn tool_projection_entry(
    tool_dir: &str,
    tool_name: &str,
    projections: &[super::checks_sync::ProjectionEntry],
    has_skills: bool,
) -> WaiCheckEntry {
    let name = format!("Agent tool projection: {tool_name}");
    // Projections that target this tool dir or a sub-path of it
    let covering: Vec<&super::checks_sync::ProjectionEntry> = projections
        .iter()
        .filter(|p| p.target == tool_dir || p.target.starts_with(&format!("{tool_dir}/")))
        .collect();

    if covering.is_empty() {
        return WaiCheckEntry {
            name,
            status: CheckStatus::Warn,
            message: format!("{tool_dir} directory detected but not in .projections.yml"),
            fix: Some(format!(
                "Add a projection for {tool_dir} in .wai/resources/agent-config/.projections.yml"
            )),
            fix_fn: None,
        };
    }
    if !has_skills {
        return WaiCheckEntry {
            name,
            status: CheckStatus::Pass,
            message: format!("{tool_dir} has a projection defined"),
            fix: None,
            fix_fn: None,
        };
    }
    let skills_synced = covering.iter().any(|p| {
        p.sources
            .iter()
            .any(|s| s == SKILLS_DIR || s.ends_with(&format!("/{SKILLS_DIR}")))
    });
    if skills_synced {
        WaiCheckEntry {
            name,
            status: CheckStatus::Pass,
            message: format!("{tool_dir} projected with skills synced"),
            fix: None,
            fix_fn: None,
        }
    } else {
        WaiCheckEntry {
            name,
            status: CheckStatus::Warn,
            message: format!(
                "{tool_dir} projected but skills source not included — wai skills won't sync to {tool_name}"
            ),
            fix: Some(format!(
                "Add 'skills' to sources for the {tool_dir} projection in .projections.yml"
            )),
            fix_fn: None,
        }
    }
}
