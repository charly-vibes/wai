use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;

use crate::config::agent_config_dir;
use crate::context::{current_context, require_safe_mode};
use crate::error::WaiError;
use crate::sync_core::{self, Projection};

use super::pipeline::{pipeline_current_status, run_is_incomplete};
use super::require_project;

#[derive(Debug, Deserialize)]
struct ProjectionsConfig {
    #[serde(default)]
    projections: Vec<Projection>,
}

/// Return `true` if `.claude/commands/` is out of date with `config_dir/skills/`.
///
/// Mirrors the traversal logic in `sync_core::execute_claude_code`: scans for
/// hierarchical skills (`<category>/<action>/SKILL.md` where the category dir
/// itself does not contain a `SKILL.md`) and checks whether each expected
/// destination file exists and is not older than its source.
fn claude_code_needs_sync(config_dir: &std::path::Path, cc_dir: &std::path::Path) -> bool {
    let skills_dir = config_dir.join("skills");
    if !skills_dir.exists() {
        return false; // Nothing to sync
    }

    let read_entries = |dir: &std::path::Path| -> Option<Vec<std::path::PathBuf>> {
        std::fs::read_dir(dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect::<Vec<_>>()
            .into()
    };

    let Some(cat_entries) = read_entries(&skills_dir) else {
        return true; // Unreadable source → conservative
    };

    cat_entries
        .iter()
        .filter(|p| p.is_dir())
        .filter(|p| !p.join("SKILL.md").exists()) // flat skills are skipped by execute_claude_code
        .any(|cat_path| category_needs_sync(cat_path, cc_dir, read_entries))
}

/// Check one skill category dir for stale/missing Claude Code projections.
fn category_needs_sync(
    cat_path: &std::path::Path,
    cc_dir: &std::path::Path,
    read_entries: impl Fn(&std::path::Path) -> Option<Vec<std::path::PathBuf>>,
) -> bool {
    let Some(category) = cat_path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let Some(action_entries) = read_entries(cat_path) else {
        return true; // Unreadable category → conservative
    };
    action_entries
        .iter()
        .filter(|p| p.is_dir())
        .map(|p| (p.join("SKILL.md"), p.file_name()))
        .filter(|(skill, _)| skill.exists())
        .any(|(skill, action)| {
            let Some(action) = action.and_then(|n| n.to_str()) else {
                return false;
            };
            let dest = cc_dir.join(category).join(format!("{}.md", action));
            destination_is_stale(&skill, &dest)
        })
}

/// True when the destination is missing, unreadable, or older than the source.
fn destination_is_stale(skill_file: &std::path::Path, dest: &std::path::Path) -> bool {
    if !dest.exists() {
        return true; // Missing destination
    }
    // Compare mtimes: stale destination means needs sync. An unreadable
    // SOURCE is skipped (not conservative-true), matching the original
    // per-artifact `if let Ok` semantics; an unreadable DESTINATION is
    // treated as stale (conservative).
    let Ok(src_mtime) = std::fs::metadata(skill_file).and_then(|m| m.modified()) else {
        return false;
    };
    let Ok(dst_mtime) = std::fs::metadata(dest).and_then(|m| m.modified()) else {
        return true;
    };
    src_mtime > dst_mtime
}

pub fn run(status_only: bool, dry_run: bool, from_main: bool) -> Result<()> {
    let project_root = require_project()?;

    if from_main {
        return run_from_main(&project_root, dry_run);
    }

    let config_dir = agent_config_dir(&project_root);
    let projections_path = config_dir.join(".projections.yml");

    if !projections_path.exists() {
        return Err(WaiError::ConfigSyncError {
            message: "No .projections.yml found in agent-config directory".to_string(),
        }
        .into());
    }

    let config = load_projections(&projections_path)?;
    let quiet = current_context().quiet;
    if config.projections.is_empty() {
        print_no_projections(quiet);
        return Ok(());
    }

    if status_only {
        print_sync_status(&project_root, &config_dir, &config.projections);
        return Ok(());
    }

    if dry_run {
        print_dry_run(&config.projections);
        return Ok(());
    }

    require_safe_mode("sync agent configs")?;
    execute_projections(&project_root, &config_dir, &config.projections)?;

    if quiet {
        return Ok(());
    }
    log::success("Agent configs synced").into_diagnostic()?;
    // Resume breadcrumb for an in-flight pipeline run (wai-xa8i.2).
    let in_flight = pipeline_current_status(&project_root)
        .ok()
        .flatten()
        .is_some_and(|s| s.active && run_is_incomplete(&s));
    if in_flight {
        println!("{} Resume the orchestration: wai pipeline next", "→".cyan());
    }
    Ok(())
}

/// `--status`: report per-projection sync state without writing.
fn print_sync_status(
    project_root: &std::path::Path,
    config_dir: &std::path::Path,
    projections: &[Projection],
) {
    let quiet = current_context().quiet;
    if quiet {
        return;
    }
    println!();
    println!("  {} Sync Status", "◆".cyan());
    for proj in projections {
        let status = if proj.target == "claude-code" {
            let cc_dir = project_root.join(".claude").join("commands");
            if !cc_dir.exists() {
                "not synced".yellow().to_string()
            } else if claude_code_needs_sync(config_dir, &cc_dir) {
                "needs sync".yellow().to_string()
            } else {
                "synced".green().to_string()
            }
        } else if project_root.join(&proj.target).exists() {
            "synced".green().to_string()
        } else {
            "not synced".yellow().to_string()
        };
        println!(
            "    {} {} → {} [{}]",
            "•".dimmed(),
            proj.sources.join(", "),
            proj.target,
            status
        );
    }
    println!();
}

/// `--dry-run`: list what would be synced without writing.
fn print_dry_run(projections: &[Projection]) {
    let quiet = current_context().quiet;
    if quiet {
        return;
    }
    println!();
    println!("  {} Dry-run — no files will be modified", "◆".cyan());
    for proj in projections {
        if proj.target == "claude-code" {
            println!(
                "    {} [claude-code] skills/ → .claude/commands/",
                "•".dimmed()
            );
            continue;
        }
        println!(
            "    {} [{}] {} → {}",
            "•".dimmed(),
            proj.strategy,
            proj.sources.join(", "),
            proj.target
        );
    }
    println!();
}

/// Execute each configured projection against its target.
fn execute_projections(
    project_root: &std::path::Path,
    config_dir: &std::path::Path,
    projections: &[Projection],
) -> Result<()> {
    let quiet = current_context().quiet;
    for proj in projections {
        // Built-in targets are dispatched before strategy-based projections.
        if proj.target == "claude-code" {
            sync_core::execute_claude_code(project_root, config_dir)?;
            continue;
        }
        if proj.target == ".agents" {
            sync_core::execute_agents_projection(project_root, config_dir)?;
            continue;
        }
        execute_strategy(project_root, config_dir, proj, quiet)?;
    }
    Ok(())
}

/// Dispatch one projection by strategy.
fn execute_strategy(
    project_root: &std::path::Path,
    config_dir: &std::path::Path,
    proj: &Projection,
    quiet: bool,
) -> Result<()> {
    match proj.strategy.as_str() {
        "symlink" => sync_core::execute_symlink(project_root, config_dir, proj),
        "inline" => sync_core::execute_inline(project_root, config_dir, proj),
        "reference" => sync_core::execute_reference(project_root, config_dir, proj),
        "copy" => sync_core::execute_copy(project_root, config_dir, proj),
        other => {
            if !quiet {
                log::warning(format!(
                    "Unknown strategy '{}' for target '{}'",
                    other, proj.target
                ))
                .into_diagnostic()?;
            }
            Ok(())
        }
    }
}

/// Sync .wai/areas/ and .wai/resources/ from the main git worktree.
fn run_from_main(project_root: &std::path::Path, dry_run: bool) -> miette::Result<()> {
    use crate::plugin::detect_main_worktree_root;

    let Some(main_root) = detect_main_worktree_root(project_root) else {
        println!("Not in a git worktree — nothing to sync.");
        return Ok(());
    };

    let mut stats = SyncStats::default();
    for dir_name in ["areas", "resources"] {
        sync_dir_from_main(project_root, &main_root, dir_name, dry_run, &mut stats);
    }

    println!(
        "Sync from main: {} copied, {} skipped, {} conflicts",
        stats.synced, stats.skipped, stats.conflicts
    );
    Ok(())
}

/// Counters for a `sync --from-main` pass.
#[derive(Default)]
struct SyncStats {
    synced: u32,
    skipped: u32,
    conflicts: u32,
}

/// Sync one `.wai/<dir_name>` subtree from the main worktree.
fn sync_dir_from_main(
    project_root: &std::path::Path,
    main_root: &std::path::Path,
    dir_name: &str,
    dry_run: bool,
    stats: &mut SyncStats,
) {
    let src_dir = main_root.join(".wai").join(dir_name);
    if !src_dir.exists() {
        return;
    }
    let dst_dir = project_root.join(".wai").join(dir_name);

    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let Ok(rel) = entry.path().strip_prefix(&src_dir) else {
            continue;
        };
        let dst_file = dst_dir.join(rel);
        let Ok(src_content) = std::fs::read(entry.path()) else {
            stats.skipped += 1;
            continue;
        };
        sync_file_from_main(project_root, &dst_file, &src_content, dry_run, stats);
    }
}

/// Copy one file from main unless it conflicts; honors dry-run.
fn sync_file_from_main(
    project_root: &std::path::Path,
    dst_file: &std::path::Path,
    src_content: &[u8],
    dry_run: bool,
    stats: &mut SyncStats,
) {
    let shown = dst_file
        .strip_prefix(project_root)
        .unwrap_or(dst_file)
        .display();
    if dst_file.exists() {
        let dst_content = std::fs::read(dst_file).unwrap_or_default();
        if src_content == dst_content {
            stats.skipped += 1;
            return;
        }
        // Content differs — conflict: skip-and-warn
        println!("! Conflict: {shown} differs from main — skipped (resolve manually)");
        stats.conflicts += 1;
        return;
    }

    if dry_run {
        println!("  [dry-run] would copy: {shown}");
        stats.synced += 1;
        return;
    }

    if let Some(parent) = dst_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(dst_file, src_content) {
        Ok(()) => {
            println!("  ✓ {shown}");
            stats.synced += 1;
        }
        Err(e) => {
            println!("! Failed to copy {shown}: {}", e);
            stats.skipped += 1;
        }
    }
}

/// Load and parse `.projections.yml`.
fn load_projections(path: &std::path::Path) -> Result<ProjectionsConfig> {
    let content = std::fs::read_to_string(path).into_diagnostic()?;
    serde_yml::from_str(&content).map_err(|e| {
        let err: miette::Report = crate::error::WaiError::ConfigError {
            message: format!("Invalid .projections.yml: {}", e),
        }
        .into();
        err
    })
}

/// Print the "nothing configured" hint.
fn print_no_projections(quiet: bool) {
    if quiet {
        return;
    }
    println!();
    println!("  {} No projections configured.", "○".dimmed());
    println!(
        "  {} Edit .wai/resources/agent-config/.projections.yml to add projections",
        "→".dimmed()
    );
    println!();
}
