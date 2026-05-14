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

// ── phase show ────────────────────────────────────────────────────────────────

#[test]
fn phase_show_displays_current_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["phase", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("research"));
}

// ── phase set (explicit advancement) ─────────────────────────────────────────

#[test]
fn phase_set_jumps_to_target_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .assert()
        .success()
        .stderr(predicate::str::contains("implement"));

    let state = fs::read_to_string(tmp.path().join(".wai/projects/my-app/.state")).unwrap();
    assert!(
        state.contains("current: implement"),
        "state file should reflect the new phase"
    );
}

// ── failure: invalid phase ────────────────────────────────────────────────────

#[test]
fn phase_set_invalid_phase_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["phase", "set", "banana"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown phase"));
}
