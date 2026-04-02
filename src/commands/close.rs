use miette::Result;
use std::path::Path;

use crate::config::projects_dir;
use crate::context::{current_context, require_safe_mode};
use crate::plugin;
use crate::plugin::{detect_main_worktree_root, store_memory};

use super::handoff::create_handoff;
use super::reflect::{count_handoffs_since, read_reflect_meta};
use super::{require_project, resolve_project};

pub fn run(project: Option<String>, remember: bool) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("create handoff")?;

    let resolved = resolve_project(&project_root, project.as_deref())?;
    let project_name = resolved.name;

    let handoff_path = create_handoff(&project_root, &project_name)?;

    // Write .pending-resume signal so wai prime can detect a mid-task resume
    let proj_dir = projects_dir(&project_root).join(&project_name);
    if let Ok(relative) = handoff_path.strip_prefix(&proj_dir) {
        let _ = std::fs::write(
            proj_dir.join(".pending-resume"),
            relative.to_string_lossy().as_bytes(),
        );
    }

    let context = current_context();
    let quiet = context.quiet;

    if !quiet {
        // Display relative path from project root
        let display_path = handoff_path
            .strip_prefix(&project_root)
            .unwrap_or(&handoff_path);
        println!("✓ Handoff created: {}", display_path.display());

        // Get uncommitted files (silently skip if git unavailable or not a repo)
        let uncommitted = get_uncommitted_files(&project_root);
        if !uncommitted.is_empty() {
            const MAX_FILES: usize = 10;
            let (shown, extra) = if uncommitted.len() > MAX_FILES {
                (&uncommitted[..MAX_FILES], uncommitted.len() - MAX_FILES)
            } else {
                (&uncommitted[..], 0)
            };
            let file_list = shown.join(", ");
            if extra > 0 {
                println!("! Uncommitted changes: {}… and {} more", file_list, extra);
            } else {
                println!("! Uncommitted changes: {}", file_list);
            }
        }

        // Print next-steps reminder
        let beads_detected = plugin::detect_plugins(&project_root)
            .iter()
            .any(|p| p.def.name == "beads" && p.detected);

        let git_add_part = if uncommitted.is_empty() {
            "git add <files> && git commit".to_string()
        } else {
            let quoted: Vec<String> = uncommitted.iter().map(|f| format!("\"{}\"", f)).collect();
            format!("git add {} && git commit", quoted.join(" "))
        };

        let in_worktree = detect_main_worktree_root(&project_root).is_some();

        let next_steps = if beads_detected && in_worktree {
            format!(
                "wai sync --from-main && bd sync --from-main && {}",
                git_add_part
            )
        } else if beads_detected {
            format!("bd sync --from-main && {}", git_add_part)
        } else if in_worktree {
            format!("wai sync --from-main && {}", git_add_part)
        } else {
            git_add_part
        };
        println!("→ Next: {}", next_steps);

        // 5.1–5.3: Reflect nudge — show if 5+ handoffs since last reflect.
        if !context.json {
            let proj_dir = projects_dir(&project_root).join(&project_name);
            let handoffs_dir = proj_dir.join("handoffs");
            let meta = read_reflect_meta(&proj_dir).unwrap_or(None);
            let last_reflected = meta
                .as_ref()
                .map(|m| m.last_reflected.as_str())
                .unwrap_or("");
            let session_count = count_handoffs_since(&handoffs_dir, last_reflected);
            if session_count >= 5 {
                // Determine which target files exist.
                let has_claude = project_root.join("CLAUDE.md").exists();
                let has_agents = project_root.join("AGENTS.md").exists();
                let target_hint = match (has_claude, has_agents) {
                    (true, true) => "CLAUDE.md and AGENTS.md",
                    (true, false) => "CLAUDE.md",
                    (false, true) => "AGENTS.md",
                    (false, false) => "CLAUDE.md",
                };
                println!(
                    "→ {} sessions since last reflect — run `wai reflect` to update {}",
                    session_count, target_hint
                );
            }
        }
    }

    if remember {
        if context.safe {
            eprintln!("! --remember skipped in safe mode");
        } else if context.no_input {
            eprintln!("! --remember skipped in no-input mode");
        } else {
            print!("→ Insight to save (bd remember): ");
            let _ = std::io::Write::flush(&mut std::io::stdout());
            let mut line = String::new();
            if std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut line).is_ok() {
                let text = line.trim().to_string();
                if !text.is_empty() {
                    match store_memory(&project_root, &text) {
                        Ok(()) => println!("✓ Memory saved"),
                        Err(e) => eprintln!("! Could not save memory: {}", e),
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_uncommitted_files(project_root: &Path) -> Vec<String> {
    let Ok(output) = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.get(3..))
        .map(|s| s.trim().to_string())
        .collect()
}
