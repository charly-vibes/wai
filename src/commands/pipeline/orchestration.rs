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

    // 1. Find and load the pipeline TOML definition
    let def_path = crate::config::pipelines_dir(&project_root).join(format!("{}.toml", name));
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

    // 1b. Validate the pipeline definition
    let issues = validate_pipeline(&definition, &project_root);
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

    // 2. Generate a unique run ID: <name>-<YYYY-MM-DD>-<topic-slug>
    let date = chrono::Utc::now().format("%Y-%m-%d");
    let topic_str = topic.unwrap_or("");
    let topic_slug = if topic_str.is_empty() {
        "run".to_string()
    } else {
        slug::slugify(topic_str)
    };
    let run_id = format!("{}-{}-{}", name, date, topic_slug);

    // 3. Create run state
    let run = PipelineRun {
        run_id: run_id.clone(),
        pipeline: name.to_string(),
        topic: topic_str.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        current_step: 0,
        approvals: HashMap::new(),
    };

    // 4. Write run state to .wai/pipeline-runs/<run-id>.yml
    let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
    fs::create_dir_all(&runs_dir).into_diagnostic()?;
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    // 5. Write .last-run pointer file
    let last_run = crate::config::last_run_path(&project_root);
    fs::create_dir_all(last_run.parent().unwrap()).into_diagnostic()?;
    fs::write(&last_run, &run_id).into_diagnostic()?;

    // 6. Write .wai/.pipeline-run so `wai add` picks up the run ID automatically
    crate::config::write_pipeline_run_state(&project_root, &run_id)
        .map_err(|e| miette::miette!("Failed to write pipeline run state: {}", e))?;

    // 7. Print env export line + first step prompt block
    println!("export WAI_PIPELINE_RUN={}", run_id);
    println!();
    print_step(&definition, 0, topic_str);

    Ok(())
}

// ─── next ─────────────────────────────────────────────────────────────────────

pub(super) fn cmd_next() -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline next")?;

    // 1. Resolve run ID (env var → .last-run)
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
            println!();
            println!(
                "  {} Gate check failed for step '{}':",
                "✗".red(),
                current_step.id
            );
            println!();
            for f in &failures {
                println!("    {} {}", "✗".red(), f);
            }
            println!();
            println!(
                "  {} Resolve the above before running `wai pipeline next`",
                "→".cyan()
            );
            return Ok(());
        }
    }

    // 5b. Lock artifacts if step has lock = true
    if current_step.lock {
        let artifact_paths = find_step_artifact_paths(&project_root, &run.run_id, &current_step.id);
        if artifact_paths.is_empty() {
            miette::bail!("Cannot lock step '{}' with no artifacts.", current_step.id);
        }
        for path in &artifact_paths {
            write_artifact_lock(path, &run.run_id, &current_step.id)?;
        }
        log::info(format!(
            "Locked {} artifact(s) for step '{}'",
            artifact_paths.len(),
            current_step.id
        ))
        .into_diagnostic()?;
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
        println!("──────────────────────────────────────────────");
        println!("Pipeline '{}' complete!", definition.name);
        println!();
        println!("Next: wai close");
        println!("      wai pipeline suggest   # start another pipeline");
    } else {
        print_step(&definition, next_step, &updated.topic);
    }

    Ok(())
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
    miette::bail!(
        "No active pipeline run. Start one with: wai pipeline start <name> --topic=<topic>"
    )
}
