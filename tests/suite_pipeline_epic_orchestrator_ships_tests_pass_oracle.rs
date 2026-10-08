//! wai-lqdr: `wai pipeline init epic-orchestrator` must ship the tests-pass
//! oracle script so its verify/ship gates resolve out of the box.

#![allow(clippy::too_many_lines)]

mod common;

use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn pipeline_init_epic_orchestrator_ships_tests_pass_oracle_script() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let script = tmp.path().join(".wai/resources/oracles/tests-pass.sh");
    let meta =
        fs::metadata(&script).expect("pipeline init epic-orchestrator should ship tests-pass.sh");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(
            meta.permissions().mode() & 0o111 != 0,
            "shipped oracle must be executable: {:?}",
            script
        );
    }
}

/// Helper: init workspace, init the built-in epic-orchestrator pipeline.
fn tests_pass_oracle_sandbox() -> TempDir {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    tmp
}

/// Helper: write an artifact and run the shipped tests-pass oracle against it,
/// returning (exit code, stderr).
fn run_tests_pass_oracle(dir: &std::path::Path, artifact: &str) -> (i32, String) {
    fs::write(dir.join("artifact.md"), artifact).unwrap();
    let script = dir.join(".wai/resources/oracles/tests-pass.sh");
    let output = std::process::Command::new(&script)
        .arg("artifact.md")
        .current_dir(dir)
        .output()
        .expect("tests-pass.sh should exist and be executable");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn tests_pass_oracle_accepts_artifact_with_verification_evidence() {
    let tmp = tests_pass_oracle_sandbox();
    let (code, stderr) = run_tests_pass_oracle(tmp.path(), "command=cargo test\n");
    assert_eq!(
        code, 0,
        "expected accept for explicit evidence, stderr: {stderr}"
    );
}

#[test]
fn tests_pass_oracle_rejects_artifact_without_evidence() {
    let tmp = tests_pass_oracle_sandbox();
    let (code, stderr) = run_tests_pass_oracle(tmp.path(), "wrote some code, seems fine");
    assert_eq!(
        code, 1,
        "expected reject without evidence, stderr: {stderr}"
    );
    assert!(
        stderr.contains("verification"),
        "stderr must explain the missing evidence, got: {stderr}"
    );
}
