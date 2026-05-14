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

fn write_artifact(
    dir: &std::path::Path,
    project: &str,
    subdir: &str,
    filename: &str,
    content: &str,
) {
    let path = dir
        .join(".wai")
        .join("projects")
        .join(project)
        .join(subdir)
        .join(filename);
    fs::write(path, content).unwrap();
}

// ── chronological output ──────────────────────────────────────────────────────

#[test]
fn timeline_shows_dated_artifacts_in_output() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-first.md",
        "First\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "plans",
        "2026-01-20-second.md",
        "Second\n",
    );

    wai_cmd(tmp.path())
        .args(["timeline", "my-app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Timeline for"))
        .stdout(predicate::str::contains("2026-01-10"))
        .stdout(predicate::str::contains("2026-01-20"));
}

// ── date filter ───────────────────────────────────────────────────────────────

#[test]
fn timeline_from_filter_excludes_older_entries() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-05-old.md",
        "Old\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-15-new.md",
        "New\n",
    );

    wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--from", "2026-02-01"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2026-02-15"))
        .stdout(predicate::str::contains("2026-01-05").not());
}

// ── failure: missing project ──────────────────────────────────────────────────

#[test]
fn timeline_nonexistent_project_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["timeline", "ghost"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
