use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::path::Path;

use crate::cli::ConfigCommands;
use crate::config::agent_config_dir;
use crate::context::require_safe_mode;

use super::require_project;

pub fn run(cmd: ConfigCommands) -> Result<()> {
    let project_root = require_project()?;
    let config_dir = agent_config_dir(&project_root);

    match cmd {
        ConfigCommands::Add { config_type, file } => {
            require_safe_mode("add config")?;
            let subdir = match config_type.as_str() {
                "skill" | "skills" => "skills",
                "rule" | "rules" => "rules",
                "context" => "context",
                other => {
                    return Err(miette::miette!(
                        "Unknown config type '{}'. Use: skill, rule, context",
                        other
                    ));
                }
            };

            let source = Path::new(&file);
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
        ConfigCommands::List => {
            println!();
            println!("  {} Agent Configuration", "◆".cyan());

            list_config_dir(&config_dir.join("skills"), "Skills")?;
            list_config_dir(&config_dir.join("rules"), "Rules")?;
            list_config_dir(&config_dir.join("context"), "Context")?;

            println!();
            Ok(())
        }
    }
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
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
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
