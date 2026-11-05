use std::fs;
use std::path::Path;
use std::path::PathBuf;

use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::commands::pipeline::{
    BypassEntry, PipelineDefinition, PipelineRun, PipelineStep, load_pipeline_toml,
    write_artifact_lock,
};
use crate::commands::require_project;
use crate::context::{current_context, require_safe_mode};

use super::current::{epic_parent_midflight_children, resolve_active_run_id};
use crate::commands::pipeline::gates::{evaluate_gates, find_step_artifact_paths};
use crate::commands::pipeline::queries::print_step;

// ─── bypass ───────────────────────────────────────────────────────────────────

/// Record a disclosed gate bypass in the active run's ledger (wai-giqg).
/// Hooks cannot self-record a `--no-verify` bypass — the flag skips them —
/// so the agent records it explicitly at bypass time. With no active run
/// this is a silent no-op (spec: no entry written, no error raised).
pub(crate) fn cmd_bypass(reason: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline bypass")?;

    let Ok(run_id) = resolve_active_run_id(&project_root) else {
        return Ok(()); // no active run: silent no-op per spec
    };
    let run_path = run_state_path(&project_root, &run_id)?;
    if !run_path.exists() {
        return Ok(()); // dangling pointer: treat as no active run
    }
    let mut run = load_run_state(&run_path, &run_id)?;
    run.bypasses.push(BypassEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        reason: reason.to_string(),
    });
    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    let quiet = current_context().quiet;
    if !quiet {
        println!(
            "✓ Recorded bypass #{} in run '{}': {}",
            run.bypasses.len(),
            run_id,
            reason
        );
    }
    Ok(())
}

// ─── next ─────────────────────────────────────────────────────────────────────

pub(crate) fn cmd_next() -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline next")?;

    // Cross-agent collision surfacing (wai-76dq): advisory warnings before
    // any gate check so a failing gate still shows them. Never hard-fails.
    for warning in crate::commands::prime::collision_warnings(&project_root) {
        println!("⚠ {warning}");
    }

    let (_run_id, run_path, run, definition) = load_advance_context(&project_root)?;

    // 5. Evaluate gates (if configured) before allowing advancement
    let current_step = &definition.steps[run.current_step];
    if gate_blocks_advancement(current_step, &run, &definition, &project_root)? {
        return Ok(());
    }

    // 5b. Lock artifacts if step has lock = true
    if current_step.lock {
        lock_step_artifacts(&project_root, &run.run_id, &current_step.id)?;
    }

    // 6. Advance step
    let next_step = run.current_step + 1;
    let updated = PipelineRun {
        current_step: next_step,
        ..run
    };
    let yaml = serde_yml::to_string(&updated)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    // 7. Print next step or completion block
    if next_step >= definition.steps.len() {
        print_pipeline_complete(&definition.name);
    } else {
        print_step(&definition, next_step, &updated.topic);
    }

    Ok(())
}

/// Evaluate the current step's gates; returns true when failures block advancement.
fn gate_blocks_advancement(
    current_step: &PipelineStep,
    run: &PipelineRun,
    definition: &PipelineDefinition,
    project_root: &Path,
) -> Result<bool> {
    let Some(ref gate) = current_step.gate else {
        return Ok(false);
    };
    let failures = evaluate_gates(gate, current_step, run, definition, project_root)?;
    if failures.is_empty() {
        return Ok(false);
    }
    report_gate_failures(&failures, &current_step.id);
    Ok(true)
}

/// `.wai/pipeline-runs/<run-id>.yml`.
fn run_state_path(project_root: &Path, run_id: &str) -> Result<PathBuf> {
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    if !run_path.exists() {
        miette::bail!(
            "Run state file not found for run '{}'. The run may have been deleted or the ID is stale.",
            run_id
        );
    }
    Ok(run_path)
}

fn load_run_state(run_path: &Path, run_id: &str) -> Result<PipelineRun> {
    serde_yml::from_str(&fs::read_to_string(run_path).into_diagnostic()?)
        .map_err(|e| miette::miette!("Failed to parse run state for '{}': {}", run_id, e))
}

/// Record a handoff artifact path (project-root-relative) on a pipeline run's
/// state file (wai-vx02.5). Called by `wai handoff create` when a run is
/// active, so the epic tree can link child runs to their handoff artifacts.
pub(crate) fn record_handoff_artifact(
    project_root: &Path,
    run_id: &str,
    artifact: &str,
) -> Result<()> {
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    let mut run = load_run_state(&run_path, run_id)?;
    run.handoff_artifact = Some(artifact.to_string());
    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;
    Ok(())
}

fn report_gate_failures(failures: &[String], step_id: &str) {
    println!();
    println!("  {} Gate check failed for step '{}':", "✗".red(), step_id);
    println!();
    for f in failures {
        println!("    {} {}", "✗".red(), f);
    }
    println!();
    println!(
        "  {} Resolve the above before running `wai pipeline next`",
        "→".cyan()
    );
}

fn lock_step_artifacts(project_root: &Path, run_id: &str, step_id: &str) -> Result<()> {
    let artifact_paths = find_step_artifact_paths(project_root, run_id, step_id);
    if artifact_paths.is_empty() {
        miette::bail!("Cannot lock step '{}' with no artifacts.", step_id);
    }
    for path in &artifact_paths {
        write_artifact_lock(path, run_id, step_id)?;
    }
    log::info(format!(
        "Locked {} artifact(s) for step '{}'",
        artifact_paths.len(),
        step_id
    ))
    .into_diagnostic()?;
    Ok(())
}

fn print_pipeline_complete(name: &str) {
    println!("──────────────────────────────────────────────");
    println!("Pipeline '{}' complete!", name);
    println!();
    println!("Next: wai close");
    println!("      wai pipeline suggest   # start another pipeline");
}

// ─── approve ─────────────────────────────────────────────────────────────────

pub(crate) fn cmd_approve() -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline approve")?;

    let run_id = resolve_active_run_id(&project_root)?;
    let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    if !run_path.exists() {
        miette::bail!("No active pipeline run.");
    }
    let mut run: PipelineRun =
        serde_yml::from_str(&fs::read_to_string(&run_path).into_diagnostic()?)
            .map_err(|e| miette::miette!("Failed to parse run state: {}", e))?;

    let def_path =
        crate::config::pipelines_dir(&project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    if run.current_step >= definition.steps.len() {
        miette::bail!("Pipeline run is already complete.");
    }

    let step_id = &definition.steps[run.current_step].id;
    let now = chrono::Utc::now().to_rfc3339();
    run.approvals.insert(step_id.clone(), now);

    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    log::success(format!(
        "Approved step '{}'. Run 'wai pipeline next' to advance.",
        step_id
    ))
    .into_diagnostic()?;

    Ok(())
}

// ─── lock ─────────────────────────────────────────────────────────────────────

pub(crate) fn cmd_lock() -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline lock")?;

    // 1. Resolve active run
    let run_id = resolve_active_run_id(&project_root)?;

    // 2. Load run state
    let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    if !run_path.exists() {
        miette::bail!(
            "Run state file not found for run '{}'. The run may have been deleted or the ID is stale.",
            run_id
        );
    }
    let run: PipelineRun =
        serde_yml::from_str(&fs::read_to_string(&run_path).into_diagnostic()?)
            .map_err(|e| miette::miette!("Failed to parse run state for '{}': {}", run_id, e))?;

    // 3. Load pipeline definition
    let def_path =
        crate::config::pipelines_dir(&project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    // 4. Check not already complete
    if run.current_step >= definition.steps.len() {
        miette::bail!(
            "Pipeline run '{}' is already complete. No step to lock.",
            run_id
        );
    }

    let current_step = &definition.steps[run.current_step];

    // 5. Find artifacts tagged with this step
    let artifact_paths = find_step_artifact_paths(&project_root, &run_id, &current_step.id);
    if artifact_paths.is_empty() {
        miette::bail!("Cannot lock step '{}' with no artifacts.", current_step.id);
    }

    // 6. Write lock sidecars for each artifact
    for path in &artifact_paths {
        write_artifact_lock(path, &run_id, &current_step.id)?;
    }

    log::success(format!(
        "Locked {} artifacts for step '{}'",
        artifact_paths.len(),
        current_step.id
    ))
    .into_diagnostic()?;

    Ok(())
}

/// Resolve the active run and its definition, refusing to advance when the
/// run is an epic parent with mid-flight children or is already complete.
fn load_advance_context(
    project_root: &Path,
) -> Result<(String, PathBuf, PipelineRun, PipelineDefinition)> {
    let run_id = resolve_active_run_id(project_root)?;
    let run_path = run_state_path(project_root, &run_id)?;
    let run = load_run_state(&run_path, &run_id)?;
    let def_path =
        crate::config::pipelines_dir(project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    // Epic coordination (wai-vx02.3): a parent run cannot advance while
    // any recorded child run is mid-flight.
    let midflight = epic_parent_midflight_children(project_root, &run);
    if !midflight.is_empty() {
        miette::bail!(
            "Epic parent run '{}' cannot advance while child runs are mid-flight: {}",
            run.epic.clone().unwrap_or_default(),
            midflight.join(", ")
        );
    }

    if run.current_step >= definition.steps.len() {
        miette::bail!(
            "Pipeline run '{}' is already complete. Start a new run with: wai pipeline start {} --topic=<topic>",
            run_id,
            run.pipeline
        );
    }

    Ok((run_id, run_path, run, definition))
}
