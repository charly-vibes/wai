use chrono::{Local, NaiveDate};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::config::{HANDOFFS_DIR, STATE_FILE, projects_dir};
use crate::context::current_context;
use crate::error::WaiError;
use crate::openspec;
use crate::plugin;
use crate::state::ProjectState;

use super::require_project;

pub fn run(project: Option<String>) -> Result<()> {
    let project_root = require_project()?;

    let project_name = resolve_project(&project_root, project)?;

    // Read phase
    let proj_dir = projects_dir(&project_root).join(&project_name);
    let state_path = proj_dir.join(STATE_FILE);
    let phase = match ProjectState::load(&state_path) {
        Ok(state) => state.current.to_string(),
        Err(_) => "unknown".to_string(),
    };

    // Date header
    let today = Local::now().format("%Y-%m-%d");
    println!("{} wai prime — {}", "◆".cyan(), today);

    // Project + phase
    println!(
        "{} Project: {} [{}]",
        "•".dimmed(),
        project_name,
        phase
    );

    // Resume detection: check for .pending-resume signal from wai close
    let today_naive = Local::now().date_naive();
    let resume_info = read_pending_resume(&proj_dir).and_then(|hp| {
        // Parse date strictly: None if frontmatter absent or date malformed (→ treated as stale)
        let date = parse_handoff_date_strict(&hp)?;
        if date != today_naive {
            return None; // stale signal
        }
        let (_, snippet) = read_handoff_summary(&hp);
        if snippet.is_empty() {
            return None;
        }
        Some((hp, date, snippet))
    });

    if let Some((handoff_path, date, snippet)) = resume_info {
        println!("⚡ RESUMING: {} — '{}'", date.format("%Y-%m-%d"), snippet);
        let steps = extract_next_steps(&handoff_path);
        if !steps.is_empty() {
            println!("  Next Steps:");
            for step in &steps {
                println!("    {}", step);
            }
        }
    } else {
        // Handoff (normal path)
        if let Some(handoff_path) = find_latest_handoff(&project_root, &project_name)? {
            let (date, snippet) = read_handoff_summary(&handoff_path);
            if !snippet.is_empty() {
                println!(
                    "{} Handoff: {} — '{}'",
                    "•".dimmed(),
                    date,
                    snippet
                );
            }
            // If snippet is empty, it means missing/invalid frontmatter → skip the line
        }
    }

    // Plugin summaries (beads, openspec)
    let hook_outputs = plugin::run_hooks(&project_root, "on_status");
    for output in &hook_outputs {
        if output.label == "beads_stats" {
            if let Some(summary) = beads_summary(&output.content) {
                println!("{} Beads:   {}", "•".dimmed(), summary);
            }
        }
    }

    let spec_status = openspec::read_status(&project_root);
    if let Some(ref spec) = spec_status {
        for change in &spec.changes {
            let pct = if change.total > 0 {
                change.done * 100 / change.total
            } else {
                0
            };
            println!(
                "{} Spec:    {}: {}/{} ({}%)",
                "•".dimmed(),
                change.name,
                change.done,
                change.total,
                pct
            );
        }
    }

    // Suggested next via bd ready --json
    if let Some(next_id) = suggested_next(&project_root) {
        println!("{} Suggested next: bd show {}", "→".cyan(), next_id);
    }

    Ok(())
}

/// Parse the `date:` field from a handoff's frontmatter, returning `None` if
/// frontmatter is absent, the date field is missing, or parsing fails.
fn parse_handoff_date_strict(path: &Path) -> Option<NaiveDate> {
    let content = std::fs::read_to_string(path).ok()?;
    let body = content.trim_start();
    if !body.starts_with("---") {
        return None;
    }
    let after = &body[3..];
    let close = after.find("\n---")?;
    after[..close].lines().find_map(|line| {
        line.trim()
            .strip_prefix("date:")
            .and_then(|v| NaiveDate::parse_from_str(v.trim(), "%Y-%m-%d").ok())
    })
}

/// Read `.pending-resume` and resolve the handoff path it points to.
///
/// Returns `None` if the file is absent, the path is invalid, or the target
/// handoff file does not exist on disk.
pub fn read_pending_resume(project_dir: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(project_dir.join(".pending-resume")).ok()?;
    let relative = content.trim();
    if relative.is_empty() {
        return None;
    }
    let resolved = project_dir.join(relative);
    if resolved.exists() { Some(resolved) } else { None }
}

/// Extract lines from the `## Next Steps` section of a handoff file.
///
/// Collects lines from after the heading until the next `##` heading or EOF.
/// Skips blank lines and lines starting with `<!--`.
pub fn extract_next_steps(handoff_path: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(handoff_path) else {
        return Vec::new();
    };
    let mut in_section = false;
    let mut items = Vec::new();
    for line in content.lines() {
        if line.starts_with("## Next Steps") {
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with("## ") {
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("<!--") {
                continue;
            }
            items.push(line.to_string());
        }
    }
    items
}

/// Find the most recent handoff file for a project (sorted descending by filename).
pub fn find_latest_handoff(project_root: &Path, project: &str) -> Result<Option<PathBuf>> {
    let handoffs_dir = projects_dir(project_root)
        .join(project)
        .join(HANDOFFS_DIR);

    if !handoffs_dir.exists() {
        return Ok(None);
    }

    let entries = std::fs::read_dir(&handoffs_dir).into_diagnostic()?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("md")
        })
        .map(|e| e.path())
        .collect();

    files.sort_by(|a, b| b.cmp(a)); // descending: newest filename first
    Ok(files.into_iter().next())
}

/// Parse frontmatter date + first paragraph snippet (up to 80 chars) from a handoff file.
///
/// Returns `(date, snippet)`. If frontmatter is missing or invalid, returns an empty
/// snippet (caller omits the handoff line). If frontmatter parses but no paragraph is
/// found, snippet is `"no summary yet"`.
pub fn read_handoff_summary(path: &Path) -> (NaiveDate, String) {
    let fallback_date = Local::now().date_naive();

    let Ok(content) = std::fs::read_to_string(path) else {
        return (fallback_date, String::new());
    };

    // Expect frontmatter between opening "---" and closing "---"
    let body = content.trim_start();
    if !body.starts_with("---") {
        return (fallback_date, String::new());
    }

    let after_open = &body[3..];
    let Some(close_pos) = after_open.find("\n---") else {
        return (fallback_date, String::new());
    };

    let frontmatter = &after_open[..close_pos];
    let rest = &after_open[close_pos + 4..]; // skip "\n---"

    // Parse date from frontmatter: look for "date: YYYY-MM-DD"
    let date = frontmatter
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("date:").map(|v| v.trim().to_string())
        })
        .and_then(|date_str| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok())
        .unwrap_or(fallback_date);

    // Find first paragraph in the body (non-empty, non-heading, non-code-fence line)
    let snippet = find_first_paragraph(rest);

    (date, snippet)
}

/// Extract the first paragraph line from markdown body content.
/// Skips headings (# ...) and blank lines. Returns "no summary yet" if none found.
fn find_first_paragraph(body: &str) -> String {
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("```") {
            continue;
        }
        // Skip HTML comments
        if trimmed.starts_with("<!--") {
            continue;
        }
        // Found a paragraph line
        if trimmed.len() <= 80 {
            return trimmed.to_string();
        } else {
            return format!("{}...", &trimmed[..77]);
        }
    }
    "no summary yet".to_string()
}

/// Parse `bd stats` output and return a compact one-liner like "3 open issues (2 ready)".
fn beads_summary(content: &str) -> Option<String> {
    let mut open: Option<u64> = None;
    let mut ready: Option<u64> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("Open:") {
            open = val.trim().parse().ok();
        } else if let Some(val) = trimmed.strip_prefix("Ready to Work:") {
            ready = val.trim().parse().ok();
        }
    }
    if let (Some(o), Some(r)) = (open, ready) {
        Some(format!("{} open issues ({} ready)", o, r))
    } else {
        None
    }
}

/// Invoke `bd ready --json` and return the first issue's `id`, or `None` on any error.
fn suggested_next(project_root: &Path) -> Option<String> {
    let output = std::process::Command::new("bd")
        .args(["ready", "--json"])
        .current_dir(project_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let first = json.as_array()?.first()?;
    first.get("id")?.as_str().map(|s| s.to_string())
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
                        "Multiple projects found ({}). Use `wai prime --project <name>` to specify one.",
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
