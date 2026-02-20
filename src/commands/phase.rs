use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::cli::PhaseCommands;
use crate::config::{STATE_FILE, projects_dir};
use crate::context::require_safe_mode;
use crate::json::Suggestion;
use crate::plugin;
use crate::state::{Phase, ProjectState};

use super::{print_suggestions, require_project};

pub fn run(cmd: PhaseCommands) -> Result<()> {
    let project_root = require_project()?;

    // Find the first project to operate on
    let (project_name, state_path) = find_active_project(&project_root)?;

    match cmd {
        PhaseCommands::Show => {
            let state = ProjectState::load(&state_path)?;
            println!();
            println!("  {} Project: {}", "◆".cyan(), project_name.bold());
            println!(
                "  {} Current phase: {}",
                "◆".cyan(),
                format_phase(state.current)
            );

            if state.history.len() > 1 {
                println!();
                println!("  {} Phase history:", "◆".cyan());
                for entry in &state.history {
                    let status = if entry.completed.is_some() {
                        "✓".green().to_string()
                    } else {
                        "●".blue().to_string()
                    };
                    let started = entry.started.format("%Y-%m-%d %H:%M");
                    println!(
                        "    {} {} (started {})",
                        status,
                        entry.phase,
                        started.to_string().dimmed()
                    );
                }
            }

            // Show available transitions
            println!();
            if let Some(next) = state.current.next() {
                println!("  {} wai phase next  → {}", "→".dimmed(), next);
            }
            if let Some(prev) = state.current.prev() {
                println!("  {} wai phase back  → {}", "→".dimmed(), prev);
            }
            println!();

            Ok(())
        }
        PhaseCommands::Next => {
            require_safe_mode("advance phase")?;
            let mut state = ProjectState::load(&state_path)?;
            let new_phase = state.advance()?;
            state.save(&state_path)?;

            plugin::run_hooks(&project_root, "on_phase_transition");

            log::success(format!(
                "Project '{}' advanced to phase: {}",
                project_name, new_phase
            ))
            .into_diagnostic()?;

            // Phase-specific suggestions after advancing
            let suggestions = get_phase_suggestions(new_phase);
            print_suggestions(&suggestions);

            Ok(())
        }
        PhaseCommands::Back => {
            require_safe_mode("move phase back")?;
            let mut state = ProjectState::load(&state_path)?;
            let new_phase = state.go_back()?;
            state.save(&state_path)?;

            plugin::run_hooks(&project_root, "on_phase_transition");

            log::success(format!(
                "Project '{}' moved back to phase: {}",
                project_name, new_phase
            ))
            .into_diagnostic()?;

            // Phase-specific suggestions after going back
            let suggestions = get_phase_suggestions(new_phase);
            print_suggestions(&suggestions);

            Ok(())
        }
        PhaseCommands::Set { phase } => {
            require_safe_mode("set phase")?;
            let target = Phase::from_str(&phase).ok_or_else(|| {
                miette::miette!(
                    "Unknown phase '{}'. Valid phases: {}",
                    phase,
                    Phase::ALL
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

            let mut state = ProjectState::load(&state_path)?;
            state.transition_to(target)?;
            state.save(&state_path)?;

            plugin::run_hooks(&project_root, "on_phase_transition");

            log::success(format!(
                "Project '{}' set to phase: {}",
                project_name, target
            ))
            .into_diagnostic()?;

            // Phase-specific suggestions after setting phase
            let suggestions = get_phase_suggestions(target);
            print_suggestions(&suggestions);

            Ok(())
        }
    }
}

fn find_active_project(project_root: &std::path::Path) -> Result<(String, std::path::PathBuf)> {
    let proj_dir = projects_dir(project_root);
    if proj_dir.exists() {
        for entry in std::fs::read_dir(&proj_dir).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            if entry.file_type().into_diagnostic()?.is_dir() {
                let state_path = entry.path().join(STATE_FILE);
                if let Some(name) = entry.file_name().to_str() {
                    return Ok((name.to_string(), state_path));
                }
            }
        }
    }

    Err(crate::error::WaiError::NoProjectContext.into())
}

/// Generate phase-specific suggestions based on the current phase
fn get_phase_suggestions(phase: Phase) -> Vec<Suggestion> {
    match phase {
        Phase::Research => vec![
            Suggestion {
                label: "Add research".to_string(),
                command: "wai add research \"...\"".to_string(),
            },
            Suggestion {
                label: "Search existing research".to_string(),
                command: "wai search \"...\"".to_string(),
            },
            Suggestion {
                label: "Check status".to_string(),
                command: "wai status".to_string(),
            },
        ],
        Phase::Design => vec![
            Suggestion {
                label: "Add design".to_string(),
                command: "wai add design \"...\"".to_string(),
            },
            Suggestion {
                label: "Review research".to_string(),
                command: "wai search \"research\"".to_string(),
            },
            Suggestion {
                label: "Show project details".to_string(),
                command: "wai show".to_string(),
            },
        ],
        Phase::Plan => vec![
            Suggestion {
                label: "Add plan".to_string(),
                command: "wai add plan \"...\"".to_string(),
            },
            Suggestion {
                label: "Review designs".to_string(),
                command: "wai search \"design\"".to_string(),
            },
            Suggestion {
                label: "Show project timeline".to_string(),
                command: "wai timeline".to_string(),
            },
        ],
        Phase::Implement => vec![
            Suggestion {
                label: "Show project details".to_string(),
                command: "wai show".to_string(),
            },
            Suggestion {
                label: "Add implementation notes".to_string(),
                command: "wai add plan \"...\"".to_string(),
            },
            Suggestion {
                label: "Check status".to_string(),
                command: "wai status".to_string(),
            },
        ],
        Phase::Review => vec![
            Suggestion {
                label: "Review project timeline".to_string(),
                command: "wai timeline".to_string(),
            },
            Suggestion {
                label: "Search artifacts".to_string(),
                command: "wai search \"...\"".to_string(),
            },
            Suggestion {
                label: "Create handoff".to_string(),
                command: "wai handoff create".to_string(),
            },
        ],
        Phase::Archive => vec![
            Suggestion {
                label: "Create handoff".to_string(),
                command: "wai handoff create".to_string(),
            },
            Suggestion {
                label: "Review project timeline".to_string(),
                command: "wai timeline".to_string(),
            },
            Suggestion {
                label: "Show project details".to_string(),
                command: "wai show".to_string(),
            },
        ],
    }
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
