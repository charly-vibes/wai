//! Integration tests for the verifier evidence contract (ticket: wai-px4g,
//! R3 of harden-orchestration-gates).
//!
//! The verify step's review artifact may only record CONFIRM when it cites
//! the artifacts it read and includes fresh test-run output. Evidence gaps
//! must resolve to REVIEW_REQUIRED, and single-model degradation is always
//! REVIEW_REQUIRED. Enforced by the bundled `verify-evidence` oracle shipped
//! with the epic-orchestrator template.

mod common;

use common::*;
use std::fs;
use tempfile::TempDir;

/// The verify-evidence oracle must be scaffolded by pipeline init for
/// epic-orchestrator and enforce the evidence contract on review artifacts.
#[test]
fn verify_evidence_oracle_scaffolded_and_enforces_contract() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let init = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("init should run");
    assert!(init.status.success(), "init failed");

    let oracle = tmp.path().join(".wai/resources/oracles/verify-evidence.sh");
    assert!(
        oracle.exists(),
        "epic-orchestrator init must scaffold verify-evidence.sh"
    );

    let tmp_art = tmp.path().join("review-artifact.md");

    // 1. Evidence-less CONFIRM must fail the oracle.
    fs::write(
        &tmp_art,
        "## Verdict\n\nCONFIRM — implementor report matches git log and beads state.\n",
    )
    .unwrap();
    let out = std::process::Command::new(&oracle)
        .arg(&tmp_art)
        .output()
        .expect("oracle should run");
    assert!(
        !out.status.success(),
        "CONFIRM without artifact citations and fresh test-run output must fail"
    );

    // 2. CONFIRM with citations + fresh test-run output must pass.
    fs::write(
        &tmp_art,
        "## Reads\n\n- read .wai/projects/p/research/2026-10-13-red-x.md\n- read .wai/projects/p/designs/2026-10-13-green-x.md\n\n## Fresh test run\n\ncargo test --test suite_x\ntest result: ok. 12 passed; 0 failed\n\n## Verdict\n\nCONFIRM\n",
    )
    .unwrap();
    let out = std::process::Command::new(&oracle)
        .arg(&tmp_art)
        .output()
        .expect("oracle should run");
    assert!(
        out.status.success(),
        "CONFIRM with artifact citations and fresh test-run output must pass: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // 3. REVIEW_REQUIRED always passes — honest no-evidence or single-model
    // degradation is a valid, non-blocking verdict.
    fs::write(
        &tmp_art,
        "## Verdict\n\nREVIEW_REQUIRED — single-model degradation: verifier ran on the same model family as the implementor.\n",
    )
    .unwrap();
    let out = std::process::Command::new(&oracle)
        .arg(&tmp_art)
        .output()
        .expect("oracle should run");
    assert!(
        out.status.success(),
        "REVIEW_REQUIRED must pass the evidence oracle: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The epic-orchestrator verify step must declare the verify-evidence oracle
/// in its gate output.
#[test]
fn epic_orchestrator_verify_step_declares_verify_evidence_oracle() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let init = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("init should run");
    assert!(init.status.success(), "init failed");

    let out = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "gates", "epic-orchestrator"])
        .output()
        .expect("gates should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "gates failed: {combined}");
    let text = strip_ansi(&combined);
    let verify_idx = text
        .find("verify")
        .unwrap_or_else(|| panic!("gates output missing verify step: {text}"));
    let section_end = text[verify_idx..]
        .find("Step:")
        .map(|e| verify_idx + e)
        .unwrap_or(text.len());
    let section = &text[verify_idx..section_end];
    assert!(
        section.contains("verify-evidence"),
        "verify step must declare the verify-evidence oracle; section:\n{section}"
    );
}

/// Prose mentioning "confirmation" must not read as a CONFIRM verdict
/// (RO5U pass-1 medium finding: word-boundary verdict detection).
#[test]
fn oracle_does_not_misread_confirmation_prose_as_confirm() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("init should run");
    let oracle = tmp.path().join(".wai/resources/oracles/verify-evidence.sh");
    let art = tmp.path().join("review-artifact.md");
    fs::write(
        &art,
        "Awaiting confirmation from the lead before the verdict step; no verdict recorded yet.\n",
    )
    .unwrap();
    let out = std::process::Command::new(&oracle)
        .arg(&art)
        .output()
        .expect("oracle should run");
    assert!(
        !out.status.success(),
        "prose 'confirmation' must not satisfy the CONFIRM check"
    );
}
