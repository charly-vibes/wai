use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd
}

fn init_workspace(dir: &std::path::Path) {
    wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();
}

fn create_project(dir: &std::path::Path, name: &str) {
    wai_cmd(dir)
        .args(["new", "project", name])
        .assert()
        .success();
}

// ── handoff creation for active project ──────────────────────────────────────

#[test]
fn close_creates_handoff_for_named_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Handoff created:"));

    let handoffs_dir = tmp.path().join(".wai/projects/myproject/handoffs");
    let files: Vec<_> = fs::read_dir(&handoffs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        files.len(),
        1,
        "expected exactly one handoff file after close"
    );
}

// ── explicit project selection when multiple projects exist ───────────────────

#[test]
fn close_with_project_flag_targets_only_named_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    wai_cmd(tmp.path())
        .args(["close", "--project", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Handoff created:"));

    let alpha_handoffs = tmp.path().join(".wai/projects/alpha/handoffs");
    let beta_handoffs = tmp.path().join(".wai/projects/beta/handoffs");

    let alpha_files: Vec<_> = fs::read_dir(&alpha_handoffs)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let beta_files: Vec<_> = fs::read_dir(&beta_handoffs)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(alpha_files.len(), 1, "alpha should have one handoff");
    assert_eq!(beta_files.len(), 0, "beta should have no handoff");
}

// ── failure: unknown project ──────────────────────────────────────────────────

#[test]
fn close_unknown_project_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["close", "--project", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// ── clearing stale complete pipeline-run pointer (wai-pa3b) ──────────────────

/// Write a pipeline definition (2 steps), a run state file, and both active-run
/// pointers (`.wai/.pipeline-run` and `.wai/resources/pipelines/.last-run`).
/// `current_step` == total means the run is complete.
fn write_pipeline_run(dir: &std::path::Path, pipeline: &str, current_step: usize) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(
        pipelines_dir.join(format!("{pipeline}.toml")),
        "[pipeline]\nname = \"flow\"\ndescription = \"d\"\n\
         [[steps]]\nid = \"a\"\nprompt = \"do {{topic}}\"\n\
         [[steps]]\nid = \"b\"\nprompt = \"do {{topic}}\"\n",
    )
    .unwrap();

    let run_id = format!("{pipeline}-run");
    let runs_dir = dir.join(".wai/pipeline-runs");
    fs::create_dir_all(&runs_dir).unwrap();
    fs::write(
        runs_dir.join(format!("{run_id}.yml")),
        format!(
            "run_id: {run_id}\npipeline: {pipeline}\ntopic: t\ncreated_at: '2026-07-29T00:00:00Z'\ncurrent_step: {current_step}\napprovals: {{}}\n"
        ),
    )
    .unwrap();
    // Both active-run pointers used by wai (file-based resolution).
    fs::write(dir.join(".wai/.pipeline-run"), &run_id).unwrap();
    fs::write(pipelines_dir.join(".last-run"), &run_id).unwrap();
}

#[test]
fn close_clears_complete_pipeline_run_pointers() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    // current_step == 2 (total) → run is complete.
    write_pipeline_run(tmp.path(), "flow", 2);

    let last_run = tmp.path().join(".wai/resources/pipelines/.last-run");
    let pipeline_run = tmp.path().join(".wai/.pipeline-run");
    assert!(
        last_run.exists() && pipeline_run.exists(),
        "pointers exist before close"
    );

    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared complete pipeline run"));

    assert!(
        !last_run.exists(),
        ".last-run should be removed after close"
    );
    assert!(
        !pipeline_run.exists(),
        ".pipeline-run should be removed after close"
    );
}

#[test]
fn close_preserves_in_progress_pipeline_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    // current_step == 0 of 2 → run is in progress.
    write_pipeline_run(tmp.path(), "flow", 0);

    let last_run = tmp.path().join(".wai/resources/pipelines/.last-run");
    let pipeline_run = tmp.path().join(".wai/.pipeline-run");

    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared complete pipeline run").not());

    assert!(
        last_run.exists(),
        ".last-run must remain for an in-progress run"
    );
    assert!(
        pipeline_run.exists(),
        ".pipeline-run must remain for an in-progress run"
    );
}

#[test]
fn close_with_no_pipeline_run_is_unchanged() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    // No pipeline run at all.

    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared complete pipeline run").not());
}
