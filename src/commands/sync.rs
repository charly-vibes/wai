use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::path::Path;

use crate::config::agent_config_dir;
use crate::context::require_safe_mode;
use crate::error::WaiError;
use crate::sync_core::{self, Projection};

use super::require_project;

#[derive(Debug, Deserialize)]
struct ProjectionsConfig {
    #[serde(default)]
    projections: Vec<Projection>,
}

pub fn run(status_only: bool) -> Result<()> {
    let project_root = require_project()?;
    let config_dir = agent_config_dir(&project_root);
    let projections_path = config_dir.join(".projections.yml");

    if !projections_path.exists() {
        return Err(WaiError::ConfigSyncError {
            message: "No .projections.yml found in agent-config directory".to_string(),
        }
        .into());
    }

    let content = std::fs::read_to_string(&projections_path).into_diagnostic()?;
    let config: ProjectionsConfig =
        serde_yaml::from_str(&content).map_err(|e| crate::error::WaiError::ConfigError {
            message: format!("Invalid .projections.yml: {}", e),
        })?;

    if config.projections.is_empty() {
        println!();
        println!("  {} No projections configured.", "○".dimmed());
        println!(
            "  {} Edit .wai/resources/agent-config/.projections.yml to add projections",
            "→".dimmed()
        );
        println!();
        return Ok(());
    }

    if status_only {
        println!();
        println!("  {} Sync Status", "◆".cyan());
        for proj in &config.projections {
            let target_path = project_root.join(&proj.target);
            let exists = target_path.exists();
            let status = if exists {
                "synced".green().to_string()
            } else {
                "not synced".yellow().to_string()
            };
            println!(
                "    {} {} → {} [{}]",
                "•".dimmed(),
                proj.sources.join(", "),
                proj.target,
                status
            );
        }
        println!();
        return Ok(());
    }

    require_safe_mode("sync agent configs")?;

    // Execute projections
    for proj in &config.projections {
        match proj.strategy.as_str() {
            "symlink" => sync_core::execute_symlink(&project_root, &config_dir, proj)?,
            "inline" => sync_core::execute_inline(&project_root, &config_dir, proj)?,
            "reference" => sync_core::execute_reference(&project_root, &config_dir, proj)?,
            other => {
                log::warning(format!(
                    "Unknown strategy '{}' for target '{}'",
                    other, proj.target
                ))
                .into_diagnostic()?;
            }
        }
    }

    log::success("Agent configs synced").into_diagnostic()?;
    Ok(())
}
