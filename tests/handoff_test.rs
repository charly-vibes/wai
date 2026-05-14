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

// ── handoff creation ─────────────────────────────────────────────────────────

#[test]
fn handoff_create_for_named_project_succeeds() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["handoff", "create", "myproject"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Created handoff for 'myproject'"));

    let handoffs_dir = tmp.path().join(".wai/projects/myproject/handoffs");
    let files: Vec<_> = fs::read_dir(&handoffs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1, "expected exactly one handoff file");
}

// ── handoff content ──────────────────────────────────────────────────────────

#[test]
fn handoff_create_includes_project_and_phase_context() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");

    wai_cmd(tmp.path())
        .args(["handoff", "create", "alpha"])
        .assert()
        .success();

    let handoffs_dir = tmp.path().join(".wai/projects/alpha/handoffs");
    let entry = fs::read_dir(&handoffs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .expect("handoff file should exist");
    let content = fs::read_to_string(entry.path()).unwrap();

    assert!(
        content.contains("project: alpha"),
        "handoff should include project name in frontmatter"
    );
    assert!(
        content.contains("phase:"),
        "handoff should include current phase in frontmatter"
    );
    assert!(
        content.contains("# Session Handoff"),
        "handoff should contain the standard heading"
    );
}

// ── failure: missing project ─────────────────────────────────────────────────

#[test]
fn handoff_create_for_missing_project_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["handoff", "create", "nonexistent"])
        .assert()
        .failure();
}
