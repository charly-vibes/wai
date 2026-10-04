use std::path::PathBuf;

use crate::commands::pipeline::PipelineDefinition;
use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::context::require_safe_mode;
use crate::json::{PipelineCurrentPayload, PipelineCurrentStep};

use super::definition::{load_pipeline_toml, validate_pipeline};
use super::gates::{evaluate_gates, find_step_artifact_paths, format_gate_summary};
use super::queries::print_step;
use super::{PipelineRun, ValidationLevel, render_prompt, write_artifact_lock};

use crate::commands::require_project;

// ─── start ────────────────────────────────────────────────────────────────────

pub(super) fn cmd_start(name: &str, topic: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline start")?;

    // 1. Find, load and validate the pipeline TOML definition
    let definition = load_validated_pipeline(&project_root, name)?;

    // 2. Generate a unique run ID: <name>-<YYYY-MM-DD>-<topic-slug>
    let run_id = new_run_id(name, topic);
    let topic_str = topic.unwrap_or("");

    // 3-5. Create run state, persist it, and point .last-run at it
    write_run_state(&project_root, &run_id, name, topic_str)?;

    // 6. Print env export line + first step prompt block
    println!("export WAI_PIPELINE_RUN={}", run_id);
    println!();
    print_step(&definition, 0, topic_str);

    Ok(())
}

/// Load `<name>.toml` from the pipelines dir and validate it, failing on
/// validation errors and printing warnings.
fn load_validated_pipeline(project_root: &Path, name: &str) -> Result<PipelineDefinition> {
    let def_path = crate::config::pipelines_dir(project_root).join(format!("{}.toml", name));
    if !def_path.exists() {
        miette::bail!(
            "Pipeline '{}' not found. Create it with: wai pipeline init {}",
            name,
            name
        );
    }
    let definition = load_pipeline_toml(&def_path)?;
    if definition.steps.is_empty() {
        miette::bail!("Pipeline '{}' has no steps defined", name);
    }
    report_validation_issues(&definition, project_root, name)?;
    Ok(definition)
}

/// Fail on validation errors, surface warnings otherwise.
fn report_validation_issues(
    definition: &PipelineDefinition,
    project_root: &Path,
    name: &str,
) -> Result<()> {
    let issues = validate_pipeline(definition, project_root);
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.level == ValidationLevel::Error)
        .collect();
    let warnings: Vec<_> = issues
        .iter()
        .filter(|i| i.level == ValidationLevel::Warn)
        .collect();

    if !errors.is_empty() {
        for e in &errors {
            log::error(&e.message).into_diagnostic()?;
        }
        miette::bail!(
            "Pipeline '{}' has validation errors. Fix them before starting.",
            name
        );
    }
    for w in &warnings {
        log::warning(&w.message).into_diagnostic()?;
    }
    Ok(())
}

/// `<name>-<YYYY-MM-DD>-<topic-slug>` ("run" slug when no topic given).
fn new_run_id(name: &str, topic: Option<&str>) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d");
    let topic_str = topic.unwrap_or("");
    let topic_slug = if topic_str.is_empty() {
        "run".to_string()
    } else {
        slug::slugify(topic_str)
    };
    format!("{}-{}-{}", name, date, topic_slug)
}

/// Create the run state, write it to `.wai/pipeline-runs/<run-id>.yml`, and
/// write the `.last-run` pointer (single source of truth for the active run).
fn write_run_state(project_root: &Path, run_id: &str, name: &str, topic: &str) -> Result<()> {
    let run = PipelineRun {
        run_id: run_id.to_string(),
        pipeline: name.to_string(),
        topic: topic.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        current_step: 0,
        approvals: HashMap::new(),
    };

    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    fs::create_dir_all(&runs_dir).into_diagnostic()?;
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    let last_run = crate::config::last_run_path(project_root);
    fs::create_dir_all(last_run.parent().unwrap()).into_diagnostic()?;
    fs::write(&last_run, run_id).into_diagnostic()?;
    Ok(())
}

// ─── next ─────────────────────────────────────────────────────────────────────

pub(super) fn cmd_next() -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline next")?;

    // 1-3. Resolve run ID, load run state and pipeline definition
    let run_id = resolve_active_run_id(&project_root)?;
    let run_path = run_state_path(&project_root, &run_id)?;
    let run = load_run_state(&run_path, &run_id)?;
    let def_path =
        crate::config::pipelines_dir(&project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    // 4. Check not already complete
    if run.current_step >= definition.steps.len() {
        miette::bail!(
            "Pipeline run '{}' is already complete. Start a new run with: wai pipeline start {} --topic=<topic>",
            run_id,
            run.pipeline
        );
    }

    // 5. Evaluate gates (if configured) before allowing advancement
    let current_step = &definition.steps[run.current_step];
    if let Some(ref gate) = current_step.gate {
        let failures = evaluate_gates(gate, current_step, &run, &definition, &project_root)?;
        if !failures.is_empty() {
            report_gate_failures(&failures, &current_step.id);
            return Ok(());
        }
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

pub(super) fn cmd_approve() -> Result<()> {
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

pub(super) fn cmd_lock() -> Result<()> {
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

    let (step, gate_summary, next_command, message) = if run.current_step >= definition.steps.len()
    {
        (
            None,
            None,
            Some("wai close".to_string()),
            Some(format!(
                "Pipeline '{}' is already complete!",
                definition.name
            )),
        )
    } else {
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
    };

    Ok(Some(PipelineCurrentPayload {
        active: true,
        message,
        pipeline: Some(definition.name),
        run_id: Some(run.run_id),
        topic: Some(run.topic),
        step,
        gate_summary,
        next_command,
    }))
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
    let _ = fs::remove_file(crate::config::last_run_path(project_root));
    name
}

// ─── resolve_active_run_id ────────────────────────────────────────────────────

/// Resolve the active run ID: check `WAI_PIPELINE_RUN` env var first, then
/// fall back to the `.last-run` pointer file at `.wai/resources/pipelines/.last-run`.
pub(super) fn resolve_active_run_id(project_root: &Path) -> Result<String> {
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

// ─── stale-run GC (wai-vx02.2) ────────────────────────────────────────────────

/// A mid-flight pipeline run whose state file looks abandoned.
#[derive(Debug, Clone)]
pub struct StaleRun {
    pub run_id: String,
    pub pipeline: String,
    pub current_step: usize,
    pub total_steps: usize,
    /// Age of the state file in whole days.
    pub age_days: u64,
    /// Path to the run state file.
    pub path: PathBuf,
}

/// Effective stale threshold in days for `project_root` (config knob
/// `pipeline.staleDays`, default 14).
pub fn stale_threshold_days(project_root: &Path) -> u64 {
    crate::config::ProjectConfig::load(project_root)
        .map(|c| c.pipeline_config().effective_stale_days())
        .unwrap_or(crate::config::PipelineConfig::DEFAULT_STALE_DAYS)
}

/// Age of a file in whole days since its mtime. Fails open: unreadable mtime
/// yields 0 (never stale).
///
/// Extracted next to the run-completeness predicate for reuse by doctor,
/// `wai pipeline gc`, and future verify tooling.
pub fn file_age_days(path: &Path) -> u64 {
    let Ok(meta) = fs::metadata(path) else {
        return 0;
    };
    let Ok(modified) = meta.modified() else {
        return 0;
    };
    let Ok(age) = std::time::SystemTime::now().duration_since(modified) else {
        return 0;
    };
    age.as_secs() / 86_400
}

/// True when `run` is mid-flight (`current_step` has not reached the final
/// step of `definition`).
fn run_is_mid_flight(run: &PipelineRun, definition: &PipelineDefinition) -> bool {
    run.current_step < definition.steps.len()
}

/// Find stale mid-flight runs in `.wai/pipeline-runs/`.
///
/// A run is stale when its state-file mtime is older than `threshold_days`
/// AND it is still mid-flight. Runs whose definition can no longer be loaded
/// are conservatively skipped (we can't prove they're mid-flight, and the GC
/// must never destroy state it doesn't understand). Never auto-deletes
/// anything — this is detection only.
pub fn find_stale_runs(project_root: &Path, threshold_days: u64) -> Result<Vec<StaleRun>> {
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    if !runs_dir.is_dir() {
        return Ok(vec![]);
    }

    let mut stale = Vec::new();
    for entry in fs::read_dir(&runs_dir).into_diagnostic()? {
        let path = entry.into_diagnostic()?.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let age_days = file_age_days(&path);
        if age_days <= threshold_days {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(run) = serde_yml::from_str::<PipelineRun>(&content) else {
            continue;
        };
        let def_path =
            crate::config::pipelines_dir(project_root).join(format!("{}.toml", run.pipeline));
        let Ok(definition) = load_pipeline_toml(&def_path) else {
            continue;
        };
        if !run_is_mid_flight(&run, &definition) {
            continue;
        }
        stale.push(StaleRun {
            run_id: run.run_id,
            pipeline: run.pipeline,
            current_step: run.current_step,
            total_steps: definition.steps.len(),
            age_days,
            path,
        });
    }
    Ok(stale)
}

/// Quarantine a stale run: move its state file to `.wai/pipeline-runs/stale/`
/// under its original name with a timestamp suffix (moved, never deleted), and
/// drop the `.last-run` pointer when it references the quarantined run.
fn quarantine_run(project_root: &Path, run: &StaleRun) -> Result<PathBuf> {
    let stale_dir = run
        .path
        .parent()
        .expect("run file has a parent")
        .join("stale");
    fs::create_dir_all(&stale_dir).into_diagnostic()?;
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let dest = stale_dir.join(format!("{}.{}.yml", run.run_id, ts));
    fs::rename(&run.path, &dest).into_diagnostic()?;

    let last_run = crate::config::last_run_path(project_root);
    if let Ok(pointer) = fs::read_to_string(&last_run)
        && pointer.trim() == run.run_id
    {
        let _ = fs::remove_file(&last_run);
    }
    Ok(dest)
}

/// `wai pipeline gc` — detect and quarantine abandoned mid-flight runs.
///
/// Default is a dry run listing what would be quarantined; `--yes` executes.
/// Deterministic and non-interactive (AFK).
pub fn cmd_gc(yes: bool) -> Result<()> {
    let project_root = require_project()?;
    let threshold = stale_threshold_days(&project_root);
    let stale = find_stale_runs(&project_root, threshold)?;

    if stale.is_empty() {
        println!("No stale runs to quarantine (threshold: {threshold} days).");
        return Ok(());
    }

    for run in &stale {
        println!(
            "stale run '{}' — pipeline '{}', step {}/{}, {}d old",
            run.run_id, run.pipeline, run.current_step, run.total_steps, run.age_days
        );
    }

    if !yes {
        println!(
            "Dry run: {} run(s) would be quarantined under .wai/pipeline-runs/stale/.",
            stale.len()
        );
        println!("Run `wai pipeline gc --yes` to quarantine them.");
        return Ok(());
    }

    for run in &stale {
        let dest = quarantine_run(&project_root, run)?;
        println!("✓ Quarantined '{}' → {}", run.run_id, dest.display());
    }
    Ok(())
}
