//! Integration tests for `wai add` pipeline-step tag correspondence with the
//! active run state (ticket: wai-e8tc).
//!
//! Symptom under investigation: after `wai pipeline next` advances the run,
//! the first `wai add <type>` allegedly tagged artifacts with the PREVIOUS
//! step id, so the new step's structural gate found 0 artifacts and blocked
//! advancement until manual retagging.
//!
//! Empirical result: the sequential CLI path (`start` → `next` → `add`) tags
//! correctly on both the current build and the session-era binary — the RED
//! attempt could not be made to fail. These tests therefore pin the
//! correspondence as a regression ratchet:
//!
//! 1. `wai add` after `wai pipeline next` must tag the artifact with the
//!    exact step id `wai pipeline current --json` reports (the GREEN-state
//!    contract; any future drift in either resolution path fails here).
//! 2. `wai add` BEFORE `wai pipeline next` tags with the then-current step —
//!    documented correct behavior that explains the observed "stale" tags
//!    under operator sequencing (artifact added, then the run advanced).

mod common;

use common::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Scaffold a temp workspace with the epic-orchestrator pipeline initialized
/// and a run started for `topic`. Returns (temp dir, run id).
fn scaffold_pipeline_run(topic: &str) -> (TempDir, String) {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "proj");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("pipeline init should run");
    assert!(out.status.success(), "pipeline init failed: {out:?}");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "epic-orchestrator", "--topic", topic])
        .output()
        .expect("pipeline start should run");
    assert!(out.status.success(), "pipeline start failed: {out:?}");

    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();
    (tmp, run_id)
}

/// Frontmatter of the research artifact whose body contains `marker`.
fn research_frontmatter(dir: &std::path::Path, marker: &str) -> String {
    let research = dir.join(".wai/projects/proj/research");
    let files: Vec<_> = fs::read_dir(&research)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert!(!files.is_empty(), "expected at least one research artifact");
    let mut matches = files
        .into_iter()
        .map(|p| fs::read_to_string(&p).unwrap())
        .filter(|c| c.contains(marker))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "marker must match exactly one artifact");
    matches.pop().unwrap()
}

/// Step id reported by `wai pipeline current --json` for the active run.
fn current_step_id(dir: &std::path::Path) -> String {
    let out = wai_cmd(dir)
        .args(["pipeline", "current", "--json"])
        .output()
        .expect("pipeline current should run");
    assert!(out.status.success(), "pipeline current failed: {out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).expect("valid JSON envelope");
    json["data"]["step"]["id"]
        .as_str()
        .expect("step.id present")
        .to_string()
}

/// `wai add` immediately after an advance must tag the artifact with the
/// same step id `wai pipeline current` reports — no lag, no off-by-one.
#[test]
fn add_after_next_tags_artifact_with_current_step() {
    let (tmp, _run_id) = scaffold_pipeline_run("repro");
    let dir = tmp.path();

    // Advance past claim: the claim step's structural gate needs one research
    // artifact tagged with the claim step.
    let out = wai_cmd(dir)
        .args(["add", "research", "ORIENT: selected ticket"])
        .output()
        .expect("add should run");
    assert!(out.status.success(), "add failed: {out:?}");
    let fm = research_frontmatter(dir, "ORIENT: selected ticket");
    assert!(
        fm.contains("pipeline-step:claim"),
        "artifact added at step 0 must be tagged claim, frontmatter: {fm}"
    );

    let out = wai_cmd(dir)
        .args(["pipeline", "next"])
        .output()
        .expect("pipeline next should run");
    assert!(out.status.success(), "pipeline next failed: {out:?}");

    // The new current step, as `pipeline current` reports it.
    let step_id = current_step_id(dir);
    assert_eq!(step_id, "gates", "run must have advanced to gates");

    // THE CONTRACT: the first add after the advance tags the CURRENT step.
    let out = wai_cmd(dir)
        .args(["add", "research", "GATES: baseline green"])
        .output()
        .expect("add should run");
    assert!(out.status.success(), "add failed: {out:?}");
    let fm = research_frontmatter(dir, "GATES: baseline green");
    assert!(
        fm.contains(&format!("pipeline-step:{step_id}")),
        "artifact added after next must be tagged '{step_id}' (what `pipeline \
         current` reports), frontmatter: {fm}"
    );
}

/// `wai add` before the advancing `pipeline next` tags with the THEN-current
/// step. This is correct behavior, not a bug — it documents why artifacts
/// recorded ahead of the advance carry the previous step id under operator
/// sequencing (the observed evidence pattern in wai-e8tc).
#[test]
fn add_before_next_tags_artifact_with_then_current_step() {
    let (tmp, _run_id) = scaffold_pipeline_run("repro");
    let dir = tmp.path();

    // Record a gates-style artifact while still at step 0 (claim)...
    let out = wai_cmd(dir)
        .args(["add", "research", "GATES: recorded before advancing"])
        .output()
        .expect("add should run");
    assert!(out.status.success(), "add failed: {out:?}");
    let before = research_frontmatter(dir, "recorded before advancing");
    assert!(
        before.contains("pipeline-step:claim"),
        "artifact added before next must be tagged with the then-current step \
         (claim), frontmatter: {before}"
    );

    // ...then advance. The earlier artifact's tag is intentionally unchanged.
    let out = wai_cmd(dir)
        .args(["pipeline", "next"])
        .output()
        .expect("pipeline next should run");
    assert!(out.status.success(), "pipeline next failed: {out:?}");
    let fm = research_frontmatter(dir, "recorded before advancing");
    assert!(
        fm.contains("pipeline-step:claim"),
        "tags are immutable at add time; advancing must not retag, \
         frontmatter: {fm}"
    );
    assert_eq!(
        current_step_id(dir),
        "gates",
        "run advanced to gates while the artifact stays tagged claim"
    );
}

/// Sanity: the run state yml write ordering — `pipeline next` persists the
/// advanced `current_step` before printing the new step prompt — so any
/// process that observed the new step prompt can rely on the yml being
/// current.
#[test]
fn next_persists_run_state_before_printing_new_step_prompt() {
    let (tmp, run_id) = scaffold_pipeline_run("repro");
    let dir = tmp.path();

    let out = wai_cmd(dir)
        .args(["add", "research", "ORIENT: selected ticket"])
        .output()
        .expect("add should run");
    assert!(out.status.success(), "add failed: {out:?}");

    let out = wai_cmd(dir)
        .args(["pipeline", "next"])
        .output()
        .expect("pipeline next should run");
    assert!(out.status.success(), "pipeline next failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("gates"),
        "next must print the new step prompt, stdout: {stdout}"
    );

    // Yml on disk already reflects the advance.
    let yml = fs::read_to_string(dir.join(format!(".wai/pipeline-runs/{run_id}.yml"))).unwrap();
    assert!(
        yml.contains("current_step: 1"),
        "run state must be persisted before the prompt is printed, yml: {yml}"
    );
}
