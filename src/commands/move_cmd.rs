use cliclack::log;
use miette::{IntoDiagnostic, Result};

use crate::cli::MoveArgs;
use crate::config::{projects_dir, areas_dir, resources_dir, archives_dir};

use super::require_project;

pub fn run(args: MoveArgs) -> Result<()> {
    let project_root = require_project()?;

    let item_name = &args.item;
    let target = &args.target;

    // Find the item in any PARA category
    let source = find_item(&project_root, item_name)?;

    // Determine target directory
    let target_parent = match target.to_lowercase().as_str() {
        "archives" | "archive" => archives_dir(&project_root),
        "projects" | "project" => projects_dir(&project_root),
        "areas" | "area" => areas_dir(&project_root),
        "resources" | "resource" => resources_dir(&project_root),
        _ => {
            return Err(miette::miette!(
                "Unknown target category '{}'. Use: archives, projects, areas, resources",
                target
            ));
        }
    };

    let destination = target_parent.join(item_name);

    if destination.exists() {
        return Err(miette::miette!(
            "Item '{}' already exists in '{}'",
            item_name,
            target
        ));
    }

    std::fs::create_dir_all(&target_parent).into_diagnostic()?;
    std::fs::rename(&source, &destination).into_diagnostic()?;

    log::success(format!("Moved '{}' to {}", item_name, target)).into_diagnostic()?;
    Ok(())
}

fn find_item(
    project_root: &std::path::Path,
    name: &str,
) -> Result<std::path::PathBuf> {
    let candidates = [
        projects_dir(project_root).join(name),
        areas_dir(project_root).join(name),
        resources_dir(project_root).join(name),
        archives_dir(project_root).join(name),
    ];

    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Err(miette::miette!(
        "Item '{}' not found in any PARA category (projects, areas, resources, archives)",
        name
    ))
}
