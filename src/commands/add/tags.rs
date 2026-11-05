//! Pipeline provenance tagging for `wai add`: run/step tags, lock detection.

use miette::{IntoDiagnostic, Result};
use std::path::Path;

/// Build the final tags list: user-supplied tags merged with the auto-injected
/// `pipeline-run:<id>` tag when an active pipeline run can be resolved.
///
/// Resolution order (first non-empty value wins):
///   1. `WAI_PIPELINE_RUN` environment variable (backwards-compatible).
///   2. `.wai/resources/pipelines/.last-run` pointer file (written by `wai pipeline start`).
pub(super) fn build_tags(user_tags: Option<&str>, project_root: &Path) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();

    if let Some(t) = user_tags {
        tags.extend(
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        );
    }

    // Resolve active pipeline run: the env var is explicit intent — stamp
    // unconditionally. The `.last-run` pointer is a heuristic (it records the
    // last *started* run, not a verified active one) — stamp only when the
    // run state confirms the run is still in progress.
    let active_run = std::env::var("WAI_PIPELINE_RUN")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| active_pointer_run(project_root));

    if let Some(ref run_id) = active_run {
        tags.push(format!("pipeline-run:{}", run_id));

        // Also inject pipeline-step:<step-id> from the run state + definition.
        if let Some(step_id) = resolve_current_step_id(project_root, run_id) {
            tags.push(format!("pipeline-step:{}", step_id));
        }
    }

    tags
}

/// Resolve the `.last-run` pointer to a run that is verifiably in progress,
/// fail-closed.
///
/// The pointer records the last *started* run, and only `wai close` clears it
/// — a run that completes without `wai close` keeps its pointer, and stamping
/// from the pointer alone produced stale `pipeline-run:` provenance on
/// unrelated artifacts (reproduced 2026-10-14: a completed
/// epic-orchestrator run tagged a new artifact). The run id is returned only
/// when its state file exists, parses, and has not reached its final step.
/// A run proven complete gets its stale pointer removed best-effort (the
/// same clearing `wai close` would perform); an unverifiable run only
/// suppresses the tag and leaves the pointer untouched.
fn active_pointer_run(project_root: &Path) -> Option<String> {
    let run_id = crate::config::read_pipeline_run_state(project_root)?;
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    let run: crate::commands::pipeline::PipelineRun =
        serde_yml::from_str(&std::fs::read_to_string(&run_path).ok()?).ok()?;
    let def_path =
        crate::config::pipelines_dir(project_root).join(format!("{}.toml", run.pipeline));
    let definition = crate::commands::pipeline::load_pipeline_toml(&def_path).ok()?;
    if run.current_step >= definition.steps.len() {
        // Complete: the pointer outlived the run — self-heal it.
        let _ = std::fs::remove_file(crate::config::last_run_path(project_root));
        return None;
    }
    Some(run_id)
}

/// Resolve the current step ID by reading the pipeline run state and definition.
///
/// Returns `None` gracefully if anything fails (missing files, parse errors,
/// step index out of bounds), so it never breaks artifact creation.
fn resolve_current_step_id(project_root: &Path, run_id: &str) -> Option<String> {
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    let content = std::fs::read_to_string(&run_path).ok()?;
    let run: crate::commands::pipeline::PipelineRun = serde_yml::from_str(&content).ok()?;

    let def_path =
        crate::config::pipelines_dir(project_root).join(format!("{}.toml", run.pipeline));
    let definition = crate::commands::pipeline::load_pipeline_toml(&def_path).ok()?;

    definition.steps.get(run.current_step).map(|s| s.id.clone())
}

/// Check if an artifact has any associated .lock sidecar files.
///
/// Lock files follow the naming convention `<artifact-filename>.<run-id>.lock`
/// in the same directory as the artifact. Returns `true` if at least one such
/// file exists.
pub(super) fn has_lock_file(artifact_path: &str) -> bool {
    let path = Path::new(artifact_path);
    let dir = match path.parent() {
        Some(d) => d,
        None => return false,
    };
    let filename = match path.file_name() {
        Some(f) => f.to_string_lossy(),
        None => return false,
    };
    let prefix = format!("{}.", filename);
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) && name.ends_with(".lock") {
                return true;
            }
        }
    }
    false
}

/// Warn when `--corrects` targets an artifact whose pipeline step is not locked.
///
/// This is informational only — the addendum is still created regardless.
pub(super) fn warn_if_unlocked(corrects_path: &str, step_id: &str) -> Result<()> {
    if !has_lock_file(corrects_path) && !crate::context::current_context().quiet {
        cliclack::log::warning(format!(
            "Step '{}' is not locked — consider editing the original artifact directly.",
            step_id
        ))
        .into_diagnostic()?;
    }
    Ok(())
}

/// Extract `pipeline-step:<id>` tag from an artifact's YAML frontmatter.
///
/// Returns `None` gracefully if the file cannot be read, has no frontmatter,
/// or contains no `pipeline-step:` tag.
pub(super) fn resolve_pipeline_step_from_artifact(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    if !content.starts_with("---\n") {
        return None;
    }
    let end = content[4..].find("---\n")?;
    let fm = &content[4..4 + end];
    for line in fm.lines() {
        if line.starts_with("tags:") {
            for tag in line
                .trim_start_matches("tags:")
                .trim()
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
            {
                let tag = tag.trim();
                if let Some(step) = tag.strip_prefix("pipeline-step:") {
                    return Some(step.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_pipeline_fixture(root: &Path, current_step: u32) {
        let wai = root.join(".wai");
        std::fs::create_dir_all(wai.join("pipeline-runs")).unwrap();
        std::fs::create_dir_all(wai.join("resources/pipelines")).unwrap();

        let toml_content = r#"[pipeline]
name = "test-pipe"

[[steps]]
id = "research"
prompt = "Do research on {topic}."

[[steps]]
id = "implement"
prompt = "Implement {topic}."
"#;
        std::fs::write(wai.join("resources/pipelines/test-pipe.toml"), toml_content).unwrap();

        let run_yaml = format!(
            r#"run_id: "test-pipe-2026-04-02-qcd"
pipeline: "test-pipe"
topic: "qcd"
created_at: "2026-04-02T00:00:00Z"
current_step: {}
"#,
            current_step
        );
        std::fs::write(
            wai.join("pipeline-runs/test-pipe-2026-04-02-qcd.yml"),
            run_yaml,
        )
        .unwrap();
        std::fs::write(
            wai.join("resources/pipelines/.last-run"),
            "test-pipe-2026-04-02-qcd",
        )
        .unwrap();
    }

    #[test]
    fn step_tag_injected_when_pipeline_run_active() {
        let dir = tempfile::tempdir().unwrap();
        write_pipeline_fixture(dir.path(), 0);

        let tags = build_tags(None, dir.path());
        assert!(
            tags.contains(&"pipeline-run:test-pipe-2026-04-02-qcd".to_string()),
            "expected pipeline-run tag, got: {:?}",
            tags
        );
        assert!(
            tags.contains(&"pipeline-step:research".to_string()),
            "expected pipeline-step:research tag, got: {:?}",
            tags
        );
    }

    #[test]
    fn step_tag_not_injected_when_no_run() {
        let dir = tempfile::tempdir().unwrap();
        let tags = build_tags(Some("custom"), dir.path());
        assert_eq!(tags, vec!["custom".to_string()]);
        assert!(!tags.iter().any(|t| t.starts_with("pipeline-step:")));
    }

    /// Regression (reproduced 2026-10-14, wild-00r): `.last-run` records the
    /// last *started* run, and only `wai close` clears it — a completed run's
    /// pointer survived and `wai add` stamped its stale id on a new artifact.
    /// A pointer-resolved run must have state proving it is still in progress.
    #[test]
    fn stale_pointer_to_completed_run_not_stamped() {
        let dir = tempfile::tempdir().unwrap();
        write_pipeline_fixture(dir.path(), 2);

        let tags = build_tags(None, dir.path());
        assert!(
            !tags.iter().any(|t| t.starts_with("pipeline-run:")),
            "stale completed run must not be stamped as provenance, got: {:?}",
            tags
        );
        assert!(!tags.iter().any(|t| t.starts_with("pipeline-step:")));
    }

    #[test]
    fn stale_pointer_cleared_when_run_complete() {
        let dir = tempfile::tempdir().unwrap();
        write_pipeline_fixture(dir.path(), 2);

        let _ = build_tags(None, dir.path());
        let pointer = dir.path().join(".wai/resources/pipelines/.last-run");
        assert!(
            !pointer.exists(),
            "completed run's stale pointer must be cleared"
        );
    }

    /// A pointer naming a run with no (or unreadable) state file cannot be
    /// verified active — suppress the tag (fail closed) but leave the pointer
    /// alone: removal is reserved for runs proven complete.
    #[test]
    fn pointer_to_missing_run_state_not_stamped_and_pointer_kept() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let wai = root.join(".wai");
        std::fs::create_dir_all(wai.join("resources/pipelines")).unwrap();
        std::fs::write(wai.join("resources/pipelines/.last-run"), "ghost-run").unwrap();

        let tags = build_tags(None, root);
        assert!(
            !tags.iter().any(|t| t.starts_with("pipeline-run:")),
            "unverifiable run must not be stamped, got: {:?}",
            tags
        );
        assert!(
            wai.join("resources/pipelines/.last-run").exists(),
            "pointer must be kept when the run is not proven complete"
        );
    }

    #[test]
    fn resolve_pipeline_step_from_artifact_parses_tag() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("original.md");
        let content =
            "---\ntags: [pipeline-run:test-run, pipeline-step:research]\n---\n\noriginal content\n";
        std::fs::write(&artifact, content).unwrap();

        let result = resolve_pipeline_step_from_artifact(artifact.to_str().unwrap());
        assert_eq!(result, Some("research".to_string()));
    }

    #[test]
    fn resolve_pipeline_step_returns_none_without_step_tag() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("no-step.md");
        let content = "---\ntags: [some-tag]\n---\n\ncontent\n";
        std::fs::write(&artifact, content).unwrap();

        let result = resolve_pipeline_step_from_artifact(artifact.to_str().unwrap());
        assert_eq!(result, None);
    }

    #[test]
    fn resolve_pipeline_step_returns_none_without_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("no-fm.md");
        std::fs::write(&artifact, "just plain content\n").unwrap();

        let result = resolve_pipeline_step_from_artifact(artifact.to_str().unwrap());
        assert_eq!(result, None);
    }

    #[test]
    fn resolve_pipeline_step_returns_none_for_missing_file() {
        let result = resolve_pipeline_step_from_artifact("/nonexistent/path.md");
        assert_eq!(result, None);
    }

    #[test]
    fn corrects_auto_adds_pipeline_addendum_tag() {
        let dir = tempfile::tempdir().unwrap();
        // Create the corrected artifact with a pipeline-step tag
        let original = dir.path().join("original.md");
        let original_content =
            "---\ntags: [pipeline-run:run1, pipeline-step:implement]\n---\n\noriginal\n";
        std::fs::write(&original, original_content).unwrap();

        let corrects_path = original.to_str().unwrap();
        let mut all_tags: Vec<String> = vec!["custom".to_string()];
        // Replicate the corrects logic from the command handler
        if let Some(step_id) = resolve_pipeline_step_from_artifact(corrects_path) {
            let addendum_tag = format!("pipeline-addendum:{}", step_id);
            if !all_tags.contains(&addendum_tag) {
                all_tags.push(addendum_tag);
            }
        }

        assert!(
            all_tags.contains(&"pipeline-addendum:implement".to_string()),
            "expected pipeline-addendum:implement tag, got: {:?}",
            all_tags
        );
        assert!(
            all_tags.contains(&"custom".to_string()),
            "user tag should be preserved"
        );
    }

    #[test]
    fn has_lock_file_returns_true_when_lock_exists() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("2026-04-01-research.md");
        std::fs::write(&artifact, "content").unwrap();
        // Create a lock sidecar
        let lock_file = dir.path().join("2026-04-01-research.md.run-abc.lock");
        std::fs::write(&lock_file, "").unwrap();

        assert!(
            has_lock_file(artifact.to_str().unwrap()),
            "should detect lock file"
        );
    }

    #[test]
    fn has_lock_file_returns_false_when_no_lock() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("2026-04-01-research.md");
        std::fs::write(&artifact, "content").unwrap();

        assert!(
            !has_lock_file(artifact.to_str().unwrap()),
            "should not detect lock file when none exists"
        );
    }

    #[test]
    fn has_lock_file_returns_false_for_nonexistent_dir() {
        assert!(
            !has_lock_file("/nonexistent/dir/artifact.md"),
            "should return false for nonexistent directory"
        );
    }

    #[test]
    fn has_lock_file_ignores_non_lock_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("2026-04-01-research.md");
        std::fs::write(&artifact, "content").unwrap();
        // Create a non-lock sidecar (e.g. .bak)
        let bak_file = dir.path().join("2026-04-01-research.md.run-abc.bak");
        std::fs::write(&bak_file, "").unwrap();

        assert!(
            !has_lock_file(artifact.to_str().unwrap()),
            "should not match non-.lock sidecar files"
        );
    }

    #[test]
    fn warn_if_unlocked_does_not_error_when_quiet() {
        // When the artifact has no lock file the function should still succeed
        // (it only prints a warning). We set quiet mode to avoid cliclack output.
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("artifact.md");
        std::fs::write(&artifact, "content").unwrap();

        // Set quiet context so the cliclack warning call is skipped
        crate::context::set_context(crate::context::CliContext {
            json: false,
            no_input: false,
            yes: false,
            safe: false,
            verbose: 0,
            quiet: true,
        });
        let result = warn_if_unlocked(artifact.to_str().unwrap(), "research");
        assert!(result.is_ok(), "warn_if_unlocked should not error");
    }

    #[test]
    fn warn_if_unlocked_skips_when_locked() {
        // When a lock file exists, warn_if_unlocked should not warn at all
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("artifact.md");
        std::fs::write(&artifact, "content").unwrap();
        let lock = dir.path().join("artifact.md.run-xyz.lock");
        std::fs::write(&lock, "").unwrap();

        // Even without quiet mode, this should succeed because the lock exists
        // (no warning emitted)
        let result = warn_if_unlocked(artifact.to_str().unwrap(), "research");
        assert!(
            result.is_ok(),
            "warn_if_unlocked should not error when locked"
        );
    }
}
