use cliclack::{intro, outro};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;

use crate::config::{
    ProjectConfig, STATE_FILE, find_project_root, last_run_path, pipelines_dir, projects_dir,
};
use crate::context::current_context;
use crate::error::WaiError;
use crate::json::{
    HookOutput, StatusChange, StatusChangeSection, StatusOpenSpec, StatusPayload, StatusPipeline,
    StatusPipelineActive, StatusPipelineAvailable, StatusPlugin, StatusProject, Suggestion,
};
use crate::openspec;
use crate::output::print_json;
use crate::plugin;
use crate::state::{Phase, ProjectState};
use crate::workflows;

use super::beads_summary;
use super::pipeline::{PipelineDefinition, PipelineRun, load_pipeline_toml};

// ─── Pipeline state detection ─────────────────────────────────────────────────

enum PipelineStatusInfo {
    Active {
        run: PipelineRun,
        definition: PipelineDefinition,
    },
    Available(Vec<(String, PipelineDefinition)>),
    None,
}

fn detect_pipeline_state(workspace_root: &Path) -> PipelineStatusInfo {
    // 1. Try to resolve active run ID: env var → .last-run file
    let last_run_file = last_run_path(workspace_root);
    let run_id = std::env::var("WAI_PIPELINE_RUN")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            fs::read_to_string(&last_run_file)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });

    if let Some(rid) = run_id {
        let run_path = workspace_root
            .join(".wai/pipeline-runs")
            .join(format!("{}.yml", rid));
        if run_path.exists()
            && let Ok(content) = fs::read_to_string(&run_path)
            && let Ok(run) = serde_yml::from_str::<PipelineRun>(&content)
        {
            let def_path =
                pipelines_dir(workspace_root).join(format!("{}.toml", run.pipeline));
            if let Ok(def) = load_pipeline_toml(&def_path) {
                return PipelineStatusInfo::Active {
                    run,
                    definition: def,
                };
            }
        }
        // Stale pointer — fall through to available pipelines
    }

    // 2. No active run — list available pipelines
    let pipelines = pipelines_dir(workspace_root);
    let mut available: Vec<(String, PipelineDefinition)> = Vec::new();
    if pipelines.exists() {
        if let Ok(entries) = fs::read_dir(&pipelines) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("toml")
                    && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                {
                    match load_pipeline_toml(&path) {
                        Ok(def) => available.push((name.to_string(), def)),
                        Err(e) => eprintln!("warning: {}: {}", path.display(), e),
                    }
                }
            }
        }
        available.sort_by(|(a, _), (b, _)| a.cmp(b));
    }

    if available.is_empty() {
        PipelineStatusInfo::None
    } else {
        PipelineStatusInfo::Available(available)
    }
}

pub fn run(verbose: u8) -> Result<()> {
    let project_root = find_project_root().ok_or(WaiError::NotInitialized)?;
    let config = ProjectConfig::load(&project_root)?;
    let context = current_context();

    if context.json {
        return render_json(&project_root, &config.project.name);
    }

    intro(format!("Project: {}", config.project.name.bold())).into_diagnostic()?;

    // List projects and their phases
    let proj_dir = projects_dir(&project_root);
    let mut project_count = 0;
    let mut suggestions: Vec<Suggestion> = Vec::new();

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

    // OpenSpec status — computed here for both Plugin Info summary and the detail section
    let spec_status = openspec::read_status(&project_root);

    let has_plugin_info = !hook_outputs.is_empty()
        || spec_status
            .as_ref()
            .is_some_and(|s| s.changes.iter().any(|c| c.total == 0 || c.done < c.total));

    if has_plugin_info {
        println!();
        println!("  {} Plugin Info", "◆".cyan());
        for output in &hook_outputs {
            if output.label == "beads_stats"
                && let Some(summary) = beads_summary(&output.content)
            {
                println!("    {} beads: {}", "•".dimmed(), summary);
                continue;
            }
            println!("    {} {}:", "•".dimmed(), output.label.bold());
            for line in output.content.lines().take(5) {
                println!("      {}", line.dimmed());
            }
        }
        if let Some(ref spec) = spec_status {
            for change in &spec.changes {
                if change.total > 0 && change.done == change.total {
                    continue; // hide completed changes; use --verbose to show all
                }
                let pct = if change.total > 0 {
                    change.done * 100 / change.total
                } else {
                    0
                };
                println!(
                    "    {} {}: {}/{} ({}%)",
                    "•".dimmed(),
                    change.name,
                    change.done,
                    change.total,
                    pct
                );
            }
        }
    }

    // OpenSpec status — detailed section
    if let Some(ref spec_status) = spec_status {
        println!();
        println!("  {} OpenSpec", "◆".cyan());

        let visible_changes: Vec<_> = spec_status
            .changes
            .iter()
            .filter(|c| verbose > 0 || c.total == 0 || c.done < c.total)
            .collect();
        if !visible_changes.is_empty() {
            for change in &visible_changes {
                let ratio = if change.total > 0 {
                    format!("{}/{}", change.done, change.total)
                } else {
                    "no tasks".to_string()
                };
                println!("    {} {}  [{}]", "•".dimmed(), change.name, ratio.cyan());
                if verbose > 0 {
                    for section in &change.sections {
                        println!(
                            "      {} {}  {}/{}",
                            "·".dimmed(),
                            section.name.dimmed(),
                            section.done,
                            section.total
                        );
                    }
                }
            }
        } else if spec_status.changes.is_empty() {
            println!("    {}", "(no active changes)".dimmed());
        } else {
            println!("    {}", "(all changes complete — use -v to show)".dimmed());
        }

        if verbose > 0 && !spec_status.specs.is_empty() {
            println!();
            println!(
                "    {} specs: {}",
                "·".dimmed(),
                spec_status.specs.join(", ")
            );
        }
    }

    // Pipeline status section
    let pipeline_state = detect_pipeline_state(&project_root);
    match &pipeline_state {
        PipelineStatusInfo::Active { run, definition } => {
            let total = definition.steps.len();
            let step_num = run.current_step + 1;
            println!();
            println!("  {} Pipeline", "◆".cyan());
            println!(
                "    {} PIPELINE ACTIVE: {} step {}/{}",
                "⚡".yellow(),
                run.pipeline,
                step_num,
                total
            );
        }
        PipelineStatusInfo::Available(pipelines) => {
            println!();
            println!("  {} Available pipelines", "◆".cyan());
            for (name, def) in pipelines {
                let desc = def.description.as_deref().unwrap_or("(no description)");
                let steps = def.steps.len();
                println!(
                    "    {} {}  {} ({} steps)",
                    "•".dimmed(),
                    name.bold(),
                    desc.dimmed(),
                    steps
                );
            }
        }
        PipelineStatusInfo::None => {}
    }

    // Suggestions — phase-aware when projects exist
    println!();
    println!("  {} Suggestions", "◆".cyan());

    if project_count == 0 {
        suggestions.push(Suggestion {
            label: "Create your first project".to_string(),
            command: "wai new project \"my-app\"".to_string(),
        });
        println!(
            "    {} Create your first project: wai new project \"my-app\"",
            "→".dimmed()
        );
    } else {
        // Gather phase-aware suggestions from workflow detection across all projects
        let proj_dir = projects_dir(&project_root);
        if proj_dir.exists() {
            let mut entries: Vec<_> = std::fs::read_dir(&proj_dir)
                .into_diagnostic()?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in &entries {
                if let Some(name) = entry.file_name().to_str()
                    && let Some(ctx) = workflows::scan_project(&project_root, name)
                {
                    let detections = workflows::detect_patterns(&ctx);
                    for detection in detections {
                        for s in detection.suggestions {
                            println!("    {} {}: {}", "→".dimmed(), s.label, s.command);
                            suggestions.push(s);
                        }
                    }
                }
            }
        }

        // Fallback suggestions if workflow detection produced nothing
        if suggestions.is_empty() {
            suggestions.push(Suggestion {
                label: "View project phase".to_string(),
                command: "wai phase".to_string(),
            });
            suggestions.push(Suggestion {
                label: "Add artifacts".to_string(),
                command: "wai add research \"...\"".to_string(),
            });
            println!("    {} View project phase: wai phase", "→".dimmed());
            println!(
                "    {} Add artifacts: wai add research \"...\"",
                "→".dimmed()
            );
        }
    }

    // Pipeline suggestions — always appended after workflow suggestions
    match &pipeline_state {
        PipelineStatusInfo::Active { .. } => {
            let s = Suggestion {
                label: "Resume pipeline".to_string(),
                command: "wai pipeline current".to_string(),
            };
            println!("    {} {}: {}", "→".dimmed(), s.label, s.command);
            suggestions.push(s);
        }
        PipelineStatusInfo::Available(_) => {
            let s = Suggestion {
                label: "Choose a pipeline".to_string(),
                command: "wai pipeline suggest".to_string(),
            };
            println!("    {} {}: {}", "→".dimmed(), s.label, s.command);
            suggestions.push(s);
        }
        PipelineStatusInfo::None => {}
    }

    outro("Run 'wai show' for full overview").into_diagnostic()?;
    Ok(())
}

fn render_json(project_root: &std::path::Path, _project_name: &str) -> Result<()> {
    let mut projects = Vec::new();
    let proj_dir = projects_dir(project_root);
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
                        Ok(state) => state.current.to_string(),
                        Err(_) => "unknown".to_string(),
                    }
                } else {
                    "no state".to_string()
                };
                projects.push(StatusProject {
                    name: name.to_string(),
                    phase,
                });
            }
        }
    }

    let plugins = plugin::detect_plugins(project_root)
        .into_iter()
        .map(|p| StatusPlugin {
            name: p.def.name,
            status: if p.detected {
                "detected".to_string()
            } else {
                "not found".to_string()
            },
            detected: p.detected,
        })
        .collect();

    let hook_outputs = plugin::run_hooks(project_root, "on_status")
        .into_iter()
        .map(|output| HookOutput {
            label: output.label,
            content: output.content,
        })
        .collect();

    // Gather workflow suggestions from pattern detection
    let mut workflow_suggestions: Vec<Suggestion> = Vec::new();
    if proj_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&proj_dir)
    {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && let Some(name) = entry.file_name().to_str()
                && let Some(ctx) = workflows::scan_project(project_root, name)
            {
                for detection in workflows::detect_patterns(&ctx) {
                    workflow_suggestions.extend(detection.suggestions);
                }
            }
        }
    }

    let mut suggestions = if projects.is_empty() {
        vec![Suggestion {
            label: "Create your first project".to_string(),
            command: "wai new project \"my-app\"".to_string(),
        }]
    } else if !workflow_suggestions.is_empty() {
        workflow_suggestions
    } else {
        vec![
            Suggestion {
                label: "View project phase".to_string(),
                command: "wai phase".to_string(),
            },
            Suggestion {
                label: "Add artifacts".to_string(),
                command: "wai add research \"...\"".to_string(),
            },
        ]
    };

    let openspec = openspec::read_status(project_root).map(|s| StatusOpenSpec {
        specs: s.specs,
        changes: s
            .changes
            .into_iter()
            .map(|c| StatusChange {
                name: c.name,
                done: c.done,
                total: c.total,
                sections: c
                    .sections
                    .into_iter()
                    .map(|sec| StatusChangeSection {
                        name: sec.name,
                        done: sec.done,
                        total: sec.total,
                    })
                    .collect(),
            })
            .collect(),
    });

    // Pipeline state for JSON
    let pipeline_state = detect_pipeline_state(project_root);
    let pipeline = match &pipeline_state {
        PipelineStatusInfo::Active { run, definition } => {
            suggestions.push(Suggestion {
                label: "Resume pipeline".to_string(),
                command: "wai pipeline current".to_string(),
            });
            Some(StatusPipeline {
                active: Some(StatusPipelineActive {
                    name: run.pipeline.clone(),
                    step: run.current_step + 1,
                    total: definition.steps.len(),
                }),
                available: Vec::new(),
            })
        }
        PipelineStatusInfo::Available(pipelines) => {
            suggestions.push(Suggestion {
                label: "Choose a pipeline".to_string(),
                command: "wai pipeline suggest".to_string(),
            });
            Some(StatusPipeline {
                active: Option::None,
                available: pipelines
                    .iter()
                    .map(|(name, def)| StatusPipelineAvailable {
                        name: name.clone(),
                        description: def.description.clone(),
                        steps: def.steps.len(),
                    })
                    .collect(),
            })
        }
        PipelineStatusInfo::None => Option::None,
    };

    let payload = StatusPayload {
        project_root: project_root.display().to_string(),
        projects,
        plugins,
        hook_outputs,
        openspec,
        pipeline,
        suggestions,
    };

    print_json(&payload)?;
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
