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
