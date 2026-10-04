// Pipeline-related doctor checks: pipeline definitions, artifact locks,
// pipeline utilization, and dont-drift signals.
//
// Decomposed from the original monolithic check functions (tidy-first,
// no behavior change).

use genesis::doctor::CheckStatus;

use super::WaiCheckEntry;
use super::checks_session::read_non_managed_block_content;
use crate::commands::pipeline::find_stale_runs;
use crate::config::projects_dir;
use crate::workspace::detect_installed_pipelines;
use std::path::Path;

/// Doctor check: flag mid-flight runs whose state file mtime exceeds the
/// stale threshold (pipeline.staleDays, default 14). Silent when green;
/// suggests `wai pipeline gc --yes` (never auto-deletes — GC is explicit).
pub(super) fn check_stale_pipeline_runs(project_root: &Path) -> Vec<WaiCheckEntry> {
    let threshold = crate::commands::pipeline::stale_threshold_days(project_root);
    let Ok(runs) = find_stale_runs(project_root, threshold) else {
        return vec![]; // fail-open: detection must never block the doctor
    };
    if runs.is_empty() {
        return vec![];
    }
    let listed = runs
        .iter()
        .map(|r| {
            format!(
                "'{}' (pipeline '{}', step {}/{}, {}d old)",
                r.run_id, r.pipeline, r.current_step, r.total_steps, r.age_days
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    vec![WaiCheckEntry {
        name: "Pipeline: stale-runs".to_string(),
        status: CheckStatus::Warn,
        message: format!(
            "{} abandoned mid-flight run(s): {listed} — quarantine with `wai pipeline gc --yes`",
            runs.len()
        ),
        fix: None,
        fix_fn: None,
    }]
}

/// Build a pipeline-check entry with the standard name prefix.
fn pipeline_entry(file_stem: &str, status: CheckStatus, message: String) -> WaiCheckEntry {
    WaiCheckEntry {
        name: format!("Pipeline: {file_stem}"),
        status,
        message,
        fix: None,
        fix_fn: None,
    }
}

/// Validate pipeline TOML definitions for correctness.
pub(super) fn check_pipeline_definitions(project_root: &Path) -> Vec<WaiCheckEntry> {
    use crate::config::pipelines_dir;

    let pipelines = pipelines_dir(project_root);
    if !pipelines.exists() {
        return vec![];
    }
    let Ok(entries) = std::fs::read_dir(&pipelines) else {
        return vec![];
    };

    let oracles_dir = crate::config::wai_dir(project_root)
        .join("resources")
        .join("oracles");

    let mut names_seen: Vec<(String, String)> = Vec::new(); // (name, file)
    let mut results = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        check_pipeline_file(
            &path,
            &file_stem,
            &oracles_dir,
            &mut names_seen,
            &mut results,
        );
    }

    results
}

/// Validate a single pipeline TOML file, appending findings to `results`.
fn check_pipeline_file(
    path: &Path,
    file_stem: &str,
    oracles_dir: &Path,
    names_seen: &mut Vec<(String, String)>,
    results: &mut Vec<WaiCheckEntry>,
) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            results.push(pipeline_entry(
                file_stem,
                CheckStatus::Fail,
                format!("Cannot read: {e}"),
            ));
            return;
        }
    };
    let parsed: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            results.push(pipeline_entry(
                file_stem,
                CheckStatus::Fail,
                format!("Invalid TOML: {e}"),
            ));
            return;
        }
    };
    let pipeline_name = parsed
        .get("pipeline")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");
    if pipeline_name.is_empty() {
        results.push(pipeline_entry(
            file_stem,
            CheckStatus::Fail,
            "Missing [pipeline].name".to_string(),
        ));
        return;
    }
    check_duplicate_name(pipeline_name, file_stem, names_seen, results);
    check_pipeline_metadata(&parsed, pipeline_name, file_stem, results);
    check_step_oracles(&parsed, file_stem, oracles_dir, results);
}

/// Fail on a pipeline name already defined in another file.
fn check_duplicate_name(
    pipeline_name: &str,
    file_stem: &str,
    names_seen: &mut Vec<(String, String)>,
    results: &mut Vec<WaiCheckEntry>,
) {
    if let Some((_, prev_file)) = names_seen.iter().find(|(n, _)| n == pipeline_name) {
        results.push(pipeline_entry(
            file_stem,
            CheckStatus::Fail,
            format!("Duplicate pipeline name '{pipeline_name}' (also in {prev_file})"),
        ));
    } else {
        names_seen.push((pipeline_name.to_string(), file_stem.to_string()));
    }
}

/// Warn when `[pipeline.metadata]` is absent (pipeline won't appear in the
/// managed block).
fn check_pipeline_metadata(
    parsed: &toml::Value,
    pipeline_name: &str,
    file_stem: &str,
    results: &mut Vec<WaiCheckEntry>,
) {
    let has_metadata = parsed
        .get("pipeline")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("when"))
        .and_then(|w| w.as_str())
        .is_some();
    if !has_metadata {
        results.push(pipeline_entry(
            file_stem,
            CheckStatus::Warn,
            format!(
                "Missing [pipeline.metadata] — pipeline '{pipeline_name}' won't appear in managed block"
            ),
        ));
    }
}

/// Verify every gate oracle referenced by the pipeline's steps resolves to an
/// executable file in the oracles directory.
fn check_step_oracles(
    parsed: &toml::Value,
    file_stem: &str,
    oracles_dir: &Path,
    results: &mut Vec<WaiCheckEntry>,
) {
    let Some(steps) = parsed.get("steps").and_then(|s| s.as_array()) else {
        return;
    };
    for step in steps {
        let Some(gate) = step.get("gate") else {
            continue;
        };
        let Some(oracles) = gate.get("oracles").and_then(|o| o.as_array()) else {
            continue;
        };
        for oracle in oracles {
            check_single_oracle(oracle, file_stem, oracles_dir, results);
        }
    }
}

/// Check one oracle reference: must be named and resolve to an executable
/// file (with an optional `.sh`/`.py` extension) in the oracles directory.
fn check_single_oracle(
    oracle: &toml::Value,
    file_stem: &str,
    oracles_dir: &Path,
    results: &mut Vec<WaiCheckEntry>,
) {
    if oracle.get("command").is_some() {
        return;
    }
    let Some(name) = oracle.get("name").and_then(|n| n.as_str()) else {
        return;
    };
    let oracle_path = find_oracle_file(name, oracles_dir);
    let Some(path) = oracle_path else {
        results.push(pipeline_entry(
            file_stem,
            CheckStatus::Warn,
            format!("Gate oracle '{name}' — command not found"),
        ));
        return;
    };
    if !oracle_file_executable(&path) {
        results.push(pipeline_entry(
            file_stem,
            CheckStatus::Warn,
            format!("Gate oracle '{name}' — not executable"),
        ));
    }
}

/// Find an oracle file by name, trying bare, `.sh`, and `.py` extensions.
fn find_oracle_file(name: &str, oracles_dir: &Path) -> Option<std::path::PathBuf> {
    ["", ".sh", ".py"]
        .iter()
        .map(|ext| oracles_dir.join(format!("{name}{ext}")))
        .find(|p| p.exists())
}

/// Whether an existing oracle file is executable (unix permission bits; on
/// non-unix, existence is enough).
fn oracle_file_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|meta| meta.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Verify artifact locks across all projects: every `.lock` sidecar must be
/// readable and match its artifact's current content hash.
pub(super) fn check_artifact_locks(project_root: &Path) -> Vec<WaiCheckEntry> {
    use walkdir::WalkDir;

    let projects = projects_dir(project_root);
    if !projects.exists() {
        return vec![];
    }

    let lock_files: Vec<std::path::PathBuf> = WalkDir::new(&projects)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().and_then(|ext| ext.to_str()) == Some("lock")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if lock_files.is_empty() {
        return vec![];
    }

    let mut verified = 0usize;
    let mut mismatches = Vec::new();
    for lock_path in &lock_files {
        match verify_artifact_lock(project_root, lock_path) {
            Ok(()) => verified += 1,
            Err(message) => mismatches.push(artifact_lock_warn(message)),
        }
    }

    if mismatches.is_empty() {
        vec![WaiCheckEntry {
            name: "Artifact locks".to_string(),
            status: CheckStatus::Pass,
            message: format!("All {verified} locked artifacts verified"),
            fix: None,
            fix_fn: None,
        }]
    } else {
        mismatches
    }
}

/// Verify one artifact lock: lock readable, artifact present, hash matches.
/// On failure, returns the warning message text.
fn verify_artifact_lock(project_root: &Path, lock_path: &Path) -> Result<(), String> {
    use super::super::pipeline::{artifact_hash, read_artifact_lock};

    let lock = read_artifact_lock(lock_path).map_err(|_| {
        format!(
            "Cannot read lock file: {}",
            lock_path
                .strip_prefix(project_root)
                .unwrap_or(lock_path)
                .display()
        )
    })?;

    let artifact_path = lock_path.parent().unwrap().join(&lock.artifact);
    if !artifact_path.exists() {
        return Err(format!(
            "Locked artifact missing: {}",
            artifact_path
                .strip_prefix(project_root)
                .unwrap_or(&artifact_path)
                .display()
        ));
    }

    let current_hash = artifact_hash(&artifact_path).map_err(|_| {
        format!(
            "Cannot hash artifact: {}",
            artifact_path
                .strip_prefix(project_root)
                .unwrap_or(&artifact_path)
                .display()
        )
    })?;
    if current_hash != lock.lock_hash {
        return Err(format!(
            "Hash mismatch: {} (run {})",
            lock.artifact, lock.pipeline_run
        ));
    }
    Ok(())
}

/// Build an artifact-lock warning entry for the given message.
fn artifact_lock_warn(message: String) -> WaiCheckEntry {
    WaiCheckEntry {
        name: "Artifact lock".to_string(),
        status: CheckStatus::Warn,
        message,
        fix: None,
        fix_fn: None,
    }
}

/// Check for agent-workflow vs pipeline mismatch.
///
/// Detects when AGENTS.md or CLAUDE.md contain manual-cycle instructions
/// (e.g. "TDD → ro5u → fix → commit → next ticket") but an equivalent
/// pipeline (with  mentioning TDD, autonomous, or ro5) is
/// available. Surfaces a warning so the agent prefers the pipeline.
pub(super) fn check_pipeline_utilization(project_root: &Path) -> Vec<WaiCheckEntry> {
    let pipelines = detect_installed_pipelines(project_root);

    // Which pipelines have TDD/autonomous/Ro5 in their  metadata?
    let tdd_pipelines: Vec<&crate::managed_block::InstalledPipeline> = pipelines
        .iter()
        .filter(|p| {
            let w = p.when.to_lowercase();
            w.contains("tdd") || w.contains("ro5") || w.contains("autonomous")
        })
        .collect();

    if tdd_pipelines.is_empty() {
        return vec![];
    }

    let agents_path = project_root.join("AGENTS.md");
    let claude_path = project_root.join("CLAUDE.md");
    let combined = format!(
        "{}\n{}",
        read_non_managed_block_content(&agents_path),
        read_non_managed_block_content(&claude_path)
    );

    let matches = manual_cycle_matches(&combined);
    if matches.is_empty() {
        return vec![];
    }

    let pipeline_names: Vec<&str> = tdd_pipelines.iter().map(|p| p.name.as_str()).collect();

    vec![WaiCheckEntry {
        name: "Pipeline utilization".to_string(),
        status: CheckStatus::Warn,
        message: format!(
            "Pipeline(s) '{}' encode the TDD+Ro5 cycle, but AGENTS.md/CLAUDE.md contain manual-cycle instructions: {}",
            pipeline_names.join(", "),
            matches.join("; ")
        ),
        fix: Some(format!(
            "Update the Quick Start in AGENTS.md to prefer 'wai pipeline start {} --topic=<...>' over the manual cycle",
            pipeline_names[0]
        )),
        fix_fn: None,
    }]
}

/// Patterns that indicate manual-cycle instructions, matched (case
///-insensitively, ignoring spaces) against agent-file content.
fn manual_cycle_matches(content: &str) -> Vec<&'static str> {
    let manual_patterns = [
        "Per-ticket pipeline",
        "TDD → ro5u",
        "TDD.*ro5.*fix.*commit",
        "follow ",
    ];
    let content_lower = content.to_lowercase();
    manual_patterns
        .into_iter()
        .filter(|pattern| {
            let pat_lower = pattern.to_lowercase();
            content_lower.contains(&pat_lower)
                || content_lower.contains(&pattern.replace(" ", "").to_lowercase())
        })
        .collect()
}

pub(super) fn check_dont_drift_signals(project_root: &Path) -> Vec<WaiCheckEntry> {
    if std::env::var("WAI_DONT_SIGNALS").is_err() {
        return vec![];
    }
    let output = std::process::Command::new("ah")
        .arg("signals")
        .current_dir(project_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();
    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };
    drift_signals_to_check_results(&output.stdout)
}

/// Parse the JSON output of `ah signals` and return check results.
/// Extracted for unit testing without a subprocess dependency.
pub(super) fn drift_signals_to_check_results(json_bytes: &[u8]) -> Vec<WaiCheckEntry> {
    let signals: Vec<serde_json::Value> = match serde_json::from_slice(json_bytes) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    if signals.is_empty() {
        return vec![WaiCheckEntry {
            name: "dont drift signals".to_string(),
            status: CheckStatus::Pass,
            message: "No dont rejection signals detected".to_string(),
            fix: None,
            fix_fn: None,
        }];
    }
    let count = signals.len();
    let rules: Vec<&str> = signals
        .iter()
        .filter_map(|s| s["rule_name"].as_str())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    vec![WaiCheckEntry {
        name: "dont drift signals".to_string(),
        status: CheckStatus::Warn,
        message: format!(
            "{count} dont rejection signal(s) detected (rules: {}); run `ah signals` for details",
            rules.join(", ")
        ),
        fix: Some("Run: dont doctor --json to inspect violations".to_string()),
        fix_fn: None,
    }]
}
