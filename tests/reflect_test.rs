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

fn reflect_workspace(dir: &std::path::Path) {
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .output()
        .ok();
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init", "--no-gpg-sign"])
        .current_dir(dir)
        .output()
        .ok();

    init_workspace(dir);
    create_project(dir, "test-proj");
    fs::write(dir.join("CLAUDE.md"), "# Claude\n").unwrap();
}

const MOCK_REFLECT_CONTENT: &str = "\
## Project-Specific AI Context\n\
_Last reflected: 2026-02-24 · 1 sessions analyzed_\n\
\n\
### Conventions\n\
- Use TDD always";

// ── synthesis output (populated workspace) ───────────────────────────────────

#[test]
fn reflect_populated_workspace_writes_resource_file() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--yes"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .assert()
        .success()
        .stdout(predicate::str::contains("Wrote"));

    let refl_dir = tmp.path().join(".wai/resources/reflections");
    assert!(refl_dir.exists(), "reflections directory should be created");

    let entries: Vec<_> = fs::read_dir(&refl_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "exactly one reflection resource file expected"
    );

    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.contains("Use TDD always"),
        "resource file should contain mock reflection content"
    );
}

#[test]
fn reflect_populated_workspace_creates_reflect_meta() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--yes"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .assert()
        .success();

    let meta_path = tmp.path().join(".wai/projects/test-proj/.reflect-meta");
    assert!(
        meta_path.exists(),
        ".reflect-meta should be created after reflect"
    );
}

// ── dry-run path ─────────────────────────────────────────────────────────────

#[test]
fn reflect_dry_run_does_not_create_resource_file() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--dry-run"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));

    let refl_dir = tmp.path().join(".wai/resources/reflections");
    assert!(
        !refl_dir.exists() || fs::read_dir(&refl_dir).unwrap().count() == 0,
        "dry-run must not create a reflection resource file"
    );
}

// ── failure / empty-state ────────────────────────────────────────────────────

#[test]
fn reflect_outside_wai_workspace_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    // No `wai init` — no .wai/ directory exists.

    wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--yes"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .assert()
        .failure();
}

#[test]
fn reflect_missing_output_target_fails_after_removing_claude_md() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    // Remove both CLAUDE.md and AGENTS.md so detect_output_targets errors.
    fs::remove_file(tmp.path().join("CLAUDE.md")).ok();
    let _ = fs::remove_file(tmp.path().join("AGENTS.md"));

    wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--yes"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .assert()
        .failure()
        .stderr(predicate::str::contains("CLAUDE.md").or(predicate::str::contains("AGENTS.md")));
}
