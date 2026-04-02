use chrono::{Local, NaiveDate};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::config::{HANDOFFS_DIR, STATE_FILE, projects_dir};
use crate::context::current_context;
use crate::json::{BeadsSummary, OpenspecEntry, PrimePayload};
use crate::openspec;
use crate::output::print_json;
use crate::plugin;
use crate::plugin::{detect_main_worktree_root, fetch_memories};
use crate::state::ProjectState;

use super::{beads_counts, beads_summary, list_projects, require_project, resolve_project};

/// Maximum age of a `.pending-resume` file before it is considered stale.
const RESUME_WINDOW: Duration = Duration::from_secs(12 * 60 * 60);

pub fn run(project: Option<String>) -> Result<()> {
    let project_root = require_project()?;
    let json_mode = current_context().json;

    // Graceful empty state: if no projects exist at all, show a helpful prompt
    // rather than crashing with "No projects found."
    if project.is_none() && list_projects(&project_root).is_empty() {
        if json_mode {
            let payload = PrimePayload {
                project: None,
                phase: None,
                resume: false,
                handoff_summary: None,
                next_steps: Vec::new(),
                beads: None,
                openspec: Vec::new(),
            };
            return print_json(&payload);
        }
        let today = Local::now().format("%Y-%m-%d");
        println!("{} wai prime — {}", "◆".cyan(), today);
        println!(
            "{} No active projects. Create one with `wai new project <name>`.",
            "→".cyan()
        );
        return Ok(());
    }

    let resolved = resolve_project(&project_root, project.as_deref())?;
    let project_name = resolved.name;

    // Read phase
    let proj_dir = projects_dir(&project_root).join(&project_name);
    let state_path = proj_dir.join(STATE_FILE);
    let phase = match ProjectState::load(&state_path) {
        Ok(state) => state.current.to_string(),
        Err(_) => "unknown".to_string(),
    };

    // Resume detection: check for .pending-resume signal from wai close.
    // The signal is valid if the .pending-resume file was written within the
    // last 12 hours.  A stale file (older than 12 hours) is deleted with a
    // diagnostic note so the user knows it was found but skipped.
    let pending_resume_path = proj_dir.join(".pending-resume");
    let resume_info = check_pending_resume(&proj_dir, &pending_resume_path);

    // Plugin summaries (beads, openspec) — gathered for both JSON and terminal paths.
    let hook_outputs = plugin::run_hooks(&project_root, "on_status");
    let spec_status = openspec::read_status(&project_root);

    if json_mode {
        return render_json(
            &project_root,
            &project_name,
            &phase,
            resume_info,
            &hook_outputs,
            spec_status,
        );
    }

    // Date header
    let today = Local::now().format("%Y-%m-%d");
    println!("{} wai prime — {}", "◆".cyan(), today);

    // Project + phase
    println!("{} Project: {} [{}]", "•".dimmed(), project_name, phase);

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
                println!("{} Handoff: {} — '{}'", "•".dimmed(), date, snippet);
            }
            // If snippet is empty, it means missing/invalid frontmatter → skip the line
        }
    }

    for output in &hook_outputs {
        if output.label == "beads_stats"
            && let Some(summary) = beads_summary(&output.content)
        {
            println!("{} Beads:   {}", "•".dimmed(), summary);
        }
    }

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

    // bd memories — show up to 5, omit section if unavailable
    if let Some(memories_raw) = fetch_memories(&project_root) {
        let lines: Vec<&str> = memories_raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        if !lines.is_empty() {
            println!("{} Memories:", "◆".cyan());
            let shown = lines.iter().take(5);
            for line in shown {
                let truncated = if line.chars().count() > 80 {
                    format!("{}…", line.chars().take(80).collect::<String>())
                } else {
                    line.to_string()
                };
                println!("  {} {}", "•".dimmed(), truncated);
            }
            if lines.len() > 5 {
                println!(
                    "  {} … and {} more, run `bd memories` to see all",
                    "•".dimmed(),
                    lines.len() - 5
                );
            }
        }
    }

    // Worktree sync suggestion
    if detect_main_worktree_root(&project_root).is_some() {
        println!(
            "{} In a git worktree — run `wai sync --from-main` to sync areas/resources",
            "→".cyan()
        );
    }

    // Suggested next via bd ready --json
    if let Some(next_id) = suggested_next(&project_root) {
        println!("{} Suggested next: bd show {}", "→".cyan(), next_id);
    }

    Ok(())
}

/// Render the prime output as structured JSON.
fn render_json(
    project_root: &Path,
    project_name: &str,
    phase: &str,
    resume_info: Option<(PathBuf, NaiveDate, String)>,
    hook_outputs: &[crate::plugin::HookOutput],
    spec_status: Option<crate::openspec::OpenSpecStatus>,
) -> Result<()> {
    let (resume, handoff_summary, next_steps) =
        if let Some((handoff_path, _, snippet)) = resume_info {
            let steps = extract_next_steps(&handoff_path);
            (true, Some(snippet), steps)
        } else {
            // Normal path: read latest handoff for summary only (no next steps shown).
            let summary = find_latest_handoff(project_root, project_name)?.and_then(|hp| {
                let (_, snippet) = read_handoff_summary(&hp);
                if snippet.is_empty() {
                    None
                } else {
                    Some(snippet)
                }
            });
            (false, summary, Vec::new())
        };

    let beads = hook_outputs
        .iter()
        .find(|o| o.label == "beads_stats")
        .and_then(|o| beads_counts(&o.content))
        .map(|(open, ready)| BeadsSummary { open, ready });

    let openspec = spec_status
        .map(|s| {
            s.changes
                .into_iter()
                .map(|c| OpenspecEntry {
                    name: c.name,
                    done: c.done,
                    total: c.total,
                })
                .collect()
        })
        .unwrap_or_default();

    // Collect suggested next step from bd ready --json as a next_steps item when
    // not resuming (resuming already has steps from the handoff).
    let next_steps = if next_steps.is_empty() {
        suggested_next(project_root)
            .map(|id| vec![format!("bd show {}", id)])
            .unwrap_or_default()
    } else {
        next_steps
    };

    let payload = PrimePayload {
        project: Some(project_name.to_string()),
        phase: Some(phase.to_string()),
        resume,
        handoff_summary,
        next_steps,
        beads,
        openspec,
    };
    print_json(&payload)
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
    if resolved.exists() {
        Some(resolved)
    } else {
        None
    }
}

/// Check the `.pending-resume` signal using a 12-hour freshness window.
///
/// Returns `Some((handoff_path, date, snippet))` when the signal is fresh and
/// the handoff has a non-empty snippet.  When the `.pending-resume` file exists
/// but is older than 12 hours, prints a diagnostic note, deletes the stale
/// file, and returns `None`.
fn check_pending_resume(
    project_dir: &Path,
    pending_path: &Path,
) -> Option<(PathBuf, NaiveDate, String)> {
    // If the signal file doesn't exist there's nothing to do.
    let meta = std::fs::metadata(pending_path).ok()?;

    // Determine the age of the .pending-resume file via its mtime.
    let mtime = meta.modified().ok()?;
    let age = SystemTime::now()
        .duration_since(mtime)
        .unwrap_or(RESUME_WINDOW);

    if age > RESUME_WINDOW {
        // Stale: emit a note to stderr (never stdout, which may carry JSON), delete the file.
        let created_local: chrono::DateTime<Local> = mtime.into();
        eprintln!(
            "note: stale resume signal found (created {}), skipping.",
            created_local.format("%Y-%m-%d %H:%M")
        );
        let _ = std::fs::remove_file(pending_path);
        return None;
    }

    // Fresh: resolve the handoff path from the file contents.
    let hp = read_pending_resume(project_dir)?;
    let date = parse_handoff_date_strict(&hp)?;

    // A pending-resume from a previous day is stale even if the file is
    // recently written (e.g. during tests or clock skew).
    let today = Local::now().date_naive();
    if date < today {
        let _ = std::fs::remove_file(pending_path);
        return None;
    }

    let (_, snippet) = read_handoff_summary(&hp);
    if snippet.is_empty() {
        return None;
    }
    Some((hp, date, snippet))
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
    let handoffs_dir = projects_dir(project_root).join(project).join(HANDOFFS_DIR);

    if !handoffs_dir.exists() {
        return Ok(None);
    }

    let entries = std::fs::read_dir(&handoffs_dir).into_diagnostic()?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
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
