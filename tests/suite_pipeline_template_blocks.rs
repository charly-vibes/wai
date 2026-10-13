//! Integration tests for the pipeline template shared-block mechanism
//! (ticket: wai-vsn6).
//!
//! `[[blocks]]` entries in a pipeline TOML define reusable content that
//! step prompts reference as `{{block:<name>}}`. Resolution happens at
//! load time and must not change rendered output for templates that
//! don't use it (see the byte-identity ratchet in
//! `src/commands/pipeline/mod.rs`). Unknown references fail loudly.

mod common;

use common::*;
use std::fs;
use tempfile::TempDir;

/// `wai pipeline validate` must pass for every built-in template after
/// the shared-block mechanism exists (ratchet test 3c).
#[test]
fn validate_passes_for_all_builtin_templates() {
    for name in ["epic-orchestrator", "scientific-research", "tdd-ro5"] {
        let tmp = TempDir::new().unwrap();
        init_workspace(tmp.path());

        let init = wai_cmd(tmp.path())
            .args(["pipeline", "init", name])
            .output()
            .expect("init should run");
        assert!(init.status.success(), "init {name} failed");

        let out = wai_cmd(tmp.path())
            .args(["pipeline", "validate"])
            .output()
            .expect("validate should run");
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.status.success(),
            "validate should pass for {name}, output: {combined}"
        );
        assert!(
            combined.contains(name),
            "validate output should mention {name}: {combined}"
        );
    }
}

/// A pipeline TOML referencing an undefined block must fail validation
/// with a clear error naming the missing block (ratchet test 3b).
#[test]
fn validate_fails_on_unknown_block_reference() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let toml_path = tmp.path().join(".wai/resources/pipelines/badblocks.toml");
    fs::create_dir_all(toml_path.parent().unwrap()).unwrap();
    fs::write(
        &toml_path,
        r#"
[pipeline]
name = "badblocks"
description = "References an undefined shared block"

[[steps]]
id = "step1"
prompt = """
Do the thing.
{{block:undefined-block}}
"""
"#,
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "validate", "badblocks"])
        .output()
        .expect("validate should run");
    assert!(
        !out.status.success(),
        "validate must fail on unknown block reference"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("undefined-block"),
        "error must name the missing block: {combined}"
    );
    assert!(
        combined.contains("unknown block"),
        "error must say 'unknown block': {combined}"
    );
}

/// A pipeline TOML with a well-formed `[[blocks]]` table and a reference
/// must validate cleanly (the mechanism must not be flagged as an error).
#[test]
fn validate_passes_with_defined_blocks() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let toml_path = tmp.path().join(".wai/resources/pipelines/goodblocks.toml");
    fs::create_dir_all(toml_path.parent().unwrap()).unwrap();
    fs::write(
        &toml_path,
        r#"
[pipeline]
name = "goodblocks"
description = "Uses a defined shared block"

[[steps]]
id = "step1"
prompt = """
Do the thing.
{{block:trail}}
"""

[[blocks]]
name = "trail"
content = "Advance: `wai pipeline next`"
"#,
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "validate", "goodblocks"])
        .output()
        .expect("validate should run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "validate should pass with defined blocks, output: {combined}"
    );
}

/// Steps that produce implementation or shipping evidence must declare a
/// structural artifact-manifest gate (ticket: wai-rhjp). The 2026-10-05
/// catastrophic RED violation (wai-vx02.3) saw a run advance past
/// refactor-or-tidy with zero GREEN artifacts because those steps declared
/// only an oracle gate — any step-tagged artifact from an earlier step is
/// not enough; each step must require its own artifact.
#[test]
fn implement_and_ship_steps_declare_structural_gates() {
    let cases: &[(&str, &[&str])] = &[
        ("epic-orchestrator", &["ship"]),
        ("tdd-ro5", &["green", "refactor", "ship-close"]),
    ];
    for (template, steps) in cases {
        let tmp = TempDir::new().unwrap();
        init_workspace(tmp.path());

        let init = wai_cmd(tmp.path())
            .args(["pipeline", "init", template])
            .output()
            .expect("init should run");
        assert!(init.status.success(), "init {template} failed");

        let out = wai_cmd(tmp.path())
            .args(["pipeline", "gates", template])
            .output()
            .expect("gates should run");
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.status.success(), "gates {template} failed: {combined}");
        let text = strip_ansi(&combined);

        // Split the gates output into per-step sections. The stripped text
        // keeps the trailing 'm' of ANSI SGR sequences (see strip_ansi), so
        // step ids appear as "Step: m<id>m" — compare id tokens with the
        // wrapper characters trimmed.
        let step_starts: Vec<usize> = text.match_indices("Step: ").map(|(i, _)| i).collect();
        for step in *steps {
            let start = step_starts
                .iter()
                .copied()
                .find(|&i| {
                    let token: String = text[i + "Step: ".len()..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches('m')
                        .to_string();
                    token == *step
                })
                .unwrap_or_else(|| panic!("gates output missing step {step}: {text}"));
            let end = step_starts
                .iter()
                .copied()
                .find(|&i| i > start)
                .unwrap_or(text.len());
            let section = &text[start..end];
            assert!(
                section.contains("Structural:"),
                "{template} step {step} must declare a structural artifact gate; \
                 section:\n{section}"
            );
        }
    }
}
