use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    // Update notice off in tests: existing assertions expect quiet stderr and
    // must not depend on real ~/.cache state (wai-r3p0 tidy).
    cmd.env("GENESIS_NO_UPDATE_CHECK", "1");
    cmd
}

fn init_workspace(dir: &std::path::Path) {
    let out = wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

fn create_project(dir: &std::path::Path, name: &str) {
    let out = wai_cmd(dir)
        .args(["new", "project", name])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

// ── handoff creation for active project ──────────────────────────────────────

#[test]
fn close_creates_handoff_for_named_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Handoff created:"));

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

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "alpha"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Handoff created:"));

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

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "nonexistent"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}

// ── clearing stale complete pipeline-run pointer (wai-pa3b) ──────────────────

/// Write a pipeline definition (2 steps), a run state file, and the `.last-run`
/// pointer. `current_step` == total means the run is complete.
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
    // Single source of truth: .last-run pointer
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
    assert!(last_run.exists(), "pointer exists before close");

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Cleared complete pipeline run"));

    assert!(
        !last_run.exists(),
        ".last-run should be removed after close"
    );
}

// ── close-time enforcement for in-progress pipeline runs (wai-csgb) ──────────

#[test]
fn close_refuses_incomplete_pipeline_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    // current_step == 0 of 2 → run is mid-flight.
    write_pipeline_run(tmp.path(), "flow", 0);

    let last_run = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_state = tmp.path().join(".wai/pipeline-runs/flow-run.yml");
    let before = fs::read_to_string(&run_state).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(!(stdout.contains("Handoff created:")));
    assert!(stderr.contains("flow-run"));
    assert!(stderr.contains("flow"));
    assert!(stderr.contains("step 1 of 2"));
    assert!(stderr.contains("wai pipeline next"));
    assert!(stderr.contains("wai close --force"));

    // Run state must be untouched by the refusal.
    assert_eq!(
        fs::read_to_string(&run_state).unwrap(),
        before,
        "refused close must not modify the run state file"
    );
    assert!(last_run.exists(), ".last-run must survive a refused close");

    // Refusal must not create a handoff either.
    let handoffs_dir = tmp.path().join(".wai/projects/myproject/handoffs");
    if handoffs_dir.exists() {
        let files: Vec<_> = fs::read_dir(&handoffs_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            files.is_empty(),
            "refused close must not create a handoff file"
        );
    }
}

#[test]
fn close_force_overrides_incomplete_pipeline_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    // current_step == 0 of 2 → run is mid-flight.
    write_pipeline_run(tmp.path(), "flow", 0);

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject", "--force"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Handoff created:"));

    // The .last-run pointer survives a forced close: --force means "close the
    // session anyway", not "clear the run" — stale-run GC (wai-vx02.2) owns
    // abandoned-run cleanup.
    let last_run = tmp.path().join(".wai/resources/pipelines/.last-run");
    assert!(
        last_run.exists(),
        "--force must not clear an in-progress run pointer"
    );
    // And the refusal path must have been skipped (close proceeded).
    let handoffs_dir = tmp.path().join(".wai/projects/myproject/handoffs");
    let files: Vec<_> = fs::read_dir(&handoffs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1, "forced close creates the handoff");
}

#[test]
fn close_still_clears_complete_pipeline_run_pointers() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    // current_step == 2 (total) → run is complete.
    write_pipeline_run(tmp.path(), "flow", 2);

    let last_run = tmp.path().join(".wai/resources/pipelines/.last-run");
    assert!(last_run.exists(), "pointer exists before close");

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Cleared complete pipeline run"));

    assert!(
        !last_run.exists(),
        ".last-run should be removed after close"
    );
}

#[test]
fn close_with_no_pipeline_run_is_unchanged() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    // No pipeline run at all.

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Nothing to clear: no run pointer was ever created, and close reports no
    // pipeline clearing.
    let last_run = tmp.path().join(".wai/resources/pipelines/.last-run");
    assert!(
        !last_run.exists(),
        "no pointer should appear on a bare close"
    );
}
