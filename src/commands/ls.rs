use miette::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

use crate::config::{STATE_FILE, projects_dir};
use crate::state::ProjectState;

struct Row {
    name: String,
    workspace: PathBuf,
    phase: String,
    counts: Option<(u64, u64)>,
}

pub fn run(root: Option<PathBuf>, depth: Option<usize>) -> Result<()> {
    // Task 4.2: Resolve root
    let root = match root {
        Some(r) => {
            if !r.exists() {
                miette::bail!("Root path does not exist: {}", r.display());
            }
            r
        }
        None => {
            dirs::home_dir().ok_or_else(|| miette::miette!("Could not determine home directory"))?
        }
    };

    // Task 4.3: Resolve depth
    let max_depth = depth.unwrap_or(3);

    // Task 4.4: Run workspace discovery
    let workspaces = discover_workspaces(&root, max_depth);

    let mut rows: Vec<Row> = Vec::new();
    let mut any_beads = false;

    // Fetch beads stats for all workspaces in parallel
    let beads_results: HashMap<PathBuf, Option<(u64, u64)>> = {
        let results = Arc::new(Mutex::new(HashMap::new()));
        let handles: Vec<_> = workspaces
            .iter()
            .filter(|ws| ws.join(".beads").exists())
            .map(|ws| {
                let ws = ws.clone();
                let results = Arc::clone(&results);
                std::thread::spawn(move || {
                    let counts = fetch_beads_counts(&ws);
                    results.lock().unwrap().insert(ws, counts);
                })
            })
            .collect();
        for handle in handles {
            let _ = handle.join();
        }
        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
    };

    for workspace in &workspaces {
        // Task 3.1-3.2: beads integration — use pre-fetched parallel results
        let counts = beads_results.get(workspace).copied().flatten();
        if counts.is_some() {
            any_beads = true;
        }

        // Task 2.2: enumerate project names from .wai/projects/ subdirectories
        let proj_dir = projects_dir(workspace);
        if !proj_dir.exists() {
            continue;
        }

        let Ok(entries) = std::fs::read_dir(&proj_dir) else {
            continue;
        };

        let mut project_names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        project_names.sort();

        for name in project_names {
            // Task 2.3: read phase
            let state_path = proj_dir.join(&name).join(STATE_FILE);
            let phase = match ProjectState::load(&state_path) {
                Ok(state) => state.current.to_string(),
                Err(_) => "unknown".to_string(),
            };
            rows.push(Row {
                name,
                workspace: workspace.clone(),
                phase,
                counts,
            });
        }
    }

    // Sort rows by project name (task 4.4)
    rows.sort_by(|a, b| a.name.cmp(&b.name));

    // Task 4.6: empty case
    if rows.is_empty() {
        println!("No wai workspaces found under {}", root.display());
        return Ok(());
    }

    // Task 4.5: duplicate name detection
    let mut name_freq: HashMap<&str, usize> = HashMap::new();
    for row in &rows {
        *name_freq.entry(row.name.as_str()).or_insert(0) += 1;
    }

    let home = dirs::home_dir();

    let display_names: Vec<String> = rows
        .iter()
        .map(|row| {
            if name_freq.get(row.name.as_str()).copied().unwrap_or(0) > 1 {
                let ws_str = match &home {
                    Some(h) => {
                        if let Ok(rel) = row.workspace.strip_prefix(h) {
                            format!("~/{}", rel.display())
                        } else {
                            row.workspace.display().to_string()
                        }
                    }
                    None => row.workspace.display().to_string(),
                };
                format!("{} ({})", row.name, ws_str)
            } else {
                row.name.clone()
            }
        })
        .collect();

    // Compute column widths based on raw (uncolored) strings
    let max_name = display_names.iter().map(|s| s.len()).max().unwrap_or(0);
    let max_phase = rows.iter().map(|r| r.phase.len()).max().unwrap_or(0);

    // Render table
    for (i, row) in rows.iter().enumerate() {
        let name_col = format!("{:<width$}", display_names[i], width = max_name);
        let phase_col = format_phase_col(&row.phase, max_phase);

        if any_beads {
            let counts_str = match row.counts {
                Some((open, ready)) => format!("{} open, {} ready", open, ready),
                None => String::new(),
            };
            println!("{}  {}  {}", name_col, phase_col, counts_str);
        } else {
            println!("{}  {}", name_col, phase_col);
        }
    }

    Ok(())
}

/// Walk the filesystem from `root` up to `max_depth` levels, returning paths of
/// directories that contain `.wai/config.toml`. Hidden directories are skipped
/// during traversal; `.wai/` itself is never recursed into.
fn discover_workspaces(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut workspaces = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Always include the root entry
            if e.depth() == 0 {
                return true;
            }
            // Skip hidden directories during traversal
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                if name.starts_with('.') {
                    return false;
                }
            }
            true
        })
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        if path.join(".wai").join("config.toml").exists() {
            workspaces.push(path.to_path_buf());
        }
    }

    workspaces
}

/// Invoke `bd stats --json` in `workspace` and parse open/ready counts.
/// Returns None on any error (bd not installed, non-zero exit, parse failure).
fn fetch_beads_counts(workspace: &Path) -> Option<(u64, u64)> {
    let output = std::process::Command::new("bd")
        .args(["stats", "--json"])
        .current_dir(workspace)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let summary = json.get("summary")?;
    let open = summary.get("open_issues")?.as_u64()?;
    let ready = summary.get("ready_issues")?.as_u64()?;
    Some((open, ready))
}

/// Format a phase string with color, right-padded to `max_width` inside brackets.
fn format_phase_col(phase: &str, max_width: usize) -> String {
    let colored = match phase {
        "research" => phase.yellow().to_string(),
        "design" => phase.magenta().to_string(),
        "plan" => phase.blue().to_string(),
        "implement" => phase.green().to_string(),
        "review" => phase.cyan().to_string(),
        "archive" | "unknown" => phase.dimmed().to_string(),
        _ => phase.dimmed().to_string(),
    };
    // Pad based on raw length so ANSI codes don't affect alignment
    let padding = " ".repeat(max_width.saturating_sub(phase.len()));
    format!("[{}{}]", colored, padding)
}
