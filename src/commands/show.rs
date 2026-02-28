use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::config::{STATE_FILE, archives_dir, areas_dir, projects_dir, resources_dir};
use crate::context::current_context;
use crate::json::{ShowEntry, ShowItemEntry, ShowItemPayload, ShowPayload};
use crate::output::print_json;
use crate::state::ProjectState;

use super::require_project;

pub fn run(name: Option<String>) -> Result<()> {
    let project_root = require_project()?;
    let json = current_context().json;

    if let Some(name) = name {
        if json {
            show_item_json(&project_root, &name)
        } else {
            show_item(&project_root, &name)
        }
    } else if json {
        show_overview_json(&project_root)
    } else {
        // Show overview of all PARA categories
        show_overview(&project_root)
    }
}

fn show_overview(project_root: &std::path::Path) -> Result<()> {
    println!();
    println!("  {} Projects", "◆".cyan());
    list_dirs_with_phase(&projects_dir(project_root), true)?;

    println!();
    println!("  {} Areas", "◆".cyan());
    list_dirs_with_phase(&areas_dir(project_root), false)?;

    println!();
    println!("  {} Resources", "◆".cyan());
    list_dirs_with_phase(&resources_dir(project_root), false)?;

    println!();
    println!("  {} Archives", "◆".cyan());
    list_dirs_with_phase(&archives_dir(project_root), false)?;

    println!();
    Ok(())
}

fn show_overview_json(project_root: &std::path::Path) -> Result<()> {
    let payload = ShowPayload {
        projects: collect_entries(&projects_dir(project_root), true)?,
        areas: collect_entries(&areas_dir(project_root), false)?,
        resources: collect_entries(&resources_dir(project_root), false)?,
        archives: collect_entries(&archives_dir(project_root), false)?,
    };
    print_json(&payload)?;
    Ok(())
}

fn show_item_json(project_root: &std::path::Path, name: &str) -> Result<()> {
    let candidates = [
        (projects_dir(project_root).join(name), "project"),
        (areas_dir(project_root).join(name), "area"),
        (resources_dir(project_root).join(name), "resource"),
        (archives_dir(project_root).join(name), "archive"),
    ];

    for (path, category) in &candidates {
        if path.exists() {
            let phase = if *category == "project" {
                let state_path = path.join(STATE_FILE);
                match ProjectState::load(&state_path) {
                    Ok(state) => Some(state.current.to_string()),
                    Err(_) => None,
                }
            } else {
                None
            };

            let contents = collect_item_contents(path)?;
            let payload = ShowItemPayload {
                name: name.to_string(),
                category: category.to_string(),
                phase,
                path: path.display().to_string(),
                contents,
            };
            print_json(&payload)?;
            return Ok(());
        }
    }

    Err(crate::error::WaiError::ProjectNotFound {
        name: name.to_string(),
    }
    .into())
}

fn collect_item_contents(dir: &std::path::Path) -> Result<Vec<ShowItemEntry>> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .into_diagnostic()?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut result = Vec::new();
    for entry in entries {
        let file_name = entry.file_name();
        let name = file_name.to_str().unwrap_or("?");
        if name.starts_with('.') {
            continue;
        }
        let ft = entry.file_type().into_diagnostic()?;
        if ft.is_dir() {
            let item_count = std::fs::read_dir(entry.path())
                .map(|rd| rd.filter_map(|e| e.ok()).count())
                .ok();
            result.push(ShowItemEntry {
                name: name.to_string(),
                kind: "dir".to_string(),
                item_count,
            });
        } else {
            result.push(ShowItemEntry {
                name: name.to_string(),
                kind: "file".to_string(),
                item_count: None,
            });
        }
    }
    Ok(result)
}

fn collect_entries(
    dir: &std::path::Path,
    read_phase: bool,
) -> Result<Vec<ShowEntry>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .into_diagnostic()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| !n.starts_with('.'))
                .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut result = Vec::new();
    for entry in entries {
        if let Some(name) = entry.file_name().to_str() {
            let phase = if read_phase {
                let state_path = entry.path().join(STATE_FILE);
                match ProjectState::load(&state_path) {
                    Ok(state) => Some(state.current.to_string()),
                    Err(_) => None,
                }
            } else {
                None
            };

            let artifact_count = count_artifacts(&entry.path());
            let path = entry.path().display().to_string();

            result.push(ShowEntry {
                name: name.to_string(),
                phase,
                artifact_count,
                path,
            });
        }
    }
    Ok(result)
}

fn count_artifacts(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| !n.starts_with('.'))
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

fn show_item(project_root: &std::path::Path, name: &str) -> Result<()> {
    // Check projects, areas, resources, archives
    let proj = projects_dir(project_root).join(name);
    if proj.exists() {
        let phase = read_phase(&proj);
        println!();
        println!("  {} Project: {}  [{}]", "◆".cyan(), name.bold(), phase);
        list_contents(&proj)?;
        println!();
        return Ok(());
    }

    let area = areas_dir(project_root).join(name);
    if area.exists() {
        println!();
        println!("  {} Area: {}", "◆".cyan(), name.bold());
        list_contents(&area)?;
        println!();
        return Ok(());
    }

    let resource = resources_dir(project_root).join(name);
    if resource.exists() {
        println!();
        println!("  {} Resource: {}", "◆".cyan(), name.bold());
        list_contents(&resource)?;
        println!();
        return Ok(());
    }

    Err(crate::error::WaiError::ProjectNotFound {
        name: name.to_string(),
    }
    .into())
}

/// Read and format the phase for a project directory, with color.
fn read_phase(proj_dir: &std::path::Path) -> String {
    let state_path = proj_dir.join(STATE_FILE);
    match ProjectState::load(&state_path) {
        Ok(state) => format_phase(state.current),
        Err(_) => "unknown".dimmed().to_string(),
    }
}

fn format_phase(phase: crate::state::Phase) -> String {
    use crate::state::Phase;
    match phase {
        Phase::Research => "research".yellow().to_string(),
        Phase::Design => "design".magenta().to_string(),
        Phase::Plan => "plan".blue().to_string(),
        Phase::Implement => "implement".green().to_string(),
        Phase::Review => "review".cyan().to_string(),
        Phase::Archive => "archive".dimmed().to_string(),
    }
}

fn list_dirs_with_phase(dir: &std::path::Path, show_phase: bool) -> Result<()> {
    if !dir.exists() {
        println!("    {}", "(none)".dimmed());
        return Ok(());
    }

    let mut found = false;
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .into_diagnostic()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| !n.starts_with('.'))
                .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        if let Some(name) = entry.file_name().to_str() {
            if show_phase {
                let phase = read_phase(&entry.path());
                println!("    {} {}  [{}]", "•".dimmed(), name, phase);
            } else {
                println!("    {} {}", "•".dimmed(), name);
            }
            found = true;
        }
    }

    if !found {
        println!("    {}", "(none)".dimmed());
    }

    Ok(())
}

fn list_contents(dir: &std::path::Path) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .into_diagnostic()?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name();
        let name = name.to_str().unwrap_or("?");
        if name.starts_with('.') {
            continue;
        }
        let ft = entry.file_type().into_diagnostic()?;
        if ft.is_dir() {
            let count = std::fs::read_dir(entry.path())
                .map(|rd| rd.filter_map(|e| e.ok()).count())
                .unwrap_or(0);
            println!(
                "    {} {}/  {} items",
                "•".dimmed(),
                name,
                count.to_string().dimmed()
            );
        } else {
            println!("    {} {}", "•".dimmed(), name);
        }
    }

    Ok(())
}
