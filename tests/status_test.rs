use assert_cmd::Command;
use predicates::prelude::*;
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

// ── project summary and suggestions ──────────────────────────────────────────

#[test]
fn status_shows_project_name_in_summary() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-app"));
}

// ── JSON output ───────────────────────────────────────────────────────────────

#[test]
fn status_json_flag_emits_suggestions_field() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"suggestions\""));
}

// ── empty-state path ──────────────────────────────────────────────────────────

#[test]
fn status_no_projects_succeeds_with_empty_workspace() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path()).args(["status"]).assert().success();
}
