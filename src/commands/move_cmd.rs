use cliclack::log;
use miette::{IntoDiagnostic, Result};

use crate::cli::MoveArgs;
use crate::config::{archives_dir, areas_dir, projects_dir, resources_dir};
use crate::context::require_safe_mode;

use super::require_project;

pub fn run(args: MoveArgs) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("move item")?;

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
    move_item(&source, &destination).into_diagnostic()?;

    log::success(format!("Moved '{}' to {}", item_name, target)).into_diagnostic()?;
    Ok(())
}

/// Move `src` to `dst`, falling back to copy+delete when a cross-device rename
/// is attempted (EXDEV / io::ErrorKind::CrossesDevices).
///
/// PARA items are directories, so the fallback performs a full recursive copy.
/// If the copy fails partway through, the partially-written destination is
/// removed before the error is returned so the workspace is never left in a
/// corrupt state.
fn move_item(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            // Cross-device: fall back to recursive copy then delete.
            match copy_dir_all(src, dst) {
                Ok(()) => {
                    // Copy succeeded; remove the original tree.
                    std::fs::remove_dir_all(src)
                }
                Err(copy_err) => {
                    // Clean up the partial destination before surfacing the error.
                    let _ = std::fs::remove_dir_all(dst);
                    Err(copy_err)
                }
            }
        }
        Err(e) => Err(e),
    }
}

/// Recursively copy the directory tree rooted at `src` into `dst`.
fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    use walkdir::WalkDir;

    std::fs::create_dir_all(dst)?;

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let relative = entry
            .path()
            .strip_prefix(src)
            .expect("walkdir always yields paths under src");
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }

    Ok(())
}

fn find_item(project_root: &std::path::Path, name: &str) -> Result<std::path::PathBuf> {
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
