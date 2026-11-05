use std::fs;
use std::path::Path;
use std::path::PathBuf;

use miette::{IntoDiagnostic, Result};

use crate::commands::pipeline::{PipelineDefinition, PipelineRun, load_pipeline_toml};
use crate::commands::require_project;

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
