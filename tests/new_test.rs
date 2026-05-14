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

// ── project creation ──────────────────────────────────────────────────────────

#[test]
fn new_project_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["new", "project", "my-app"])
        .assert()
        .success();

    let proj = tmp.path().join(".wai/projects/my-app");
    assert!(proj.join("research").is_dir());
    assert!(proj.join("plans").is_dir());
    assert!(proj.join("handoffs").is_dir());

    let state = fs::read_to_string(proj.join(".state")).unwrap();
    assert!(
        state.contains("current: research"),
        "new project should start in research phase"
    );
}

// ── area creation (non-project item type) ─────────────────────────────────────

#[test]
fn new_area_creates_area_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["new", "area", "dev-standards"])
        .assert()
        .success();

    assert!(
        tmp.path().join(".wai/areas/dev-standards").is_dir(),
        "area directory should be created under .wai/areas/"
    );
}

// ── duplicate creation ────────────────────────────────────────────────────────

#[test]
fn new_project_duplicate_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["new", "project", "my-app"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["new", "project", "my-app"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}
