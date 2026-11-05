use std::fs;
use std::path::Path;

use miette::{IntoDiagnostic, Result};

use crate::commands::pipeline::{
    PipelineDefinition, PipelineRun, load_pipeline_toml, render_prompt,
};
use crate::json::{PipelineCurrentPayload, PipelineCurrentStep};

use crate::commands::pipeline::gates::format_gate_summary;

// ─── pipeline_current_status ─────────────────────────────────────────────────

pub fn pipeline_current_status(project_root: &Path) -> Result<Option<PipelineCurrentPayload>> {
    let run_id = match resolve_active_run_id(project_root) {
        Ok(run_id) => run_id,
        Err(_) => return Ok(None),
    };

    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    if !run_path.exists() {
        return Ok(None);
    }

    let run: PipelineRun =
        serde_yml::from_str(&fs::read_to_string(&run_path).into_diagnostic()?)
            .map_err(|e| miette::miette!("Failed to parse run state for '{}': {}", run_id, e))?;

    let def_path =
        crate::config::pipelines_dir(project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    let (step, gate_summary, next_command, message) = current_step_fields(&run, &definition);

    // Epic run tree (wai-vx02.3): when the active run is an epic parent,
    // render its children — every child issue, with the run and mid-flight
    // state for those whose run has been started (topic == issue id).
    let epic_tree = build_epic_tree(project_root, &run, &runs_dir);

    Ok(Some(PipelineCurrentPayload {
        active: true,
        message,
        pipeline: Some(definition.name),
        run_id: Some(run.run_id),
        topic: Some(run.topic),
        step,
        gate_summary,
        next_command,
        epic: epic_tree,
        bypass_count: (!run.bypasses.is_empty()).then_some(run.bypasses.len()),
        last_bypass_reason: run.bypasses.last().map(|b| b.reason.clone()),
    }))
}

/// Total step count of a run's pipeline definition, for mid-flight checks.
fn child_def_steps(project_root: &Path, run: &PipelineRun) -> Option<usize> {
    let def_path =
        crate::config::pipelines_dir(project_root).join(format!("{}.toml", run.pipeline));
    load_pipeline_toml(&def_path).ok().map(|d| d.steps.len())
}

/// Child run ids of an epic parent run that are still mid-flight. Unreadable
/// or unknown runs are skipped, not treated as blockers.
pub(crate) fn epic_parent_midflight_children(
    project_root: &Path,
    run: &PipelineRun,
) -> Vec<String> {
    let mut midflight = Vec::new();
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    for child_run_id in &run.child_runs {
        let child_path = runs_dir.join(format!("{}.yml", child_run_id));
        let Ok(text) = fs::read_to_string(&child_path) else {
            continue;
        };
        let Ok(child_run) = serde_yml::from_str::<PipelineRun>(&text) else {
            continue;
        };
        if child_def_steps(project_root, &child_run)
            .is_some_and(|total| child_run.current_step < total)
        {
            midflight.push(child_run_id.clone());
        }
    }
    midflight
}

/// Build the epic run tree payload for an epic parent run: every child issue,
/// with the run and mid-flight state for those whose run has been started
/// (run topic == issue id).
fn build_epic_tree(
    project_root: &Path,
    run: &PipelineRun,
    runs_dir: &Path,
) -> Option<crate::json::EpicTreePayload> {
    // The tree is anchored on the epic parent run: the active run itself when
    // it is the parent, otherwise the parent run that owns the active child's
    // topic (wai-vx02.5 — a started child surfaces sibling handoffs).
    if run.epic.is_some() {
        build_epic_tree_from_parent(project_root, run, runs_dir)
    } else {
        let parent = find_epic_parent_run_by_topic(runs_dir, &run.topic)?;
        build_epic_tree_from_parent(project_root, &parent, runs_dir)
    }
}

/// Find the epic parent run that owns `topic` as a child issue. Unreadable
/// or non-parent runs are skipped, not errors. (Distinct from
/// `find_epic_parent_run`, which matches by pipeline name + epic id at start
/// time; this resolves the owning parent for an already-started child.)
fn find_epic_parent_run_by_topic(runs_dir: &Path, topic: &str) -> Option<PipelineRun> {
    for entry in fs::read_dir(runs_dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yml") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(candidate) = serde_yml::from_str::<PipelineRun>(&text) else {
            continue;
        };
        if candidate.epic.is_some() && candidate.child_issues.iter().any(|i| i == topic) {
            return Some(candidate);
        }
    }
    None
}

fn build_epic_tree_from_parent(
    project_root: &Path,
    run: &PipelineRun,
    runs_dir: &Path,
) -> Option<crate::json::EpicTreePayload> {
    let epic_id = run.epic.as_ref()?;
    let mut children = Vec::new();
    for issue in &run.child_issues {
        let mut started: Option<(String, PipelineRun)> = None;
        if let Ok(entries) = fs::read_dir(runs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|x| x.to_str()) != Some("yml") {
                    continue;
                }
                let Ok(text) = fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(child_run) = serde_yml::from_str::<PipelineRun>(&text) else {
                    continue;
                };
                if child_run.epic.is_some() || &child_run.topic != issue {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|x| x.to_str()) else {
                    continue;
                };
                started = Some((stem.to_string(), child_run));
                break;
            }
        }
        let node = match started {
            Some((child_run_id, child_run)) => {
                let mid_flight = child_def_steps(project_root, &child_run)
                    .is_some_and(|total| child_run.current_step < total);
                crate::json::ChildRunNode {
                    issue: issue.clone(),
                    run_id: Some(child_run_id),
                    mid_flight,
                    // Handoff-ready = terminal step + artifact recorded
                    // (wai-vx02.5).
                    handoff_ready: !mid_flight && child_run.handoff_artifact.is_some(),
                    handoff_artifact: child_run.handoff_artifact.clone(),
                }
            }
            None => crate::json::ChildRunNode {
                issue: issue.clone(),
                run_id: None,
                mid_flight: false,
                handoff_ready: false,
                handoff_artifact: None,
            },
        };
        children.push(node);
    }
    Some(crate::json::EpicTreePayload {
        epic: epic_id.clone(),
        children,
    })
}

// ─── run-completeness predicate ──────────────────────────────────────────────

/// True when the payload describes an active pipeline run that has not yet
/// reached its final step. Shared by `wai close` (refusal),
/// `clear_complete_pipeline_run`, and prime adoption gating (wai-vx02.1).
pub fn run_is_incomplete(status: &PipelineCurrentPayload) -> bool {
    status.step.is_some()
}

// ─── clear_complete_pipeline_run ─────────────────────────────────────────────

/// If the active pipeline run is complete (its `current_step` has reached the
/// final step), clear the file-based active-run pointers so `wai status` and
/// `wai prime` stop reporting a stale "PIPELINE ACTIVE ... complete" line.
///
/// Returns the completed pipeline's name when a run was cleared, so callers can
/// report it. Returns `None` when there is no active run or the run is still in
/// progress — in both cases the pointers are left intact.
///
/// The run state file itself (`.wai/pipeline-runs/<run>.yml`) is kept as a
/// historical record; only the `.last-run` pointer file at
/// `.wai/resources/pipelines/.last-run` is removed. An active run resolved via
/// the `WAI_PIPELINE_RUN` env var is left to the caller's environment.
pub fn clear_complete_pipeline_run(project_root: &Path) -> Option<String> {
    let status = pipeline_current_status(project_root).ok().flatten()?;
    // Only clear when the run is complete (no current step remains).
    if run_is_incomplete(&status) {
        return None;
    }
    let name = status.pipeline.clone();
    let _ = fs::remove_file(crate::config::last_run_path(project_root));
    name
}

// ─── resolve_active_run_id ────────────────────────────────────────────────────

/// Resolve the active run ID: check `WAI_PIPELINE_RUN` env var first, then
/// fall back to the `.last-run` pointer file at `.wai/resources/pipelines/.last-run`.
pub(crate) fn resolve_active_run_id(project_root: &Path) -> Result<String> {
    // Try env var first
    if let Ok(run_id) = std::env::var("WAI_PIPELINE_RUN")
        && !run_id.is_empty()
    {
        return Ok(run_id);
    }
    // Fall back to .last-run pointer file
    let last_run = crate::config::last_run_path(project_root);
    if last_run.exists() {
        let run_id = fs::read_to_string(&last_run)
            .into_diagnostic()?
            .trim()
            .to_string();
        if !run_id.is_empty() {
            return Ok(run_id);
        }
    }
    Err(miette::miette!(
        "No active pipeline run. Start one with: wai pipeline start <name> --topic=<topic>"
    ))
}

/// Build (step, gate_summary, next_command, message) for the active run.
fn current_step_fields(
    run: &PipelineRun,
    definition: &PipelineDefinition,
) -> (
    Option<PipelineCurrentStep>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    if run.current_step >= definition.steps.len() {
        return (
            None,
            None,
            Some("wai close".to_string()),
            Some(format!(
                "Pipeline '{}' is already complete!",
                definition.name
            )),
        );
    }
    let current_step = &definition.steps[run.current_step];
    (
        Some(PipelineCurrentStep {
            index: run.current_step + 1,
            total: definition.steps.len(),
            id: current_step.id.clone(),
            prompt: render_prompt(&current_step.prompt, &run.topic),
        }),
        Some(format_gate_summary(&current_step.gate)),
        Some("wai pipeline next".to_string()),
        None,
    )
}
