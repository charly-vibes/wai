//! Purpose: regression tests for the canonical run wait primitive (ticket
//! wai-63h9, openspec change add-run-wait-primitive).
//! Responsibilities: pin the contract that (1) the epic-orchestrator template
//! requires `wai pipeline wait` for status waits and names free-form sleep
//! polling as a violation, (2) the exit-code mapping + summary-line formatting
//! is pure and testable (pass → 0 + one line naming PR/state/count; failure →
//! nonzero; timeout → nonzero + elapsed budget + last observed state), and
//! (3) a zero-checks PR is a terminal pass. Rationale: week 2026-10-13..19
//! measured 539 `sleep >=60s` poll commands (~53h blocked wall-clock) because
//! agents improvise poll loops the template prohibits without providing a
//! capability for (canon Invariant 10: prose does not enforce).

mod common;

use common::*;
use std::fs;
use tempfile::TempDir;
use wai::pipeline_wait;

/// Ticket wai-63h9 task 1.1: the epic-orchestrator template spawn step must
/// name `wai pipeline wait` as the required way to wait on CI/worktree status
/// and must name free-form `sleep <N>` poll loops as a violation — the
/// template currently encodes the prohibition ("Poll or log rather than
/// blind-blocking") without encoding the capability.
#[test]
fn orchestrator_template_requires_wait_primitive() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let init = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("init should run");
    assert!(init.status.success(), "init epic-orchestrator failed");

    let template = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    let content =
        fs::read_to_string(&template).expect("epic-orchestrator template should exist after init");

    // Locate the spawn step section.
    let spawn_pos = content
        .find("id = \"spawn\"")
        .expect("template must have a spawn step");
    let next_step = content[spawn_pos..]
        .find("[[steps]]")
        .map(|i| spawn_pos + i)
        .unwrap_or(content.len());
    let spawn_section = &content[spawn_pos..next_step];

    assert!(
        spawn_section.contains("wai pipeline wait"),
        "spawn step must require `wai pipeline wait` for status waits; \
         spawn section:\n{spawn_section}"
    );
    assert!(
        spawn_section.to_lowercase().contains("sleep"),
        "spawn step must name free-form sleep polling as a violation; \
         spawn section:\n{spawn_section}"
    );
}

/// Ticket wai-63h9 task 1.2: the exit-code mapping and summary-line
/// formatting are pure logic — pass exits 0 with exactly one summary line
/// naming the PR, state, and check count; failure and timeout exit nonzero;
/// the timeout summary includes the elapsed budget and the last observed
/// state so the agent never re-polls to reconstruct it.
#[test]
fn pipeline_wait_exit_codes_and_summary() {
    // Pass: exit code 0, one summary line naming PR, state, and count.
    let outcome = pipeline_wait::WaitOutcome::Passed {
        checks: 43,
        total: 43,
    };
    let line = pipeline_wait::summary_line(123, &outcome);
    assert_eq!(
        pipeline_wait::exit_code(&outcome),
        0,
        "pass must map to exit code 0"
    );
    assert!(
        line.contains("123"),
        "summary must name the PR number, line: {line}"
    );
    assert!(
        line.to_lowercase().contains("pass"),
        "summary must name the terminal state, line: {line}"
    );
    assert!(
        line.contains("43"),
        "summary must name the check count, line: {line}"
    );
    assert_eq!(
        line.lines().count(),
        1,
        "summary must be exactly one line, got: {line}"
    );

    // Failure: nonzero exit, summary names the failed state and counts.
    let outcome = pipeline_wait::WaitOutcome::Failed {
        checks: 41,
        total: 43,
    };
    assert_ne!(
        pipeline_wait::exit_code(&outcome),
        0,
        "check failure must map to a nonzero exit code"
    );
    let line = pipeline_wait::summary_line(123, &outcome);
    assert!(
        line.to_lowercase().contains("fail"),
        "failure summary must name the state, line: {line}"
    );
    assert_eq!(
        line.lines().count(),
        1,
        "failure summary must still be exactly one line, got: {line}"
    );

    // Timeout: nonzero exit, summary includes elapsed budget and the last
    // observed state so the agent can decide without re-polling.
    let outcome = pipeline_wait::WaitOutcome::Timeout {
        elapsed: "30m".to_string(),
        last_state: "3/43 pending".to_string(),
    };
    assert_ne!(
        pipeline_wait::exit_code(&outcome),
        0,
        "timeout exhaustion must map to a nonzero exit code"
    );
    let line = pipeline_wait::summary_line(123, &outcome);
    assert!(
        line.contains("30m"),
        "timeout summary must include the elapsed budget, line: {line}"
    );
    assert!(
        line.contains("3/43 pending"),
        "timeout summary must include the last observed state, line: {line}"
    );
}

/// Ro5u fix review: `summary_line` accepts any Display PR (branch names
/// render verbatim instead of degrading to `PR 0`), and a `total: 0`
/// outcome omits the counts segment instead of rendering a misleading
/// `(0/0)`.
#[test]
fn pipeline_wait_summary_handles_branch_names_and_unparsed_counts() {
    let outcome = pipeline_wait::WaitOutcome::Passed {
        checks: 0,
        total: 0,
    };
    let line = pipeline_wait::summary_line("feature-x", &outcome);
    assert_eq!(line, "PR feature-x: all checks pass");

    let failed = pipeline_wait::WaitOutcome::Failed {
        checks: 0,
        total: 0,
    };
    let line = pipeline_wait::summary_line("feature-x", &failed);
    assert_eq!(line, "PR feature-x: checks FAILED");
    assert!(
        !line.contains("(0/0)"),
        "must never render a misleading 0/0"
    );
}

/// Ro5u fix review: zero-checks detection must come BEFORE the exit-code
/// check — some gh versions exit 0 while printing the "no checks" message,
/// and a no-CI PR is a terminal pass either way.
#[test]
fn pipeline_wait_classify_detects_zero_checks_regardless_of_exit_code() {
    let outcome = pipeline_wait::classify(Some(0), "no checks reported on the 'x' branch", "");
    assert_eq!(outcome, pipeline_wait::WaitOutcome::ZeroChecks);

    let outcome = pipeline_wait::classify(Some(1), "", "no checks reported on the 'x' branch");
    assert_eq!(outcome, pipeline_wait::WaitOutcome::ZeroChecks);

    // A genuine pass with counts parses them; a genuine failure maps Failed.
    let outcome = pipeline_wait::classify(Some(0), "some table ... 43/43", "");
    assert_eq!(
        outcome,
        pipeline_wait::WaitOutcome::Passed {
            checks: 43,
            total: 43
        }
    );
    let outcome = pipeline_wait::classify(Some(1), "some table ... 41/43", "");
    assert_eq!(
        outcome,
        pipeline_wait::WaitOutcome::Failed {
            checks: 41,
            total: 43
        }
    );
}

/// Ticket wai-63h9 task 1.3: a PR with zero checks is a terminal pass —
/// exit 0 and a summary that says so explicitly, so waiting on a PR with no
/// CI configured does not spin until timeout.
#[test]
fn pipeline_wait_zero_checks_terminal_pass() {
    let outcome = pipeline_wait::WaitOutcome::ZeroChecks;
    assert_eq!(
        pipeline_wait::exit_code(&outcome),
        0,
        "zero checks must map to exit code 0 (terminal pass)"
    );
    let line = pipeline_wait::summary_line(456, &outcome);
    assert!(
        line.to_lowercase().contains("zero") || line.to_lowercase().contains("no checks"),
        "zero-checks summary must say there were no checks, line: {line}"
    );
    assert!(
        line.contains("456"),
        "zero-checks summary must still name the PR, line: {line}"
    );
}
