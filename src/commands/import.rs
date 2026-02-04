use cliclack::log;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::config::agent_config_dir;

use super::require_project;

pub fn run(path: String) -> Result<()> {
    let project_root = require_project()?;
    let config_dir = agent_config_dir(&project_root);
    let source = Path::new(&path);

    if !source.exists() {
        return Err(miette::miette!("Path not found: {}", path));
    }

    if source.is_dir() {
        // Import directory contents (e.g., .claude/)
        import_directory(source, &config_dir)?;
    } else {
        // Import single file (e.g., .cursorrules)
        import_file(source, &config_dir)?;
    }

    log::success(format!("Imported from '{}'", path)).into_diagnostic()?;
    println!("  Run 'wai config list' to review imported files");
    println!("  Run 'wai sync' to project configs to tool locations");
    Ok(())
}

fn import_directory(source: &Path, config_dir: &Path) -> Result<()> {
    let rules_dir = config_dir.join("rules");
    let context_dir = config_dir.join("context");
    let skills_dir = config_dir.join("skills");

    std::fs::create_dir_all(&rules_dir).into_diagnostic()?;
    std::fs::create_dir_all(&context_dir).into_diagnostic()?;
    std::fs::create_dir_all(&skills_dir).into_diagnostic()?;

    for entry in std::fs::read_dir(source).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = entry.file_name();
        let name = filename.to_str().unwrap_or("");

        // Categorize by filename patterns
        let target_dir = if name.contains("rule") || name.contains("cursorrule") {
            &rules_dir
        } else if name.contains("skill") || name.contains("command") {
            &skills_dir
        } else {
            &context_dir
        };

        std::fs::copy(&path, target_dir.join(&filename)).into_diagnostic()?;
        log::info(format!("  Imported {}", name)).into_diagnostic()?;
    }

    Ok(())
}

fn import_file(source: &Path, config_dir: &Path) -> Result<()> {
    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("imported");

    // Determine category from filename
    let target_dir = if filename.contains("rule") {
        config_dir.join("rules")
    } else if filename.contains("skill") || filename.contains("command") {
        config_dir.join("skills")
    } else {
        config_dir.join("context")
    };

    std::fs::create_dir_all(&target_dir).into_diagnostic()?;
    std::fs::copy(source, target_dir.join(filename)).into_diagnostic()?;

    log::info(format!("  Imported {}", filename)).into_diagnostic()?;
    Ok(())
}
