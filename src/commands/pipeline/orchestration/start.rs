use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cliclack::log;
use miette::{IntoDiagnostic, Result};

use crate::commands::pipeline::{
    PipelineDefinition, PipelineRun, ValidationLevel, load_pipeline_toml,
};
use crate::commands::require_project;
use crate::context::require_safe_mode;

use crate::commands::pipeline::definition::validate_pipeline;
use crate::commands::pipeline::queries::print_step;

// ─── start ────────────────────────────────────────────────────────────────────

pub(crate) fn cmd_start(name: &str, topic: Option<&str>, epic: Option<&str>) -> Result<()> {
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
        bypasses: Vec::new(),
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
