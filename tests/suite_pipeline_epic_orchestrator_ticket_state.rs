//! Integration tests for the epic-orchestrator pipeline's per-ticket run
//! state contract (ticket: wai-hr0w).
//!
//! Covers the state-file shape declared by the template (canon:
//! orchestrator-subagents.md — "progress must be pokeable" / briefs are repo
//! artifacts): `.wai/projects/<project>/runs/<ticket>.state` with the ticket
//! id, branch, brief path, and `step id + sha` step-history entries, walked
//! through a simulated claim → gates → brief ticket.

#![allow(clippy::too_many_lines)]

mod common;

use common::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Collapse whitespace so prose assertions survive template line wrapping.
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Current HEAD sha of the temp repo (empty string when unavailable).
fn head_sha(dir: &std::path::Path) -> String {
    let out = StdCommand::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .expect("git rev-parse should run");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Stage everything and commit in the temp repo (simulates an agent commit).
fn commit_all(dir: &std::path::Path, message: &str) {
    let out = StdCommand::new("git")
        .args(["add", "-A"])
        .current_dir(dir)
        .output()
        .expect("git add should run");
    assert!(
        out.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = StdCommand::new("git")
        .args(["commit", "--allow-empty", "-m", message])
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .expect("git commit should run");
    assert!(
        out.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The template must declare the per-ticket state-file contract the docs
/// describe: path, shape (ticket / branch / brief path / step history), and
/// retry-drift semantics.
#[test]
fn pipeline_epic_orchestrator_template_declares_ticket_state_contract() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let raw = fs::read_to_string(
        tmp.path()
            .join(".wai/resources/pipelines/epic-orchestrator.toml"),
    )
    .unwrap();
    let content = squash(&raw);

    // State-file path convention.
    assert!(
        content.contains(".wai/projects/<project>/runs/<ticket>.state"),
        "template must declare the per-ticket state file path, got:\n{content}"
    );

    // State-file shape: ticket id, branch, brief path, step history entries.
    assert!(
        content.contains("ticket id"),
        "state file must record the ticket id, got:\n{content}"
    );
    assert!(
        content.contains("branch"),
        "state file must record the branch, got:\n{content}"
    );
    assert!(
        content.contains("brief path"),
        "state file must record the brief path, got:\n{content}"
    );
    assert!(
        content.contains("step history"),
        "state file must record a step history, got:\n{content}"
    );
    // Checkpoint entries carry the step id plus the current HEAD sha.
    assert!(
        content.contains("step id + sha"),
        "checkpoints must be `step id + sha` entries, got:\n{content}"
    );

    // Retry/drift semantics: resume from state, refuse drifted retries.
    assert!(
        content.contains("drift check"),
        "retry entry must run a drift check, got:\n{content}"
    );
    assert!(
        content.contains("refuse to resume"),
        "retry must refuse to resume on drift, got:\n{content}"
    );
}

/// The template's spawn and verify steps must encode the refusal conditions
/// and single-model degradation the docs describe.
#[test]
fn pipeline_epic_orchestrator_template_declares_refusals_and_degradation() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let raw = fs::read_to_string(
        tmp.path()
            .join(".wai/resources/pipelines/epic-orchestrator.toml"),
    )
    .unwrap();
    let content = squash(&raw);

    // Spawn refusals (canon Invariants 4 + 2).
    assert!(
        content.contains("Refuse to spawn without a committed brief"),
        "spawn must refuse without a committed brief, got:\n{content}"
    );
    assert!(
        content.contains("Refuse to spawn without the state file"),
        "spawn must refuse without the state file, got:\n{content}"
    );

    // Single-model degradation in the verify step: lead-side checks,
    // non-blocking; only report-vs-reality contradiction blocks ship.
    assert!(
        content.contains("Single-model degradation"),
        "verify step must document the single-model degradation, got:\n{content}"
    );
    assert!(
        content.contains("lead-side checks"),
        "degradation path must name the lead-side checks, got:\n{content}"
    );
    assert!(
        content.contains("degradation is non-blocking"),
        "degradation must be non-blocking once lead-side checks pass, got:\n{content}"
    );
    assert!(
        content.to_lowercase().contains("contradiction"),
        "a report-vs-reality contradiction must still block the run, got:\n{content}"
    );
}

/// Walk a simulated ticket through claim → gates → brief, asserting the
/// per-ticket state file shape after each step and that the loop hands off
/// to the spawn step with the recorded brief.
#[test]
fn pipeline_epic_orchestrator_walks_claim_gates_brief_to_ticket_state() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "demo");

    // Make the temp dir a git repo so step checkpoints can record the
    // current HEAD sha, and so the brief is genuinely committed.
    git_repo_init_and_commit(tmp.path(), "init workspace");
    let sha_claim = head_sha(tmp.path());
    assert_eq!(sha_claim.len(), 40, "expected a 40-hex HEAD sha");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args([
            "pipeline",
            "start",
            "epic-orchestrator",
            "--topic=wai-test-child",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();
    assert!(
        run_id.starts_with("epic-orchestrator-"),
        "run id should identify the pipeline, got: {run_id}"
    );

    let state_path = tmp
        .path()
        .join(".wai/projects/demo/runs/wai-test-child.state");

    // ── claim ────────────────────────────────────────────────────────────
    // The agent writes the per-ticket state file at claim (create if absent).
    fs::create_dir_all(state_path.parent().unwrap()).unwrap();
    fs::write(
        &state_path,
        format!(
            "ticket: wai-test-child\nbranch: main\nbrief_path: \nstep_history: []\n- claim @ {sha_claim}\n"
        ),
    )
    .unwrap();

    // Orientation artifact satisfies the claim step's structural gate.
    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "--project=demo",
            "ORIENT: claimed wai-test-child; no blockers",
        ])
        .output()
        .expect("wai add should run");
    assert!(
        out.status.success(),
        "add research failed: {:?}",
        out.stderr
    );

    // Advancing past claim prints the gates prompt.
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(
        out.status.success(),
        "pipeline next failed: {:?}",
        out.stderr
    );
    let stdout = strip_ansi(&String::from_utf8_lossy(&out.stdout));
    assert!(
        stdout.contains("gates"),
        "advancing past claim should reach the gates step, got:\n{stdout}"
    );

    // ── gates ────────────────────────────────────────────────────────────
    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "--project=demo",
            "GATES: baseline suite green",
        ])
        .output()
        .expect("wai add should run");
    assert!(
        out.status.success(),
        "add research failed: {:?}",
        out.stderr
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(
        out.status.success(),
        "pipeline next failed: {:?}",
        out.stderr
    );

    // Simulate the agent committing claim/gates work so the brief checkpoint
    // lands on a new HEAD.
    commit_all(tmp.path(), "claim + gates artifacts");
    let sha_brief = head_sha(tmp.path());
    assert_ne!(sha_brief, sha_claim, "brief checkpoint sha must advance");

    // ── brief ────────────────────────────────────────────────────────────
    // Briefs are repo artifacts at a recorded path (canon Invariant 4).
    let brief_rel = ".wai/projects/demo/briefs/wai-test-child.md";
    let brief_path = tmp.path().join(brief_rel);
    fs::create_dir_all(brief_path.parent().unwrap()).unwrap();
    fs::write(
        &brief_path,
        "# Brief: wai-test-child\n\n## Why\n\nDelegated: single bounded test.\n\n## Completion criteria\n\n- cargo test --test wai-test-child (exit 0)\n\n## Spawn model\n\nModel: default; context ceiling ≤ 40% of the model window.\n",
    )
    .unwrap();

    // The agent records the brief path in the state file and checkpoints.
    let state = fs::read_to_string(&state_path).unwrap();
    let state = state.replace("brief_path: \n", &format!("brief_path: {brief_rel}\n"));
    let state = format!("{state}- gates @ {sha_claim}\n- brief @ {sha_brief}\n");
    fs::write(&state_path, state).unwrap();

    // Brief record artifact satisfies the brief step's structural gate.
    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "design",
            "--project=demo",
            &format!("BRIEF: {brief_rel}; committed"),
        ])
        .output()
        .expect("wai add should run");
    assert!(out.status.success(), "add design failed: {:?}", out.stderr);

    // Advancing past brief hands off to spawn.
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(
        out.status.success(),
        "pipeline next failed: {:?}",
        out.stderr
    );
    let stdout = strip_ansi(&String::from_utf8_lossy(&out.stdout));
    assert!(
        stdout.contains("Spawn"),
        "advancing past brief should reach the spawn step, got:\n{stdout}"
    );
    // The spawn prompt carries the refusal conditions the docs describe.
    assert!(
        stdout.contains("Refuse to spawn without a committed brief"),
        "spawn prompt must refuse uncommitted briefs, got:\n{stdout}"
    );
    assert!(
        stdout.contains("Refuse to spawn without the state file"),
        "spawn prompt must refuse missing state files, got:\n{stdout}"
    );

    // ── state-file shape ─────────────────────────────────────────────────
    let state = fs::read_to_string(&state_path).unwrap();
    assert!(
        state.contains("ticket: wai-test-child"),
        "state file must record the ticket id, got:\n{state}"
    );
    assert!(
        state.contains("branch: main"),
        "state file must record the branch, got:\n{state}"
    );
    assert!(
        state.contains(&format!("brief_path: {brief_rel}")),
        "state file must record the committed brief path, got:\n{state}"
    );
    for checkpoint in ["- claim @ ", "- gates @ ", "- brief @ "] {
        assert!(
            state.contains(checkpoint),
            "step history must contain a `{checkpoint}` entry, got:\n{state}"
        );
    }
    // Every checkpoint entry carries a 40-hex sha.
    for line in state.lines().filter(|l| l.starts_with("- ")) {
        let sha = line
            .rsplit_once(" @ ")
            .map(|(_, s)| s.trim())
            .expect("checkpoint entry must be `step id @ sha`");
        assert_eq!(
            sha.len(),
            40,
            "checkpoint sha must be a full git sha, got: {line}"
        );
        assert!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "checkpoint sha must be hex, got: {line}"
        );
    }
    // Step history is ordered: claim before gates before brief.
    let claim_idx = state.find("- claim @ ").unwrap();
    let gates_idx = state.find("- gates @ ").unwrap();
    let brief_idx = state.find("- brief @ ").unwrap();
    assert!(
        claim_idx < gates_idx && gates_idx < brief_idx,
        "step history must be in loop order claim → gates → brief, got:\n{state}"
    );
}
