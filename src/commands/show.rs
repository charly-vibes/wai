use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::config::{projects_dir, areas_dir, resources_dir, archives_dir};

use super::require_project;

pub fn run(name: Option<String>) -> Result<()> {
    let project_root = require_project()?;

    if let Some(name) = name {
        // Show details for a specific item
        show_item(&project_root, &name)
    } else {
        // Show overview of all PARA categories
        show_overview(&project_root)
    }
}

fn show_overview(project_root: &std::path::Path) -> Result<()> {
    println!();
    println!("  {} Projects", "◆".cyan());
    list_dirs(&projects_dir(project_root))?;

    println!();
    println!("  {} Areas", "◆".cyan());
    list_dirs(&areas_dir(project_root))?;

    println!();
    println!("  {} Resources", "◆".cyan());
    list_dirs(&resources_dir(project_root))?;

    println!();
    println!("  {} Archives", "◆".cyan());
    list_dirs(&archives_dir(project_root))?;

    println!();
    Ok(())
}

fn show_item(project_root: &std::path::Path, name: &str) -> Result<()> {
    // Check projects, areas, resources, archives
    let proj = projects_dir(project_root).join(name);
    if proj.exists() {
        println!();
        println!("  {} Project: {}", "◆".cyan(), name.bold());
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

fn list_dirs(dir: &std::path::Path) -> Result<()> {
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
            println!("    {} {}", "•".dimmed(), name);
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
