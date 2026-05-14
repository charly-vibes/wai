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

// ── session orientation ───────────────────────────────────────────────────────

#[test]
fn prime_single_project_shows_orientation_output() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project: myproject"))
        .stdout(predicate::str::contains("wai prime"));
}

// ── project selection ─────────────────────────────────────────────────────────

#[test]
fn prime_project_flag_selects_named_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    wai_cmd(tmp.path())
        .args(["prime", "--project", "alpha", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project: alpha"))
        .stdout(predicate::str::contains("Project: beta").not());
}

// ── failure: unknown project ──────────────────────────────────────────────────

#[test]
fn prime_unknown_project_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["prime", "--project", "doesnotexist", "--no-input"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("doesnotexist"));
}
