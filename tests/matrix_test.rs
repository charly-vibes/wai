//! Integration tests for the `wai matrix` command group (decision matrix).
//!
//! Covers openspec change `add-decision-matrix`: core scaffold (tasks 1.x),
//! deciding (3.x), renderer (4.x), lints (5.x), and phase gate (6.1).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd
}

fn init_workspace(dir: &Path) {
    wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();
}

fn create_project(dir: &Path, name: &str) {
    wai_cmd(dir)
        .args(["new", "project", name])
        .assert()
        .success();
}

fn matrix_dir(dir: &Path, project: &str) -> std::path::PathBuf {
    dir.join(".wai/projects")
        .join(project)
        .join("designs/matrix")
}

// ── 1.1 matrix init ───────────────────────────────────────────────────────────

#[test]
fn matrix_init_scaffolds_expected_layout() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["matrix", "init", "Which storage engine should we adopt?"])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    assert!(m.join("problem.md").exists(), "problem.md must exist");
    assert!(m.join("criteria").is_dir(), "criteria/ must exist");
    assert!(
        m.join("approaches/01-status-quo").is_dir(),
        "approaches/01-status-quo/ must exist"
    );
    assert!(
        m.join("approaches/01-status-quo/_description.md").exists(),
        "status quo needs _description.md"
    );
    assert!(m.join("decision.md").exists(), "decision.md must exist");

    let problem = fs::read_to_string(m.join("problem.md")).unwrap();
    assert!(
        problem.contains("Which storage engine should we adopt?"),
        "problem statement must be written into problem.md"
    );
}

// 1.2 — status quo is the first column on init (verified via _description.md)

#[test]
fn matrix_init_twice_fails_naming_existing_matrix() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["matrix", "init", "first"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["matrix", "init", "second"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("matrix"));
}

#[test]
fn matrix_init_requires_workspace() {
    let tmp = TempDir::new().unwrap();
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .failure();
}

// ── 1.3 criterion add ─────────────────────────────────────────────────────────

#[test]
fn matrix_criterion_add_touches_every_approach() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "operational-cost"])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    let crit = m.join("criteria/01-operational-cost.md");
    assert!(crit.exists(), "criterion definition file must exist");
    assert!(
        m.join("approaches/01-status-quo/01-operational-cost")
            .is_dir(),
        "cell dir must be created in status quo"
    );
    assert!(
        m.join("approaches/02-event-sourcing/01-operational-cost")
            .is_dir(),
        "cell dir must be created in every approach"
    );
}

// ── 1.4 approach add ──────────────────────────────────────────────────────────

#[test]
fn matrix_approach_add_creates_empty_cells_for_all_criteria() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "impact"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "risk"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    let approach = m.join("approaches/02-event-sourcing");
    assert!(
        approach.join("_description.md").exists(),
        "new approach needs _description.md"
    );
    assert!(approach.join("01-impact").is_dir());
    assert!(approach.join("02-risk").is_dir());
}

// ── numbering ─────────────────────────────────────────────────────────────────

#[test]
fn matrix_add_commands_number_sequentially() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "impact"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "risk"])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    assert!(m.join("criteria/01-impact.md").exists());
    assert!(m.join("criteria/02-risk.md").exists());

    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "cqrs"])
        .assert()
        .success();
    assert!(m.join("approaches/02-event-sourcing").is_dir());
    assert!(m.join("approaches/03-cqrs").is_dir());
}
