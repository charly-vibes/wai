// Library interface for the `wai pipeline wait` primitive (ticket wai-63h9).
//! Purpose: pure wait-outcome logic for the canonical blocking wait
//! primitive — exit-code mapping and one-line summary formatting.
//! Responsibilities: map a terminal wait outcome to a process exit code and
//! format exactly one human-readable summary line that names the PR, the
//! terminal state, and the check counts, so agents never re-poll to
//! reconstruct what happened. Rationale: week 2026-10-13..19 measured 539
//! `sleep >=60s` poll commands (~53h blocked wall-clock); the primitive
//! replaces improvised poll loops, and its summary is the single artifact
//! the waiting agent consumes. Declared in both the lib and the bin
//! (pattern: `matrix`, `plugin`) so integration tests reach it via
//! `wai::pipeline_wait` while `cmd_wait` (gh supervision) stays bin-side in
//! `src/commands/pipeline/orchestration.rs`.

/// Terminal outcome of a `wai pipeline wait` invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaitOutcome {
    /// All checks passed (`checks` passing of `total` total).
    Passed { checks: usize, total: usize },
    /// At least one check failed (`checks` passing of `total` total).
    Failed { checks: usize, total: usize },
    /// The time budget was exhausted before a terminal state; `elapsed` is
    /// the human-readable budget and `last_state` the last observed state
    /// from the single post-timeout re-query.
    Timeout { elapsed: String, last_state: String },
    /// The PR has zero checks — a terminal pass, so waiting on a PR with no
    /// CI configured does not spin until timeout.
    ZeroChecks,
}

/// Map a terminal wait outcome to the process exit code: pass (including
/// zero-checks) → 0; check failure or timeout exhaustion → nonzero.
pub fn exit_code(outcome: &WaitOutcome) -> i32 {
    match outcome {
        WaitOutcome::Passed { .. } | WaitOutcome::ZeroChecks => 0,
        WaitOutcome::Failed { .. } | WaitOutcome::Timeout { .. } => 1,
    }
}

/// Format the ONE summary line for the outcome. Every branch names the PR
/// and the terminal state; pass/fail name the counts (omitted when the
/// total could not be parsed, so no misleading `(0/0)` is ever rendered);
/// timeout names the elapsed budget and the last observed state. Never
/// multi-line — the waiting agent consumes exactly one line. `pr` is any
/// Display value so branch-name PRs render verbatim instead of as `PR 0`.
pub fn summary_line(pr: impl std::fmt::Display, outcome: &WaitOutcome) -> String {
    match outcome {
        WaitOutcome::Passed { total: 0, .. } => format!("PR {pr}: all checks pass"),
        WaitOutcome::Passed { checks, total } => {
            format!("PR {pr}: all checks pass ({checks}/{total})")
        }
        WaitOutcome::Failed { total: 0, .. } => format!("PR {pr}: checks FAILED"),
        WaitOutcome::Failed { checks, total } => {
            format!("PR {pr}: checks FAILED ({checks}/{total} passing)")
        }
        WaitOutcome::Timeout {
            elapsed,
            last_state,
        } => {
            format!("PR {pr}: TIMEOUT after {elapsed} — last observed state: {last_state}")
        }
        WaitOutcome::ZeroChecks => format!("PR {pr}: zero checks reported — terminal pass"),
    }
}

/// Extract the trailing `passing/total` counts from `gh pr checks` output.
/// Scans for the last `N/M` token; returns (0, 0) when the output carries
/// no such summary (summary_line then degrades gracefully).
pub fn parse_counts(stdout: &str) -> (usize, usize) {
    let mut found = (0usize, 0usize);
    for token in stdout.split_whitespace() {
        if let Some((passing, total)) = token.split_once('/')
            && let (Ok(p), Ok(t)) = (passing.parse::<usize>(), total.parse::<usize>())
        {
            found = (p, t);
        }
    }
    found
}

/// Classify a completed `gh pr checks --watch` exit into a terminal outcome.
/// The zero-checks case is detected FIRST, regardless of exit code, because
/// gh reports a PR with no CI with a "no checks" message whose exit code
/// varies by version — and a no-CI PR is a terminal pass either way.
/// Exit 0 → Passed; other nonzero exits → Failed.
pub fn classify(code: Option<i32>, stdout: &str, stderr: &str) -> WaitOutcome {
    let combined = format!("{stdout}{stderr}");
    if combined.to_lowercase().contains("no checks") {
        return WaitOutcome::ZeroChecks;
    }
    if code == Some(0) {
        let (checks, total) = parse_counts(stdout);
        return WaitOutcome::Passed { checks, total };
    }
    let (checks, total) = parse_counts(stdout);
    WaitOutcome::Failed { checks, total }
}
