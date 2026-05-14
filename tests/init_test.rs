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

// ── fresh workspace initialization ───────────────────────────────────────────

#[test]
fn init_creates_wai_directory_structure() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["init", "--name", "my-ws"])
        .assert()
        .success();

    assert!(tmp.path().join(".wai/projects").is_dir());
    assert!(tmp.path().join(".wai/areas").is_dir());
    assert!(tmp.path().join(".wai/resources").is_dir());
    assert!(tmp.path().join(".wai/archives").is_dir());

    let config = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert!(
        config.contains("my-ws"),
        "config.toml should contain workspace name"
    );
}

// ── re-init behavior ──────────────────────────────────────────────────────────

#[test]
fn init_reinit_warns_already_initialized() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["init", "--name", "my-ws"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["init", "--name", "my-ws"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already initialized"));
}

// ── non-default flag: --json ──────────────────────────────────────────────────

#[test]
fn init_json_flag_emits_structured_output() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["--json", "init", "--name", "my-ws", "--yes"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&out);
    let payload: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

    assert_eq!(
        payload["already_initialized"], false,
        "fresh init should report already_initialized: false"
    );
    assert_eq!(
        payload["project_name"], "my-ws",
        "JSON should include the workspace name"
    );
}
