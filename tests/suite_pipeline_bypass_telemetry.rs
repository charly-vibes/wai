//! Integration tests for gate bypass telemetry (ticket: wai-giqg, R2 of
//! harden-orchestration-gates).
//!
//! `--no-verify` bypasses cannot be recorded by hooks (the flag skips them),
//! so the spec mandates a `wai pipeline bypass <reason>` record command that
//! appends to the active run's state ledger, and `wai pipeline status` must
//! surface the bypass count and last disclosed reason. Ledger-only — this is
//! telemetry, not enforcement (design.md Decision 2).

mod common;

use assert_cmd::Command;
use common::*;
use std::fs;
use tempfile::TempDir;

/// Hermetic wai command: strips an ambient WAI_PIPELINE_RUN so the child
/// resolves the active run from the temp workspace's `.last-run` pointer,
/// never from the orchestrating session's environment.
fn hermetic_wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = wai_cmd(dir);
    cmd.env_remove("WAI_PIPELINE_RUN");
    cmd
}

/// A bypass recorded during an active run appends a ledger entry to the run
/// state and is surfaced by `wai pipeline status` (count + last reason).
#[test]
fn bypass_during_active_run_recorded_and_surfaced() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let init = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "init", "tdd-ro5"])
        .output()
        .expect("init should run");
    assert!(init.status.success(), "init failed");

    let start = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "start", "tdd-ro5", "--topic=bypass-test"])
        .output()
        .expect("start should run");
    assert!(start.status.success(), "start failed");

    let bypass = hermetic_wai_cmd(tmp.path())
        .args([
            "pipeline",
            "bypass",
            "skipped pre-push hook: flaky under parallel load",
        ])
        .output()
        .expect("bypass should run");
    assert!(
        bypass.status.success(),
        "bypass during active run must succeed: {}{}",
        String::from_utf8_lossy(&bypass.stdout),
        String::from_utf8_lossy(&bypass.stderr)
    );

    // Ledger entry exists in the run state file.
    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();
    let run_state = fs::read_to_string(
        tmp.path()
            .join(".wai/pipeline-runs")
            .join(format!("{run_id}.yml")),
    )
    .unwrap();
    assert!(
        run_state.contains("bypasses"),
        "run state must contain a bypasses ledger:\n{run_state}"
    );
    assert!(
        run_state.contains("skipped pre-push hook"),
        "run state ledger must contain the disclosed reason:\n{run_state}"
    );

    // Second bypass: count accumulates.
    let bypass2 = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "bypass", "second disclosed reason"])
        .output()
        .expect("second bypass should run");
    assert!(bypass2.status.success(), "second bypass failed");

    // Status surfaces count + last reason.
    let status = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "status"])
        .output()
        .expect("status should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(status.status.success(), "status failed: {combined}");
    assert!(
        combined.contains("Gate bypasses: 2"),
        "status must show the bypass count 2: {combined}"
    );
    assert!(
        combined.contains("second disclosed reason"),
        "status must show the last disclosed bypass reason: {combined}"
    );
}

/// A bypass with no active pipeline run writes nothing and raises no error
/// (spec scenario: bypass without active run ignored).
#[test]
fn bypass_without_active_run_is_silent_noop() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let bypass = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "bypass", "no run active"])
        .output()
        .expect("bypass should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&bypass.stdout),
        String::from_utf8_lossy(&bypass.stderr)
    );
    assert!(
        bypass.status.success(),
        "bypass without active run must not error: {combined}"
    );

    let runs_dir = tmp.path().join(".wai/pipeline-runs");
    if runs_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&runs_dir).unwrap().collect();
        assert!(
            entries.is_empty(),
            "no run-state entries may be written without an active run"
        );
    }
}
