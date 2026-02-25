use miette::{IntoDiagnostic, Result};
use std::io::IsTerminal;
use std::path::Path;

use crate::config::projects_dir;
use crate::context::{current_context, require_safe_mode};
use crate::error::WaiError;
use crate::plugin;

use super::handoff::create_handoff;
use super::reflect::{count_handoffs_since, read_reflect_meta};
use super::require_project;

pub fn run(project: Option<String>) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("create handoff")?;

    let project_name = resolve_project(&project_root, project)?;

    let handoff_path = create_handoff(&project_root, &project_name)?;

    // Write .pending-resume signal so wai prime can detect a mid-task resume
    let proj_dir = projects_dir(&project_root).join(&project_name);
    if let Ok(relative) = handoff_path.strip_prefix(&proj_dir) {
        let _ = std::fs::write(proj_dir.join(".pending-resume"), relative.to_string_lossy().as_bytes());
    }

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
        let quoted: Vec<String> = uncommitted
            .iter()
            .map(|f| format!("\"{}\"", f))
            .collect();
        format!("git add {} && git commit", quoted.join(" "))
    };

    let next_steps = if beads_detected {
        format!("bd sync --from-main && {}", git_add_part)
    } else {
        git_add_part
    };
    println!("→ Next: {}", next_steps);

    // 5.1–5.3: Reflect nudge — show if 5+ handoffs since last reflect.
    let context = current_context();
    if !context.json {
        let proj_dir = projects_dir(&project_root).join(&project_name);
        let handoffs_dir = proj_dir.join("handoffs");
        let meta = read_reflect_meta(&proj_dir).unwrap_or(None);
        let last_reflected = meta.as_ref().map(|m| m.last_reflected.as_str()).unwrap_or("");
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

    Ok(())
}

fn resolve_project(project_root: &Path, project: Option<String>) -> Result<String> {
    if let Some(name) = project {
        let proj_dir = projects_dir(project_root).join(&name);
        if !proj_dir.exists() {
            let available = list_projects(project_root);
            let available_str = if available.is_empty() {
                "none".to_string()
            } else {
                available.join(", ")
            };
            miette::bail!(
                "Project '{}' not found. Available projects: {}",
                name,
                available_str
            );
        }
        return Ok(name);
    }

    let mut projects = list_projects(project_root);
    projects.sort();

    match projects.len() {
        0 => miette::bail!(
            "No projects found. Create one with `wai new project <name>`."
        ),
        1 => Ok(projects.remove(0)),
        _ => {
            let ctx = current_context();
            if ctx.no_input || !std::io::stdin().is_terminal() {
                return Err(WaiError::NonInteractive {
                    message: format!(
                        "Multiple projects found ({}). Use --project <name> to specify one.",
                        projects.join(", ")
                    ),
                }
                .into());
            }
            let mut sel = cliclack::select("Multiple projects found — which one?");
            for name in &projects {
                sel = sel.item(name.clone(), name.as_str(), "");
            }
            let selected: String = sel.interact().into_diagnostic()?;
            Ok(selected)
        }
    }
}

fn list_projects(project_root: &Path) -> Vec<String> {
    let dir = projects_dir(project_root);
    if !dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
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
