use miette::Result;
use owo_colors::OwoColorize;

use crate::cli::ProjectCommands;
use crate::config::{STATE_FILE, projects_dir};
use crate::state::ProjectState;
use crate::suggestions::SuggestionEngine;

use super::{list_projects, require_project};

pub fn run(cmd: ProjectCommands) -> Result<()> {
    // Handle wrong-order detection first (doesn't require a workspace)
    if let ProjectCommands::External(ref args) = cmd {
        let patterns_owned = crate::cli::wai_subcommand_patterns();
        let valid_patterns: Vec<(&str, &str)> = patterns_owned
            .iter()
            .map(|(v, n)| (v.as_str(), n.as_str()))
            .collect();
        let engine = SuggestionEngine::new();
        let sub = args.first().map(|s| s.as_str()).unwrap_or("");

        if let Some(suggestion) = engine.suggest_order("project", sub, &valid_patterns) {
            miette::bail!(
                "{}. {}",
                suggestion.message(),
                "Run 'wai --help' to see available commands."
            );
        }
        miette::bail!(
            "Unknown subcommand 'project {}'. Run 'wai project --help' for available commands.",
            args.join(" ")
        );
    }

    let project_root = require_project()?;

    match cmd {
        ProjectCommands::Use { name } => {
            let projects = list_projects(&project_root);

            if let Some(name) = name {
                // Validate project exists
                let proj_dir = projects_dir(&project_root).join(&name);
                if !proj_dir.exists() {
                    let available = if projects.is_empty() {
                        "none".to_string()
                    } else {
                        projects.join(", ")
                    };
                    miette::bail!(
                        "Project '{}' not found. Available projects: {}",
                        name,
                        available
                    );
                }

                // Detect shell and print appropriate export syntax
                let shell = std::env::var("SHELL").unwrap_or_default();
                if shell.ends_with("/fish") || shell.ends_with("\\fish") {
                    println!("set -gx WAI_PROJECT {}", name);
                } else {
                    println!("export WAI_PROJECT={}", name);
                }

                // Print hint to stderr when stdout is a terminal
                if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
                    eprintln!(
                        "{}",
                        format!(
                            "# Paste the line above, or run: eval $(wai project use {})",
                            name
                        )
                        .dimmed()
                    );
                }
            } else {
                // No args: list available projects with phases
                if projects.is_empty() {
                    println!("No projects. Create one with `wai new project <name>`.");
                    return Ok(());
                }

                println!();
                for name in &projects {
                    let state_path = projects_dir(&project_root).join(name).join(STATE_FILE);
                    let phase = ProjectState::load(&state_path)
                        .map(|s| s.current.to_string())
                        .unwrap_or_else(|_| "unknown".to_string());
                    println!("  {} {}  [{}]", "•".dimmed(), name.bold(), phase.dimmed());
                }
                println!();
                println!("  {} Set project: wai project use <name>", "→".dimmed());
                println!();
            }

            Ok(())
        }
        ProjectCommands::External(_) => unreachable!(),
    }
}
