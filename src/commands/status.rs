use cliclack::{intro, outro};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::config::{find_project_root, projects_dir, ProjectConfig, STATE_FILE};
use crate::error::WaiError;
use crate::plugin;
use crate::state::{Phase, ProjectState};

pub fn run() -> Result<()> {
    let project_root = find_project_root().ok_or(WaiError::NotInitialized)?;
    let config = ProjectConfig::load(&project_root)?;

    intro(format!("Project: {}", config.project.name.bold())).into_diagnostic()?;

    // List projects and their phases
    let proj_dir = projects_dir(&project_root);
    let mut project_count = 0;

    println!();
    println!("  {} Projects", "◆".cyan());

    if proj_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&proj_dir)
            .into_diagnostic()?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            if let Some(name) = entry.file_name().to_str() {
                let state_path = entry.path().join(STATE_FILE);
                let phase = if state_path.exists() {
                    match ProjectState::load(&state_path) {
                        Ok(state) => format_phase(state.current),
                        Err(_) => "unknown".dimmed().to_string(),
                    }
                } else {
                    "no state".dimmed().to_string()
                };

                println!("    {} {}  [{}]", "•".dimmed(), name, phase);
                project_count += 1;
            }
        }
    }

    if project_count == 0 {
        println!("    {}", "(no projects yet)".dimmed());
    }

    // Plugin status via plugin system
    println!();
    println!("  {} Plugins", "◆".cyan());

    let plugins = plugin::detect_plugins(&project_root);
    let mut any_detected = false;
    for p in &plugins {
        if p.detected {
            println!(
                "    {} {}  {}",
                "•".dimmed(),
                p.def.name.bold(),
                "detected".green()
            );
            any_detected = true;
        }
    }
    if !any_detected {
        println!("    {}", "(none detected)".dimmed());
    }

    // Run on_status hooks for enrichment
    let hook_outputs = plugin::run_hooks(&project_root, "on_status");
    if !hook_outputs.is_empty() {
        println!();
        println!("  {} Plugin Info", "◆".cyan());
        for output in &hook_outputs {
            println!("    {} {}:", "•".dimmed(), output.label.bold());
            for line in output.content.lines().take(5) {
                println!("      {}", line.dimmed());
            }
        }
    }

    // Suggestions
    println!();
    println!("  {} Suggestions", "◆".cyan());

    if project_count == 0 {
        println!(
            "    {} Create your first project: wai new project \"my-app\"",
            "→".dimmed()
        );
    } else {
        println!(
            "    {} View project phase: wai phase",
            "→".dimmed()
        );
        println!(
            "    {} Add artifacts: wai add research \"...\"",
            "→".dimmed()
        );
    }

    outro("Run 'wai show' for full overview").into_diagnostic()?;
    Ok(())
}

fn format_phase(phase: Phase) -> String {
    match phase {
        Phase::Research => "research".yellow().to_string(),
        Phase::Design => "design".magenta().to_string(),
        Phase::Plan => "plan".blue().to_string(),
        Phase::Implement => "implement".green().to_string(),
        Phase::Review => "review".cyan().to_string(),
        Phase::Archive => "archive".dimmed().to_string(),
    }
}
