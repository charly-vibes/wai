//! Unit tests for coverage gates, oracle resolution, and built-in templates.

use super::tests_definition::write_toml;
use crate::commands::pipeline::definition::load_pipeline_toml;
use crate::commands::pipeline::gates::{
    evaluate_gates, execute_oracle, format_gate_summary, resolve_oracle_command,
};
use crate::commands::pipeline::model::CoverageGate;
use crate::commands::pipeline::setup::{builtin_template_names, get_builtin_template};
use crate::commands::pipeline::{PipelineDefinition, PipelineRun, PipelineStep, StepGate};
use std::fs;

use std::collections::HashMap;

// ── coverage gate tests ───────────────────────────────────────────────

#[test]
fn coverage_gate_toml_backward_compat() {
    // TOML without [steps.gate.coverage] should still parse correctly
    let toml = r#"
[pipeline]
name = "no-coverage"

[[steps]]
id = "step1"
prompt = "Do something."

[steps.gate.structural]
min_artifacts = 1
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse without coverage gate");
    let gate = def.steps[0].gate.as_ref().expect("should have gate");
    assert!(
        gate.coverage.is_none(),
        "coverage should be None when omitted"
    );
    assert!(gate.structural.is_some());
}

#[test]
fn coverage_gate_toml_parses() {
    let toml = r#"
[pipeline]
name = "with-coverage"

[[steps]]
id = "generate"
prompt = "Generate {topic}."

[steps.gate.coverage]
require_input_manifest = true
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse coverage gate");
    let gate = def.steps[0].gate.as_ref().expect("should have gate");
    let cg = gate
        .coverage
        .as_ref()
        .expect("coverage gate should be present");
    assert!(cg.require_input_manifest);
}

#[test]
fn format_gate_summary_coverage_only() {
    let gate = StepGate {
        coverage: Some(CoverageGate {
            require_input_manifest: true,
        }),
        ..Default::default()
    };
    assert_eq!(format_gate_summary(&Some(gate)), "[coverage]");
}

// ── coverage gate evaluation tests ─────────────────────────────────────

/// Helper to build a standard test scaffold for coverage gate tests.
fn coverage_gate_scaffold(
    require: bool,
) -> (StepGate, PipelineStep, PipelineRun, PipelineDefinition) {
    let gate = StepGate {
        coverage: Some(CoverageGate {
            require_input_manifest: require,
        }),
        ..Default::default()
    };
    let step = PipelineStep {
        id: "gen".to_string(),
        prompt: "test".to_string(),
        gate: Some(gate.clone()),
        lock: false,
    };
    let run = PipelineRun {
        run_id: "test-run".to_string(),
        pipeline: "test".to_string(),
        topic: "topic".to_string(),
        created_at: "2026-04-02T00:00:00Z".to_string(),
        current_step: 0,
        approvals: HashMap::new(),
        epic: None,
        child_issues: Vec::new(),
        child_runs: Vec::new(),
        handoff_artifact: None,
        bypasses: Vec::new(),
    };
    let def = PipelineDefinition {
        name: "test".to_string(),
        description: None,
        steps: vec![step.clone()],
        metadata: None,
    };
    (gate, step, run, def)
}

#[test]
fn coverage_gate_passes_with_manifest() {
    let (gate, step, run, def) = coverage_gate_scaffold(true);
    let dir = tempfile::tempdir().unwrap();

    // Create the project structure with a coverage manifest
    let reviews_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project")
        .join("reviews");
    fs::create_dir_all(&reviews_dir).unwrap();
    fs::write(
        reviews_dir.join("coverage-manifest.md"),
        "---\ntags: [coverage-manifest:gen]\n---\n\nManifest body\n",
    )
    .unwrap();

    let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
    assert!(
        failures.is_empty(),
        "expected no failures when manifest exists, got: {:?}",
        failures
    );
}

#[test]
fn coverage_gate_fails_without_manifest() {
    let (gate, step, run, def) = coverage_gate_scaffold(true);
    let dir = tempfile::tempdir().unwrap();

    // Create the project structure but no manifest
    let reviews_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project")
        .join("reviews");
    fs::create_dir_all(&reviews_dir).unwrap();

    let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
    assert!(!failures.is_empty(), "expected failure when no manifest");
    assert!(
        failures[0].contains("Coverage gate not satisfied"),
        "got: {}",
        failures[0]
    );
    assert!(
        failures[0].contains("coverage-manifest:gen"),
        "message should include expected tag, got: {}",
        failures[0]
    );
}

#[test]
fn coverage_gate_skipped_when_not_required() {
    let (gate, step, run, def) = coverage_gate_scaffold(false);
    let dir = tempfile::tempdir().unwrap();

    // No manifest, but gate doesn't require one
    let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
    assert!(
        failures.is_empty(),
        "expected no failures when require_input_manifest is false, got: {:?}",
        failures
    );
}

// ── oracle resolution tests ───────────────────────────────────────────

#[test]
fn resolve_oracle_exact_name() {
    let dir = tempfile::tempdir().unwrap();
    let wai = dir.path().join(".wai").join("resources").join("oracles");
    fs::create_dir_all(&wai).unwrap();
    fs::write(wai.join("check"), "#!/bin/sh\nexit 0\n").unwrap();
    let result = resolve_oracle_command("check", dir.path());
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert!(result.unwrap().contains("check"));
}

#[test]
fn resolve_oracle_sh_extension() {
    let dir = tempfile::tempdir().unwrap();
    let wai = dir.path().join(".wai").join("resources").join("oracles");
    fs::create_dir_all(&wai).unwrap();
    fs::write(wai.join("check.sh"), "#!/bin/sh\nexit 0\n").unwrap();
    let result = resolve_oracle_command("check", dir.path());
    assert!(result.is_ok());
    assert!(result.unwrap().contains("check.sh"));
}

#[test]
fn resolve_oracle_py_extension() {
    let dir = tempfile::tempdir().unwrap();
    let wai = dir.path().join(".wai").join("resources").join("oracles");
    fs::create_dir_all(&wai).unwrap();
    fs::write(wai.join("check.py"), "#!/usr/bin/env python3\n").unwrap();
    let result = resolve_oracle_command("check", dir.path());
    assert!(result.is_ok());
    assert!(result.unwrap().contains("check.py"));
}

#[test]
fn resolve_oracle_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let wai = dir.path().join(".wai").join("resources").join("oracles");
    fs::create_dir_all(&wai).unwrap();
    let result = resolve_oracle_command("nonexistent", dir.path());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("not found"), "got: {msg}");
}

#[test]
fn resolve_oracle_probes_in_order() {
    // When both .sh and .py exist, .sh should win (probed first)
    let dir = tempfile::tempdir().unwrap();
    let wai = dir.path().join(".wai").join("resources").join("oracles");
    fs::create_dir_all(&wai).unwrap();
    fs::write(wai.join("check.sh"), "#!/bin/sh\n").unwrap();
    fs::write(wai.join("check.py"), "#!/usr/bin/env python3\n").unwrap();
    let result = resolve_oracle_command("check", dir.path()).unwrap();
    assert!(
        result.contains("check.sh"),
        "expected .sh to win probe order, got: {result}"
    );
}

#[test]
fn execute_oracle_success() {
    let result = execute_oracle("true", &[], 30).unwrap();
    assert!(
        result.is_none(),
        "expected None (success), got: {:?}",
        result
    );
}

#[test]
fn execute_oracle_failure() {
    let result = execute_oracle("false", &[], 30).unwrap();
    assert!(result.is_some(), "expected Some (failure)");
}

// ── built-in templates ────────────────────────────────────────────────

#[test]
fn builtin_template_scientific_research_exists() {
    let template = get_builtin_template("scientific-research");
    assert!(template.is_some(), "expected scientific-research template");
    let content = template.unwrap();
    assert!(content.contains("[pipeline]"));
    assert!(content.contains("scientific-research"));
    assert!(content.contains("[pipeline.metadata]"));
}

#[test]
fn builtin_template_tdd_ro5_exists() {
    let template = get_builtin_template("tdd-ro5");
    assert!(template.is_some(), "expected tdd-ro5 template");
    let content = template.unwrap();
    assert!(content.contains("[pipeline]"));
    assert!(content.contains("tdd-ro5"));
    assert!(content.contains("[pipeline.metadata]"));
}

#[test]
fn builtin_template_tdd_ro5_has_autonomous_ro5u_steps() {
    let content = get_builtin_template("tdd-ro5").unwrap();
    let f = write_toml(content);
    let def = load_pipeline_toml(f.path()).expect("should parse tdd-ro5 template");
    assert_eq!(def.steps.len(), 9);
    let ids: Vec<&str> = def.steps.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "orient",
            "plan",
            "red",
            "green",
            "refactor",
            "ro5u-review",
            "fix-review",
            "quality-ledger",
            "ship-close"
        ]
    );
}

#[test]
fn builtin_template_tdd_ro5_has_autonomous_gates() {
    let content = get_builtin_template("tdd-ro5").unwrap();
    let f = write_toml(content);
    let def = load_pipeline_toml(f.path()).expect("should parse tdd-ro5 template");
    // orient and plan steps require artifacts
    assert!(def.steps[0].gate.is_some());
    assert!(def.steps[1].gate.is_some());
    // ro5u-review step has procedural review gate
    let review_gate = def.steps[5].gate.as_ref().unwrap();
    assert!(review_gate.procedural.is_some());
    // green/refactor/fix-review/ship-close steps have oracle gates
    for idx in [3, 4, 6, 8] {
        let gate = def.steps[idx].gate.as_ref().unwrap();
        assert!(
            !gate.oracles.is_empty(),
            "step {} should have oracle gate",
            def.steps[idx].id
        );
    }
}

#[test]
fn builtin_template_unknown_returns_none() {
    assert!(get_builtin_template("nonexistent").is_none());
}

#[test]
fn builtin_template_names_not_empty() {
    let names = builtin_template_names();
    assert!(!names.is_empty());
    assert!(names.contains(&"scientific-research"));
    assert!(names.contains(&"tdd-ro5"));
}

#[test]
fn all_builtin_template_names_resolve_to_templates() {
    for name in builtin_template_names() {
        assert!(
            get_builtin_template(name).is_some(),
            "built-in template name '{name}' should resolve"
        );
    }
}
