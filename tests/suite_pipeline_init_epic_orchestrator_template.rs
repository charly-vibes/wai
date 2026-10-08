//! Tests for `wai pipeline init epic-orchestrator` — the built-in
//! epic-orchestrator template (orchestrator + subagents pattern, canon:
//! orchestrator-subagents.md Invariant 8 "prose does not enforce").

#![allow(clippy::too_many_lines)]

mod common;

use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn pipeline_init_epic_orchestrator_scaffolds_template() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let toml_path = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    fs::metadata(&toml_path).expect("pipeline init epic-orchestrator should scaffold the template");
}

#[test]
fn pipeline_init_epic_orchestrator_declares_orchestrator_loop_steps() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let toml_path = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // The orchestrator loop: claim → gates → brief → spawn → verify → ship → STOP
    for expected in [
        "id = \"claim\"",
        "id = \"gates\"",
        "id = \"brief\"",
        "id = \"spawn\"",
        "id = \"verify\"",
        "id = \"ship\"",
        "id = \"stop\"",
    ] {
        assert!(
            content.contains(expected),
            "Expected {expected} step in TOML: {content}"
        );
    }
}

#[test]
fn pipeline_init_epic_orchestrator_cites_canon_pattern() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let toml_path = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Header must cite the canon pattern and its tool-agnostic invariants.
    assert!(
        content.contains("orchestrator-subagents"),
        "Header must cite canon orchestrator-subagents.md: {content}"
    );
    assert!(
        content.contains("Invariant 8") || content.contains("prose does not enforce"),
        "Header must cite Invariant 8 (prose does not enforce): {content}"
    );
}

#[test]
fn pipeline_init_epic_orchestrator_toml_is_valid() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let toml_path = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    let content = fs::read_to_string(&toml_path).unwrap();
    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Generated TOML should be valid, but got error: {:?}",
        parsed.err()
    );
}

#[test]
fn pipeline_init_epic_orchestrator_encodes_run_state_durability() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let toml_path = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Claim step writes the durable per-ticket state file with its fixed shape.
    assert!(
        content.contains(".wai/projects/<project>/runs/<ticket>.state"),
        "Claim step must write the per-ticket state file path: {content}"
    );
    for field in ["ticket id", "branch", "brief path", "empty step history"] {
        assert!(
            content.contains(field),
            "State file shape must include {field:?}: {content}"
        );
    }

    // Every step boundary appends step id + sha to the state file.
    assert!(
        content.contains("step id + sha"),
        "Step-boundary state append must be encoded: {content}"
    );
    assert!(
        content.contains("branch head at verify entry"),
        "Verify step must record the branch head at entry: {content}"
    );

    // Spawn refuses to run without the state file.
    assert!(
        content.contains("refuse") && content.contains("without the state file"),
        "Spawn step must refuse without the state file: {content}"
    );

    // Retry-entry drift check: refuse resume when HEAD advanced without a
    // completed step accounting for it; mid-run advances are expected, never
    // flagged; retry resumes from the last completed gate.
    assert!(
        content.contains("drift"),
        "Retry-entry drift check must be encoded: {content}"
    );
    assert!(
        content.contains("mid-run advance is EXPECTED"),
        "Mid-run advance must be explicitly expected, never flagged: {content}"
    );
    assert!(
        content.contains("resumes from the last completed gate"),
        "Retry must resume from the last completed gate: {content}"
    );
}

#[test]
fn pipeline_init_epic_orchestrator_encodes_brief_format() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let toml_path = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Brief step mandates the three brief sections.
    for section in ["## Why", "## Completion criteria", "## Spawn model"] {
        assert!(
            content.contains(section),
            "Brief step must mandate section {section:?}: {content}"
        );
    }

    // Completion criteria are runnable commands: exit 0 = done.
    assert!(
        content.contains("exit 0"),
        "Completion criteria must be stated as runnable commands where exit 0 = done: {content}"
    );

    // Spawn model: context ceiling as a % of the actual model window,
    // computed at spawn; fixed token counts are flagged invalid.
    assert!(
        content.contains('%'),
        "Spawn model must express the context ceiling as a percentage: {content}"
    );
    assert!(
        content.contains("computed at spawn"),
        "Context ceiling must be computed at spawn: {content}"
    );
    assert!(
        content.contains("Fixed token counts") && content.contains("invalid"),
        "Fixed token counts must be flagged invalid: {content}"
    );

    // Spawn step refuses without a committed brief.
    assert!(
        content.contains("without a committed brief"),
        "Spawn step must refuse to spawn without a committed brief: {content}"
    );
}

#[test]
fn pipeline_help_lists_epic_orchestrator_builtin() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "--help"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("epic-orchestrator"),
        "pipeline help should list the epic-orchestrator built-in template: {stdout}"
    );
}
