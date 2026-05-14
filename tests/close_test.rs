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
