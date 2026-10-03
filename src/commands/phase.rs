use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::cli::{PhaseArgs, PhaseCommands};
use crate::config::{STATE_FILE, projects_dir};
use crate::context::require_safe_mode;
use crate::json::Suggestion;
use crate::plugin;
use crate::state::{Phase, ProjectState};

use super::{ProjectSource, print_suggestions, require_project, resolve_project};

pub fn run(args: PhaseArgs) -> Result<()> {
    let project_root = require_project()?;

    let resolved = resolve_project(&project_root, args.project.as_deref())?;
    let project_name = &resolved.name;
    let state_path = projects_dir(&project_root)
        .join(project_name)
        .join(STATE_FILE);

    match args.command.unwrap_or(PhaseCommands::Show) {
        PhaseCommands::Show => cmd_show(&resolved, &state_path),
        PhaseCommands::Next => cmd_next(&project_root, project_name, &state_path),
        PhaseCommands::Back => cmd_back(&project_root, project_name, &state_path),
        PhaseCommands::Set { phase } => cmd_set(&project_root, project_name, &state_path, phase),
    }
}

fn cmd_show(resolved: &super::ResolvedProject, state_path: &std::path::Path) -> Result<()> {
    let state = ProjectState::load(state_path)?;
    println!();
    let source_hint = format_source(resolved.source);
    println!(
        "  {} Project: {}{}",
        "◆".cyan(),
        resolved.name.bold(),
        source_hint
    );
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

fn cmd_next(
    project_root: &std::path::Path,
    project_name: &str,
    state_path: &std::path::Path,
) -> Result<()> {
    require_safe_mode("advance phase")?;
    let mut state = ProjectState::load(state_path)?;

    // Design → plan matrix gate (openspec add-decision-matrix 6.1):
    // a project with a matrix must have a current decision before
    // leaving the design phase. Projects without a matrix are never
    // gated.
    if state.current == Phase::Design {
        enforce_design_gate(project_root, project_name)?;
    }

    let new_phase = state.advance()?;
    state.save(state_path)?;

    plugin::run_hooks(project_root, "on_phase_transition");

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

fn cmd_back(
    project_root: &std::path::Path,
    project_name: &str,
    state_path: &std::path::Path,
) -> Result<()> {
    require_safe_mode("move phase back")?;
    let mut state = ProjectState::load(state_path)?;
    let new_phase = state.go_back()?;
    state.save(state_path)?;

    plugin::run_hooks(project_root, "on_phase_transition");

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

fn cmd_set(
    project_root: &std::path::Path,
    project_name: &str,
    state_path: &std::path::Path,
    phase: String,
) -> Result<()> {
    require_safe_mode("set phase")?;
    let target = Phase::parse(&phase).ok_or_else(|| {
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

    let mut state = ProjectState::load(state_path)?;

    // Same matrix gate as `phase next` — `wai phase set plan` must not
    // bypass the design → plan requirement (6.1).
    if state.current == Phase::Design && target == Phase::Plan {
        enforce_design_gate(project_root, project_name)?;
    }

    state.transition_to(target)?;
    state.save(state_path)?;

    plugin::run_hooks(project_root, "on_phase_transition");

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

/// Design → plan matrix gate (openspec add-decision-matrix 6.1).
fn enforce_design_gate(project_root: &std::path::Path, project_name: &str) -> Result<()> {
    let matrix_dir = crate::matrix::matrix_dir(project_root, project_name);
    match crate::matrix::design_gate(&matrix_dir) {
        Some(crate::matrix::GateCheck::Block(msg)) => {
            miette::bail!("{msg}");
        }
        Some(crate::matrix::GateCheck::Warn(msg)) => {
            log::warning(msg).into_diagnostic()?;
        }
        Some(crate::matrix::GateCheck::Pass) | None => {}
    }
    Ok(())
}

/// Generate phase-specific suggestions based on the current phase
fn get_phase_suggestions(phase: Phase) -> Vec<Suggestion> {
    match phase {
        Phase::Research => research_suggestions(),
        Phase::Design => design_suggestions(),
        Phase::Plan => plan_suggestions(),
        Phase::Implement => implement_suggestions(),
        Phase::Review => review_suggestions(),
        Phase::Archive => archive_suggestions(),
    }
}

fn research_suggestions() -> Vec<Suggestion> {
    vec![
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
    ]
}

fn design_suggestions() -> Vec<Suggestion> {
    vec![
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
    ]
}

fn plan_suggestions() -> Vec<Suggestion> {
    vec![
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
    ]
}

fn implement_suggestions() -> Vec<Suggestion> {
    vec![
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
    ]
}

fn review_suggestions() -> Vec<Suggestion> {
    vec![
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
    ]
}

fn archive_suggestions() -> Vec<Suggestion> {
    vec![
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
    ]
}

fn format_source(source: ProjectSource) -> String {
    match source {
        ProjectSource::Flag => format!(" {}", "[via --project]".dimmed()),
        ProjectSource::EnvVar => format!(" {}", "[via WAI_PROJECT]".dimmed()),
        ProjectSource::AutoDetect | ProjectSource::Interactive => String::new(),
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
