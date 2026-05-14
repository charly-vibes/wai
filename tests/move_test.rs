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

// ── move between PARA categories ──────────────────────────────────────────────

#[test]
fn move_project_to_archives() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "old-proj");

    wai_cmd(tmp.path())
        .args(["move", "old-proj", "archives"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Moved"));

    assert!(
        !tmp.path().join(".wai/projects/old-proj").exists(),
        "project dir should no longer exist in projects/"
    );
    assert!(
        tmp.path().join(".wai/archives/old-proj").exists(),
        "project dir should exist in archives/"
    );
}

#[test]
fn move_project_to_areas() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");

    wai_cmd(tmp.path())
        .args(["move", "my-proj", "areas"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Moved"));

    assert!(
        !tmp.path().join(".wai/projects/my-proj").exists(),
        "project dir should no longer exist in projects/"
    );
    assert!(
        tmp.path().join(".wai/areas/my-proj").exists(),
        "project dir should exist in areas/"
    );
}

// ── failure: unknown item ─────────────────────────────────────────────────────

#[test]
fn move_nonexistent_item_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["move", "ghost", "archives"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
