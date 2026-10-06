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

pub(super) fn cmd_start(name: &str, topic: Option<&str>, epic: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline start")?;

    // 1. Find, load and validate the pipeline TOML definition
    let definition = load_validated_pipeline(&project_root, name)?;

    if let Some(epic_id) = epic {
        return cmd_start_epic(&project_root, &definition, name, topic, epic_id);
    }

    // 2. Generate a unique run ID: <name>-<YYYY-MM-DD>-<topic-slug>
    let run_id = new_run_id(name, topic);
    let topic_str = topic.unwrap_or("");

    // 3-5. Create run state, persist it, and point .last-run at it
    write_run_state(&project_root, &run_id, name, topic_str, None, &[])?;

    // 5b. Epic coordination (wai-vx02.3): when the topic matches a ready child
    // issue of an epic parent run, record this run on the parent's state.
    record_child_run_on_epic_parent(&project_root, &run_id, topic_str)?;

    // 6. Print env export line + first step prompt block
    println!("export WAI_PIPELINE_RUN={}", run_id);
    println!();
    print_step(&definition, 0, topic_str);

    Ok(())
}

/// Epic coordination (wai-vx02.3): `pipeline start <pipeline> --epic=<id>`
/// discovers ready children via `bd ready --json` (parent filter) and creates
/// a parent run (one per epic, per repo) that coordinates child runs.
fn cmd_start_epic(
    project_root: &Path,
    definition: &PipelineDefinition,
    name: &str,
    topic: Option<&str>,
    epic_id: &str,
) -> Result<()> {
    // Idempotent: reuse an existing parent run for this epic instead of
    // resetting its state (a mid-flight parent run must survive re-starts).
    // Checked before discovery: an epic with a mid-flight parent but nothing
    // ready right now still reports the active parent, not "no ready children".
    if let Some(existing) = find_epic_parent_run(project_root, name, epic_id)? {
        println!("Epic parent run already active: {}", existing);
        println!("export WAI_PIPELINE_RUN={}", existing);
        return Ok(());
    }

    let children = discover_ready_children(project_root, epic_id)?;
    if children.is_empty() {
        println!(
            "Epic '{}' has no ready children — no parent run created.",
            epic_id
        );
        println!("Nothing to coordinate: all children are closed or picked up elsewhere.");
        return Ok(());
    }

    // Parent run id: <name>-<date>-<topic-or-epic-slug>-parent.
    let topic_str = topic.unwrap_or(epic_id);
    let run_id = format!("{}-parent", new_run_id(name, topic));
    write_run_state(
        project_root,
        &run_id,
        name,
        topic_str,
        Some(epic_id),
        &children,
    )?;

    println!("Epic parent run created: {}", run_id);
    println!("Ready children of '{}':", epic_id);
    for child in &children {
        println!(
            "  - {} (start its run with: wai pipeline start {} --topic={})",
            child, name, child
        );
    }
    println!("export WAI_PIPELINE_RUN={}", run_id);
    println!();
    print_step(definition, 0, topic_str);

    Ok(())
}

/// Invoke `bd ready --json` and return the ids of issues whose `parent`
/// equals `epic_id` (wai-vx02.3 parent filter). Contract assumption: beads
/// issue JSON carries `id` and `parent` string fields.
fn discover_ready_children(project_root: &Path, epic_id: &str) -> Result<Vec<String>> {
    let output = std::process::Command::new("bd")
        .args(["ready", "--json"])
        .current_dir(project_root)
        .output()
        .map_err(|e| miette::miette!("Epic start requires bd: `bd ready --json` failed: {}", e))?;
    if !output.status.success() {
        miette::bail!("Epic start requires bd: `bd ready --json` exited with failure");
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| miette::miette!("Failed to parse `bd ready --json` output: {}", e))?;
    let Some(arr) = json.as_array() else {
        miette::bail!("Unexpected `bd ready --json` output: expected a JSON array");
    };
    Ok(arr
        .iter()
        .filter_map(|item| {
            if item.get("parent")?.as_str()? == epic_id {
                item.get("id")?.as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect())
}

/// Find an existing epic parent run for `epic_id` by scanning run state files
/// for `epic: <id>`. Returns the parent run id, if any.
fn find_epic_parent_run(
    project_root: &Path,
    pipeline_name: &str,
    epic_id: &str,
) -> Result<Option<String>> {
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    if !runs_dir.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(&runs_dir).into_diagnostic()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yml") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(run) = serde_yml::from_str::<PipelineRun>(&text)
            && run.epic.as_deref() == Some(epic_id)
            && run.pipeline == pipeline_name
        {
            return Ok(Some(run.run_id));
        }
    }
    Ok(None)
}

/// Append a freshly started run to the `child_runs` of every epic parent run
/// whose `child_issues` contain the run's topic (wai-vx02.3).
fn record_child_run_on_epic_parent(project_root: &Path, run_id: &str, topic: &str) -> Result<()> {
    if topic.is_empty() {
        return Ok(());
    }
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    if !runs_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&runs_dir).into_diagnostic()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yml") {
            continue;
        }
        if path.file_stem().and_then(|x| x.to_str()) == Some(run_id) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(mut parent) = serde_yml::from_str::<PipelineRun>(&text) else {
            continue;
        };
        if parent.epic.is_none() || !parent.child_issues.iter().any(|i| i == topic) {
            continue;
        }
        if !parent.child_runs.iter().any(|r| r == run_id) {
            parent.child_runs.push(run_id.to_string());
            let yaml = serde_yml::to_string(&parent)
                .map_err(|e| miette::miette!("Failed to serialize parent run state: {}", e))?;
            fs::write(&path, yaml).into_diagnostic()?;
        }
    }
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
fn write_run_state(
    project_root: &Path,
    run_id: &str,
    name: &str,
    topic: &str,
    epic: Option<&str>,
    child_issues: &[String],
) -> Result<()> {
    let run = PipelineRun {
        run_id: run_id.to_string(),
        pipeline: name.to_string(),
        topic: topic.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        current_step: 0,
        approvals: HashMap::new(),
        epic: epic.map(|e| e.to_string()),
        child_issues: child_issues.to_vec(),
        child_runs: Vec::new(),
        handoff_artifact: None,
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

    // 3b. Epic coordination (wai-vx02.3): a parent run cannot advance while
    // any recorded child run is mid-flight.
    let midflight = epic_parent_midflight_children(&project_root, &run);
    if !midflight.is_empty() {
        miette::bail!(
            "Epic parent run '{}' cannot advance while child runs are mid-flight: {}",
            run.epic.clone().unwrap_or_default(),
            midflight.join(", ")
        );
    }

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
fn epic_parent_midflight_children(project_root: &Path, run: &PipelineRun) -> Vec<String> {
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
