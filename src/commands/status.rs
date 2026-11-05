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
use crate::output::print_envelope;
use crate::plugin;
use crate::state::{Phase, ProjectState};
use crate::workflows;

use super::beads_summary;
use super::pipeline::{PipelineDefinition, PipelineRun, load_pipeline_toml};

// ─── Pipeline state detection ─────────────────────────────────────────────────

enum PipelineStatusInfo {
    Active {
        run: Box<PipelineRun>,
        definition: Box<PipelineDefinition>,
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
            let def_path = pipelines_dir(workspace_root).join(format!("{}.toml", run.pipeline));
            if let Ok(def) = load_pipeline_toml(&def_path) {
                return PipelineStatusInfo::Active {
                    run: Box::new(run),
                    definition: Box::new(def),
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
                if path.is_file()
                    && path.extension().and_then(|e| e.to_str()) == Some("toml")
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

    intro(header_label(&project_root, &config.project.name)).into_diagnostic()?;

    // List projects and their phases
    let project_count = render_projects(&project_root);

    // Plugin status via plugin system
    let hook_outputs = render_plugins(&project_root);

    // OpenSpec status — computed here for both Plugin Info summary and the detail section
    let spec_status = openspec::read_status(&project_root);
    render_plugin_info(&hook_outputs, spec_status.as_ref());

    // OpenSpec status — detailed section
    render_openspec(spec_status.as_ref(), verbose);

    // Pipeline status section
    let pipeline_state = detect_pipeline_state(&project_root);
    render_pipeline(&pipeline_state);

    // Doctor health summary — one line when not clean, silent when green.
    render_health(&project_root);

    // Suggestions — phase-aware when projects exist
    render_suggestions(&project_root, project_count);
    append_pipeline_suggestions(&pipeline_state);
    outro("Run 'wai show' for full overview").into_diagnostic()?;
    Ok(())
}

/// Sorted project directory entries (dirs only).
fn project_entries(proj_dir: &Path) -> Vec<std::path::PathBuf> {
    if !proj_dir.exists() {
        return Vec::new();
    }
    let mut entries: Vec<_> = std::fs::read_dir(proj_dir)
        .into_diagnostic()
        .ok()
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    entries.sort_by_key(|e| e.file_name());
    entries.into_iter().map(|e| e.path()).collect()
}

/// Header label: single/multiple project names, or the workspace name.
fn header_label(project_root: &Path, workspace_name: &str) -> String {
    let entries = project_entries(&projects_dir(project_root));
    match entries.first() {
        Some(first) => {
            let name = first.to_string_lossy().to_string();
            if entries.len() == 1 {
                format!("Project: {}", name.bold())
            } else {
                format!("Project: {} and {} others", name.bold(), entries.len() - 1)
            }
        }
        None => format!("Workspace: {}", workspace_name.bold()),
    }
}

/// Render the Projects section; returns the number of projects listed.
fn render_projects(project_root: &Path) -> usize {
    let mut project_count = 0;
    println!();
    println!("  {} Projects", "◆".cyan());

    let entries = project_entries(&projects_dir(project_root));
    for path in &entries {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let state_path = path.join(STATE_FILE);
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

    if project_count == 0 {
        println!("    {}", "(no projects yet)".dimmed());
    }
    project_count
}

/// Render the Plugins section; returns the collected on_status hook outputs.
fn render_plugins(project_root: &Path) -> Vec<plugin::HookOutput> {
    println!();
    println!("  {} Plugins", "◆".cyan());

    let plugins = plugin::detect_plugins(project_root);
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
    plugin::run_hooks(project_root, "on_status")
}

/// Render the Plugin Info summary when hooks or incomplete spec changes exist.
fn render_plugin_info(
    hook_outputs: &[plugin::HookOutput],
    spec_status: Option<&openspec::OpenSpecStatus>,
) {
    let has_plugin_info = !hook_outputs.is_empty()
        || spec_status.is_some_and(|s| s.changes.iter().any(|c| c.total == 0 || c.done < c.total));
    if !has_plugin_info {
        return;
    }

    println!();
    println!("  {} Plugin Info", "◆".cyan());
    for output in hook_outputs {
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
    if let Some(spec) = spec_status {
        for change in &spec.changes {
            if change.total > 0 && change.done == change.total {
                continue; // hide completed changes; use --verbose to show all
            }
            let pct = (change.done * 100).checked_div(change.total).unwrap_or(0);
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

/// Render the detailed OpenSpec section.
fn render_openspec(spec_status: Option<&openspec::OpenSpecStatus>, verbose: u8) {
    let Some(spec_status) = spec_status else {
        return;
    };
    println!();
    println!("  {} OpenSpec", "◆".cyan());

    let visible_changes: Vec<_> = spec_status
        .changes
        .iter()
        .filter(|c| verbose > 0 || c.total == 0 || c.done < c.total)
        .collect();
    if !visible_changes.is_empty() {
        for change in &visible_changes {
            render_spec_change(change, verbose);
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

/// Render one spec change with optional per-section detail.
fn render_spec_change(change: &openspec::ChangeStatus, verbose: u8) {
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

/// Render the Pipeline section (active run or available pipelines).
fn render_pipeline(pipeline_state: &PipelineStatusInfo) {
    match pipeline_state {
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
}

/// Render the doctor health summary — one line when not clean, silent when green.
fn render_health(project_root: &Path) {
    let health = crate::commands::doctor::health_summary(project_root);
    if health.is_healthy() {
        return;
    }
    let mut parts: Vec<String> = Vec::new();
    if health.summary.warn > 0 {
        parts.push(format!("{} warning(s)", health.summary.warn));
    }
    if health.summary.fail > 0 {
        parts.push(format!("{} failure(s)", health.summary.fail));
    }
    println!();
    println!(
        "  {} Health: {} — run `wai doctor` for details",
        "⚠".yellow(),
        parts.join(", ")
    );
}

/// Render the Suggestions section; returns the suggestions collected.
fn render_suggestions(project_root: &Path, project_count: usize) {
    println!();
    println!("  {} Suggestions", "◆".cyan());

    if project_count == 0 {
        println!(
            "    {} Create your first project: wai new project \"my-app\"",
            "→".dimmed()
        );
        return;
    }

    let mut suggestions: Vec<Suggestion> = Vec::new();
    gather_workflow_suggestions(project_root, &mut suggestions);
    if suggestions.is_empty() {
        push_fallback_suggestions(&mut suggestions);
    }
}

/// Gather phase-aware suggestions from workflow detection across projects.
fn gather_workflow_suggestions(project_root: &Path, suggestions: &mut Vec<Suggestion>) {
    for path in project_entries(&projects_dir(project_root)) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(ctx) = workflows::scan_project(project_root, name) else {
            continue;
        };
        if ctx.phase == Phase::Design {
            render_matrix_suggestion(project_root, name, suggestions);
        }

        let detections = workflows::detect_patterns(&ctx);
        for detection in detections {
            for s in detection.suggestions {
                println!("    {} {}: {}", "→".dimmed(), s.label, s.command);
                suggestions.push(s);
            }
        }
    }
}

/// Matrix awareness (6.2): during the design phase, surface the problem
/// statement, completion count, and the next unfilled cell.
fn render_matrix_suggestion(project_root: &Path, name: &str, suggestions: &mut Vec<Suggestion>) {
    let matrix_dir = crate::matrix::matrix_dir(project_root, name);
    let Some(progress) = crate::matrix::progress(&matrix_dir) else {
        return;
    };
    println!();
    println!(
        "    {} Matrix: {}",
        "◆".magenta(),
        progress.problem.dimmed()
    );
    println!(
        "      Matrix: {}/{} cells filled",
        progress.filled, progress.total
    );
    match progress.next_cell {
        Some(cell) => {
            println!("      {} Next unfilled cell: {}", "→".dimmed(), cell);
            suggestions.push(Suggestion {
                label: "Fill the next matrix cell".to_string(),
                command: format!("# write {cell}/fact.md + a judgment marker"),
            });
        }
        None => {
            suggestions.push(Suggestion {
                label: "Record the matrix decision".to_string(),
                command: "wai matrix decide <approach> <rationale>".to_string(),
            });
        }
    }
}

/// Fallback suggestions when workflow detection produced nothing.
fn push_fallback_suggestions(suggestions: &mut Vec<Suggestion>) {
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

/// Append pipeline suggestions — always after workflow suggestions.
fn append_pipeline_suggestions(pipeline_state: &PipelineStatusInfo) {
    let s = match pipeline_state {
        PipelineStatusInfo::Active { .. } => Suggestion {
            label: "Resume pipeline".to_string(),
            command: "wai pipeline current".to_string(),
        },
        PipelineStatusInfo::Available(_) => Suggestion {
            label: "Choose a pipeline".to_string(),
            command: "wai pipeline suggest".to_string(),
        },
        PipelineStatusInfo::None => return,
    };
    println!("    {} {}: {}", "→".dimmed(), s.label, s.command);
}

fn render_json(project_root: &std::path::Path, _project_name: &str) -> Result<()> {
    let projects = json_projects(project_root);
    let plugins = json_plugins(project_root);
    let hook_outputs = json_hook_outputs(project_root);
    let mut suggestions = json_suggestions(project_root, &projects);
    let openspec = json_openspec(project_root);

    // Pipeline state for JSON
    let pipeline_state = detect_pipeline_state(project_root);
    let pipeline = json_pipeline(&pipeline_state, &mut suggestions);

    let payload = StatusPayload {
        project_root: project_root.display().to_string(),
        projects,
        plugins,
        hook_outputs,
        openspec,
        pipeline,
        suggestions,
    };

    print_envelope(genesis::envelope::EnvelopeKind::Ok, payload, vec![], vec![])?;
    Ok(())
}

/// Collect project name/phase pairs for the JSON payload.
fn json_projects(project_root: &std::path::Path) -> Vec<StatusProject> {
    let mut projects = Vec::new();
    for path in project_entries(&projects_dir(project_root)) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let state_path = path.join(STATE_FILE);
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
    projects
}

fn json_plugins(project_root: &std::path::Path) -> Vec<StatusPlugin> {
    plugin::detect_plugins(project_root)
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
        .collect()
}

fn json_hook_outputs(project_root: &std::path::Path) -> Vec<HookOutput> {
    plugin::run_hooks(project_root, "on_status")
        .into_iter()
        .map(|output| HookOutput {
            label: output.label,
            content: output.content,
        })
        .collect()
}

/// Suggestion list for the JSON payload: workflow detections with fallbacks.
fn json_suggestions(project_root: &std::path::Path, projects: &[StatusProject]) -> Vec<Suggestion> {
    let mut workflow_suggestions: Vec<Suggestion> = Vec::new();
    for path in project_entries(&projects_dir(project_root)) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(ctx) = workflows::scan_project(project_root, name) else {
            continue;
        };
        for detection in workflows::detect_patterns(&ctx) {
            workflow_suggestions.extend(detection.suggestions);
        }
    }

    if projects.is_empty() {
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
    }
}

fn json_openspec(project_root: &std::path::Path) -> Option<StatusOpenSpec> {
    openspec::read_status(project_root).map(|s| StatusOpenSpec {
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
    })
}

/// Build the pipeline JSON block, appending pipeline suggestions.
fn json_pipeline(
    pipeline_state: &PipelineStatusInfo,
    suggestions: &mut Vec<Suggestion>,
) -> Option<StatusPipeline> {
    match pipeline_state {
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
                active: None,
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
        PipelineStatusInfo::None => None,
    }
}

/// Build a genesis StatusBuilder with wai's status contributors registered.
#[allow(dead_code)]
///
/// This enables cross-tool discovery of wai's health status via
/// genesis::status::StatusBuilder.
pub fn status_builder() -> genesis::status::StatusBuilder {
    let mut builder = genesis::status::StatusBuilder::new();
    builder.register_doctor("wai", crate::commands::doctor::build_doctor_runner());
    builder
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
