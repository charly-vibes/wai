use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;

use crate::config::agent_config_dir;
use crate::context::{current_context, require_safe_mode};
use crate::error::WaiError;
use crate::sync_core::{self, Projection};

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

    let cat_entries = match std::fs::read_dir(&skills_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => return true, // Unreadable source → conservative
    };

    for cat_entry in cat_entries {
        let cat_path = cat_entry.path();
        if !cat_path.is_dir() {
            continue;
        }
        if cat_path.join("SKILL.md").exists() {
            continue; // Flat skill, skipped by execute_claude_code
        }

        let category = match cat_path.file_name().and_then(|n| n.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        let action_entries = match std::fs::read_dir(&cat_path) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect::<Vec<_>>(),
            Err(_) => return true,
        };

        for action_entry in action_entries {
            let action_path = action_entry.path();
            if !action_path.is_dir() {
                continue;
            }
            let skill_file = action_path.join("SKILL.md");
            if !skill_file.exists() {
                continue;
            }

            let action = match action_path.file_name().and_then(|n| n.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let dest = cc_dir.join(&category).join(format!("{}.md", action));
            if !dest.exists() {
                return true; // Missing destination
            }

            // Compare mtimes: stale destination means needs sync
            if let Ok(src_meta) = std::fs::metadata(&skill_file) {
                if let Ok(dst_meta) = std::fs::metadata(&dest) {
                    if let (Ok(src_mtime), Ok(dst_mtime)) =
                        (src_meta.modified(), dst_meta.modified())
                        && src_mtime > dst_mtime
                    {
                        return true; // Source newer than destination
                    }
                } else {
                    return true; // Can't read dest metadata → conservative
                }
            }
        }
    }

    false
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

    let content = std::fs::read_to_string(&projections_path).into_diagnostic()?;
    let config: ProjectionsConfig =
        serde_yml::from_str(&content).map_err(|e| crate::error::WaiError::ConfigError {
            message: format!("Invalid .projections.yml: {}", e),
        })?;

    let quiet = current_context().quiet;

    if config.projections.is_empty() {
        if !quiet {
            println!();
            println!("  {} No projections configured.", "○".dimmed());
            println!(
                "  {} Edit .wai/resources/agent-config/.projections.yml to add projections",
                "→".dimmed()
            );
            println!();
        }
        return Ok(());
    }

    if status_only {
        if !quiet {
            println!();
            println!("  {} Sync Status", "◆".cyan());
            for proj in &config.projections {
                if proj.target == "claude-code" {
                    let cc_dir = project_root.join(".claude").join("commands");
                    let status = if !cc_dir.exists() {
                        "not synced".yellow().to_string()
                    } else if claude_code_needs_sync(&config_dir, &cc_dir) {
                        "needs sync".yellow().to_string()
                    } else {
                        "synced".green().to_string()
                    };
                    println!(
                        "    {} [claude-code] → .claude/commands/ [{}]",
                        "•".dimmed(),
                        status
                    );
                    continue;
                }
                let target_path = project_root.join(&proj.target);
                let exists = target_path.exists();
                let status = if exists {
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
        return Ok(());
    }

    if dry_run {
        if !quiet {
            println!();
            println!("  {} Dry-run — no files will be modified", "◆".cyan());
            for proj in &config.projections {
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
        return Ok(());
    }

    require_safe_mode("sync agent configs")?;

    // Execute projections
    for proj in &config.projections {
        // Built-in targets are dispatched before strategy-based projections.
        if proj.target == "claude-code" {
            sync_core::execute_claude_code(&project_root, &config_dir)?;
            continue;
        }
        match proj.strategy.as_str() {
            "symlink" => sync_core::execute_symlink(&project_root, &config_dir, proj)?,
            "inline" => sync_core::execute_inline(&project_root, &config_dir, proj)?,
            "reference" => sync_core::execute_reference(&project_root, &config_dir, proj)?,
            "copy" => sync_core::execute_copy(&project_root, &config_dir, proj)?,
            other => {
                if !quiet {
                    log::warning(format!(
                        "Unknown strategy '{}' for target '{}'",
                        other, proj.target
                    ))
                    .into_diagnostic()?;
                }
            }
        }
    }

    if !quiet {
        log::success("Agent configs synced").into_diagnostic()?;
    }
    Ok(())
}

/// Sync .wai/areas/ and .wai/resources/ from the main git worktree.
fn run_from_main(project_root: &std::path::Path, dry_run: bool) -> miette::Result<()> {
    use crate::plugin::detect_main_worktree_root;

    let main_root = match detect_main_worktree_root(project_root) {
        Some(r) => r,
        None => {
            println!("Not in a git worktree — nothing to sync.");
            return Ok(());
        }
    };

    let dirs_to_sync = ["areas", "resources"];
    let wai_subdir = ".wai";

    let mut synced = 0u32;
    let mut skipped = 0u32;
    let mut conflicts = 0u32;

    for dir_name in &dirs_to_sync {
        let src_dir = main_root.join(wai_subdir).join(dir_name);
        let dst_dir = project_root.join(wai_subdir).join(dir_name);

        if !src_dir.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel = match entry.path().strip_prefix(&src_dir) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let dst_file = dst_dir.join(rel);
            let src_content = match std::fs::read(entry.path()) {
                Ok(c) => c,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            if dst_file.exists() {
                let dst_content = std::fs::read(&dst_file).unwrap_or_default();
                if src_content == dst_content {
                    skipped += 1;
                    continue;
                }
                // Content differs — conflict: skip-and-warn
                println!(
                    "! Conflict: {} differs from main — skipped (resolve manually)",
                    dst_file
                        .strip_prefix(project_root)
                        .unwrap_or(&dst_file)
                        .display()
                );
                conflicts += 1;
                continue;
            }

            if dry_run {
                println!(
                    "  [dry-run] would copy: {}",
                    dst_file
                        .strip_prefix(project_root)
                        .unwrap_or(&dst_file)
                        .display()
                );
                synced += 1;
                continue;
            }

            if let Some(parent) = dst_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&dst_file, &src_content) {
                Ok(()) => {
                    println!(
                        "  ✓ {}",
                        dst_file
                            .strip_prefix(project_root)
                            .unwrap_or(&dst_file)
                            .display()
                    );
                    synced += 1;
                }
                Err(e) => {
                    println!(
                        "! Failed to copy {}: {}",
                        dst_file
                            .strip_prefix(project_root)
                            .unwrap_or(&dst_file)
                            .display(),
                        e
                    );
                    skipped += 1;
                }
            }
        }
    }

    println!(
        "Sync from main: {} copied, {} skipped, {} conflicts",
        synced, skipped, conflicts
    );
    Ok(())
}
