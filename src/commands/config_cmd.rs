use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::path::Path;
use std::process::Command;

use crate::cli::ConfigCommands;
use crate::config::agent_config_dir;
use crate::context::require_safe_mode;

use super::require_project;

pub fn run(cmd: ConfigCommands) -> Result<()> {
    let project_root = require_project()?;
    let config_dir = agent_config_dir(&project_root);

    match cmd {
        ConfigCommands::Add { config_type, file } => config_add(&config_dir, &config_type, &file),
        ConfigCommands::List => config_list(&config_dir),
        ConfigCommands::Edit { path } => config_edit(&config_dir, &path),
    }
}

fn config_add(config_dir: &Path, config_type: &str, file: &str) -> Result<()> {
    require_safe_mode("add config")?;
    let subdir = config_subdir(config_type)?;

    let source = Path::new(file);
    if !source.exists() {
        return Err(miette::miette!("File not found: {}", file));
    }

    let target_dir = config_dir.join(subdir);
    std::fs::create_dir_all(&target_dir).into_diagnostic()?;

    let filename = source
        .file_name()
        .ok_or_else(|| miette::miette!("Invalid file path"))?;
    let target = target_dir.join(filename);

    std::fs::copy(source, &target).into_diagnostic()?;

    log::success(format!(
        "Added {} '{}'",
        config_type,
        filename.to_str().unwrap_or("?")
    ))
    .into_diagnostic()?;
    println!(
        "  {} Run 'wai sync' to project to tool-specific locations",
        "→".dimmed()
    );
    Ok(())
}

fn config_subdir(config_type: &str) -> Result<&'static str> {
    match config_type {
        "skill" | "skills" => Ok("skills"),
        "rule" | "rules" => Ok("rules"),
        "context" => Ok("context"),
        other => Err(miette::miette!(
            "Unknown config type '{}'. Use: skill, rule, context",
            other
        )),
    }
}

fn config_list(config_dir: &Path) -> Result<()> {
    println!();
    println!("  {} Agent Configuration", "◆".cyan());

    list_config_dir(&config_dir.join("skills"), "Skills")?;
    list_config_dir(&config_dir.join("rules"), "Rules")?;
    list_config_dir(&config_dir.join("context"), "Context")?;

    println!();
    Ok(())
}

fn config_edit(config_dir: &Path, path: &str) -> Result<()> {
    let target = config_dir.join(path);
    if !target.exists() {
        return Err(miette::miette!(
            "Config file not found: {}\n  Run 'wai config list' to see available files",
            path
        ));
    }

    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(&editor)
        .arg(&target)
        .status()
        .into_diagnostic()?;

    if !status.success() {
        return Err(miette::miette!("Editor '{}' exited with error", editor));
    }

    log::success(format!("Edited '{}'", path)).into_diagnostic()?;
    println!(
        "  {} Run 'wai sync' to project changes to tool-specific locations",
        "→".dimmed()
    );
    Ok(())
}

fn list_config_dir(dir: &Path, label: &str) -> Result<()> {
    println!();
    println!("    {} {}", "◆".dimmed(), label);

    if !dir.exists() {
        println!("      {}", "(none)".dimmed());
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .into_diagnostic()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type()
                .map(|t| t.is_file() || t.is_dir())
                .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("      {}", "(none)".dimmed());
    } else {
        for entry in entries {
            if let Some(name) = entry.file_name().to_str() {
                println!("      {} {}", "•".dimmed(), name);
            }
        }
    }

    Ok(())
}
