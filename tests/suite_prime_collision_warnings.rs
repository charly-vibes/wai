//! Integration tests for cross-agent collision surfacing (ticket: wai-76dq,
//! R4 of harden-orchestration-gates).
//!
//! Detection is heuristic and best-effort (design.md Decision 4): prime and
//! `wai pipeline next` warn (never hard-fail) when a git stash entry message
//! matches an agent-provenance pattern (`<agent> WIP`, "parked", "owns") or
//! when an unexpired agent lock-file lives under `.wai/`. Silent when green.

mod common;

use common::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Make the temp workspace a git repo with one commit and a stash entry.
fn seed_repo_with_stash(dir: &std::path::Path, stash_message: &str) {
    let run = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git should run");
        assert!(out.status.success(), "git {args:?} failed");
    };
    run(&["init", "-q"]);
    run(&["config", "user.email", "test@example.com"]);
    run(&["config", "user.name", "test"]);
    fs::write(dir.join("tracked.txt"), "one\n").unwrap();
    run(&["add", "tracked.txt"]);
    run(&["commit", "-q", "-m", "initial"]);
    fs::write(dir.join("tracked.txt"), "two\n").unwrap();
    run(&["stash", "push", "-m", stash_message]);
}

/// Prime warns on a provenance-matching stash entry (agent WIP parked in a
/// stash is invisible to other agents — the evallerina-00n failure mode).
#[test]
fn prime_warns_on_provenance_matching_stash_entry() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-ws");
    seed_repo_with_stash(tmp.path(), "codex WIP on main: parked refactor");

    let prime = hermetic_wai_cmd(tmp.path())
        .args(["prime"])
        .output()
        .expect("prime should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&prime.stdout),
        String::from_utf8_lossy(&prime.stderr)
    );
    assert!(
        prime.status.success(),
        "prime must not hard-fail: {combined}"
    );
    assert!(
        combined.contains("codex WIP on main: parked refactor"),
        "prime must surface the provenance-matching stash entry: {combined}"
    );
}

/// A stash entry without agent provenance must NOT trigger the warning —
/// the feature is silent-when-green.
#[test]
fn prime_is_silent_on_non_matching_stash_entry() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    seed_repo_with_stash(tmp.path(), "fix readme wording typos");

    let prime = hermetic_wai_cmd(tmp.path())
        .args(["prime"])
        .output()
        .expect("prime should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&prime.stdout),
        String::from_utf8_lossy(&prime.stderr)
    );
    assert!(
        prime.status.success(),
        "prime must not hard-fail: {combined}"
    );
    assert!(
        !combined.contains("fix readme wording typos"),
        "prime must not surface a non-provenance stash entry: {combined}"
    );
}

/// An unexpired agent lock-file under .wai/ triggers the warning.
#[test]
fn prime_warns_on_unexpired_agent_lock_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let locks_dir = tmp.path().join(".wai/projects/test-ws/research");
    fs::create_dir_all(&locks_dir).unwrap();
    fs::write(
        locks_dir.join("some-artifact.md.lock"),
        format!(
            "artifact = \"some-artifact.md\"\nlocked_at = \"{}\"\nlock_hash = \"abc\"\npipeline_run = \"run-x\"\npipeline_step = \"execute\"\n",
            chrono::Utc::now().to_rfc3339()
        ),
    )
    .unwrap();

    let prime = hermetic_wai_cmd(tmp.path())
        .args(["prime"])
        .output()
        .expect("prime should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&prime.stdout),
        String::from_utf8_lossy(&prime.stderr)
    );
    assert!(
        prime.status.success(),
        "prime must not hard-fail: {combined}"
    );
    assert!(
        combined.contains("some-artifact.md.lock"),
        "prime must surface the unexpired agent lock-file: {combined}"
    );
}

/// `wai pipeline next` surfaces the same warnings while a run is active
/// (task 4.3) — before the gate check, so a failing gate still shows them.
#[test]
fn pipeline_next_surfaces_collision_warnings_when_run_active() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-ws");
    seed_repo_with_stash(tmp.path(), "claude owns the ledger work right now");
    hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("init should run");
    hermetic_wai_cmd(tmp.path())
        .args([
            "pipeline",
            "start",
            "epic-orchestrator",
            "--topic=collision",
        ])
        .output()
        .expect("start should run");

    let next = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("next should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&next.stdout),
        String::from_utf8_lossy(&next.stderr)
    );
    assert!(
        combined.contains("claude owns the ledger work right now"),
        "pipeline next must surface the collision warning while a run is active: {combined}"
    );
}

/// Substring lookalikes ('towns', 'owners', 'wiping') must not trigger the
/// provenance warning (RO5U pass-1 medium finding: word-boundary match).
#[test]
fn prime_is_silent_on_provenance_lookalike_words() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-ws");
    seed_repo_with_stash(
        tmp.path(),
        "owners manual and towns near the wiping station",
    );

    let prime = hermetic_wai_cmd(tmp.path())
        .args(["prime"])
        .output()
        .expect("prime should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&prime.stdout),
        String::from_utf8_lossy(&prime.stderr)
    );
    assert!(
        prime.status.success(),
        "prime must not hard-fail: {combined}"
    );
    assert!(
        !combined.contains("owners manual"),
        "lookalike words must not trigger the collision warning: {combined}"
    );
}
