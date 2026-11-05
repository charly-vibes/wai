//! Unit tests for gate evaluation, metadata parsing, and validate_pipeline.

use crate::commands::pipeline::definition::{load_pipeline_toml, validate_pipeline};
use crate::commands::pipeline::gates::format_gate_summary;
use crate::commands::pipeline::gates::evaluate_gates;

use crate::commands::pipeline::model::CoverageGate;
use crate::commands::pipeline::model::{
    ApprovalGate, OracleGate, PipelineDefinition, PipelineMetadataSection, PipelineRun,
    ProceduralGate, StructuralGate, ValidationLevel,
};
use crate::commands::pipeline::{PipelineStep, StepGate};
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

fn write_toml(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create tempfile");
    f.write_all(content.as_bytes()).expect("write toml");
    f
}

// ── gate evaluation ──────────────────────────────────────────────────

#[test]
fn structural_gate_fails_on_missing_artifacts() {
    let gate = StepGate {
        structural: Some(StructuralGate {
            min_artifacts: 1,
            types: vec!["research".to_string()],
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
    let dir = tempfile::tempdir().unwrap();
    let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
    assert!(!failures.is_empty());
    assert!(
        failures[0].contains("requires at least 1"),
        "got: {}",
        failures[0]
    );
}

#[test]
fn approval_gate_fails_when_not_approved() {
    let gate = StepGate {
        approval: Some(ApprovalGate {
            required: true,
            message: Some("Please review.".to_string()),
        }),
        ..Default::default()
    };
    let step = PipelineStep {
        id: "review-step".to_string(),
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
    let dir = tempfile::tempdir().unwrap();
    let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
    assert!(!failures.is_empty());
    assert!(
        failures[0].contains("Please review."),
        "got: {}",
        failures[0]
    );
}

#[test]
fn approval_gate_passes_when_approved() {
    let gate = StepGate {
        approval: Some(ApprovalGate {
            required: true,
            message: None,
        }),
        ..Default::default()
    };
    let step = PipelineStep {
        id: "review-step".to_string(),
        prompt: "test".to_string(),
        gate: Some(gate.clone()),
        lock: false,
    };
    let mut approvals = HashMap::new();
    approvals.insert(
        "review-step".to_string(),
        "2099-01-01T00:00:00Z".to_string(),
    );
    let run = PipelineRun {
        run_id: "test-run".to_string(),
        pipeline: "test".to_string(),
        topic: "topic".to_string(),
        created_at: "2026-04-02T00:00:00Z".to_string(),
        current_step: 0,
        approvals,
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
    let dir = tempfile::tempdir().unwrap();
    let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
    assert!(
        failures.is_empty(),
        "expected no failures, got: {:?}",
        failures
    );
}

// ── pipeline metadata parsing ──────────────────────────────────────

#[test]
fn load_pipeline_toml_with_metadata() {
    let toml = r#"
[pipeline]
name = "research"
description = "Research workflow"

[pipeline.metadata]
when = "Frontier-level research requiring systematic validation"
skills = ["design-practice", "ro5"]

[[steps]]
id = "start"
prompt = "Start {topic}."
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse pipeline with metadata");
    let meta = def.metadata.as_ref().expect("should have metadata");
    assert_eq!(
        meta.when.as_deref(),
        Some("Frontier-level research requiring systematic validation")
    );
    assert_eq!(meta.skills, vec!["design-practice", "ro5"]);
}

#[test]
fn load_pipeline_toml_without_metadata() {
    let toml = r#"
[pipeline]
name = "simple"

[[steps]]
id = "go"
prompt = "Do {topic}."
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse pipeline without metadata");
    assert!(def.metadata.is_none());
}

#[test]
fn load_pipeline_toml_metadata_partial() {
    let toml = r#"
[pipeline]
name = "partial"

[pipeline.metadata]
when = "Only when needed"

[[steps]]
id = "go"
prompt = "Do {topic}."
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse partial metadata");
    let meta = def.metadata.as_ref().expect("should have metadata");
    assert_eq!(meta.when.as_deref(), Some("Only when needed"));
    assert!(meta.skills.is_empty());
}

#[test]
fn no_gates_means_no_failures() {
    let gate = StepGate::default();
    let step = PipelineStep {
        id: "free".to_string(),
        prompt: "test".to_string(),
        gate: Some(gate.clone()),
        lock: false,
    };
    let run = PipelineRun {
        run_id: "r".to_string(),
        pipeline: "p".to_string(),
        topic: "t".to_string(),
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
        name: "p".to_string(),
        description: None,
        steps: vec![step.clone()],
        metadata: None,
    };
    let dir = tempfile::tempdir().unwrap();
    let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
    assert!(failures.is_empty());
}

// ── validate_pipeline ─────────────────────────────────────────────────

#[test]
fn validate_warns_on_missing_metadata() {
    let def = PipelineDefinition {
        name: "test".to_string(),
        description: None,
        steps: vec![PipelineStep {
            id: "go".to_string(),
            prompt: "test".to_string(),
            gate: None,
            lock: false,
        }],
        metadata: None,
    };
    let dir = tempfile::tempdir().unwrap();
    let issues = validate_pipeline(&def, dir.path());
    assert!(
        issues
            .iter()
            .any(|i| i.level == ValidationLevel::Warn && i.message.contains("metadata")),
        "expected warning about missing metadata"
    );
}

#[test]
fn validate_no_warning_with_metadata() {
    let def = PipelineDefinition {
        name: "test".to_string(),
        description: None,
        steps: vec![PipelineStep {
            id: "go".to_string(),
            prompt: "test".to_string(),
            gate: None,
            lock: false,
        }],
        metadata: Some(PipelineMetadataSection {
            when: Some("When needed".to_string()),
            skills: vec![],
        }),
    };
    let dir = tempfile::tempdir().unwrap();
    let issues = validate_pipeline(&def, dir.path());
    assert!(
        !issues.iter().any(|i| i.message.contains("metadata")),
        "should not warn about metadata when present"
    );
}

#[test]
fn validate_warns_on_missing_oracle() {
    let def = PipelineDefinition {
        name: "test".to_string(),
        description: None,
        steps: vec![PipelineStep {
            id: "check".to_string(),
            prompt: "check".to_string(),
            gate: Some(StepGate {
                oracles: vec![OracleGate {
                    name: "nonexistent-oracle".to_string(),
                    command: None,
                    description: None,
                    timeout: None,
                    scope: None,
                }],
                ..Default::default()
            }),
            lock: false,
        }],
        metadata: Some(PipelineMetadataSection::default()),
    };
    let dir = tempfile::tempdir().unwrap();
    let issues = validate_pipeline(&def, dir.path());
    assert!(
        issues
            .iter()
            .any(|i| i.level == ValidationLevel::Warn
                && i.message.contains("nonexistent-oracle")),
        "expected warning about missing oracle"
    );
}

#[test]
fn validate_skips_oracle_with_explicit_command() {
    let def = PipelineDefinition {
        name: "test".to_string(),
        description: None,
        steps: vec![PipelineStep {
            id: "check".to_string(),
            prompt: "check".to_string(),
            gate: Some(StepGate {
                oracles: vec![OracleGate {
                    name: "custom".to_string(),
                    command: Some("python check.py".to_string()),
                    description: None,
                    timeout: None,
                    scope: None,
                }],
                ..Default::default()
            }),
            lock: false,
        }],
        metadata: Some(PipelineMetadataSection::default()),
    };
    let dir = tempfile::tempdir().unwrap();
    let issues = validate_pipeline(&def, dir.path());
    assert!(
        !issues.iter().any(|i| i.message.contains("custom")),
        "should not warn about oracle with explicit command"
    );
}

// ── validate lock-without-gate ───────────────────────────────────────

#[test]
fn validate_warns_lock_true_no_gate() {
    let def = PipelineDefinition {
        name: "test".to_string(),
        description: None,
        steps: vec![PipelineStep {
            id: "locked-step".to_string(),
            prompt: "do work".to_string(),
            gate: None,
            lock: true,
        }],
        metadata: Some(PipelineMetadataSection::default()),
    };
    let dir = tempfile::tempdir().unwrap();
    let issues = validate_pipeline(&def, dir.path());
    assert!(
        issues.iter().any(|i| i.level == ValidationLevel::Warn
            && i.message.contains("locked-step")
            && i.message.contains("lock = true but no gate")),
        "expected warning about lock without gate, got: {:?}",
        issues
    );
}

#[test]
fn validate_no_warning_lock_true_with_gate() {
    let def = PipelineDefinition {
        name: "test".to_string(),
        description: None,
        steps: vec![PipelineStep {
            id: "gated-step".to_string(),
            prompt: "do work".to_string(),
            gate: Some(StepGate {
                structural: Some(StructuralGate {
                    min_artifacts: 1,
                    types: vec![],
                }),
                ..Default::default()
            }),
            lock: true,
        }],
        metadata: Some(PipelineMetadataSection::default()),
    };
    let dir = tempfile::tempdir().unwrap();
    let issues = validate_pipeline(&def, dir.path());
    assert!(
        !issues
            .iter()
            .any(|i| i.message.contains("lock = true but no gate")),
        "should not warn when gate is present, got: {:?}",
        issues
    );
}

#[test]
fn validate_no_warning_lock_false_no_gate() {
    let def = PipelineDefinition {
        name: "test".to_string(),
        description: None,
        steps: vec![PipelineStep {
            id: "unlocked-step".to_string(),
            prompt: "do work".to_string(),
            gate: None,
            lock: false,
        }],
        metadata: Some(PipelineMetadataSection::default()),
    };
    let dir = tempfile::tempdir().unwrap();
    let issues = validate_pipeline(&def, dir.path());
    assert!(
        !issues
            .iter()
            .any(|i| i.message.contains("lock = true but no gate")),
        "should not warn when lock is false, got: {:?}",
        issues
    );
}

// ── format_gate_summary ───────────────────────────────────────────────

#[test]
fn format_gate_summary_none_gate() {
    assert_eq!(format_gate_summary(&None), "");
}

#[test]
fn format_gate_summary_empty_gate() {
    let gate = StepGate::default();
    assert_eq!(format_gate_summary(&Some(gate)), "");
}

#[test]
fn format_gate_summary_structural_only() {
    let gate = StepGate {
        structural: Some(StructuralGate {
            min_artifacts: 1,
            types: vec![],
        }),
        ..Default::default()
    };
    assert_eq!(format_gate_summary(&Some(gate)), "[structural]");
}

#[test]
fn format_gate_summary_all_tiers() {
    let gate = StepGate {
        structural: Some(StructuralGate {
            min_artifacts: 1,
            types: vec![],
        }),
        procedural: Some(ProceduralGate {
            require_review: true,
            review_types: vec![],
            max_critical: None,
            max_high: None,
        }),
        coverage: Some(CoverageGate {
            require_input_manifest: true,
        }),
        oracles: vec![OracleGate {
            name: "check".to_string(),
            command: None,
            description: None,
            timeout: None,
            scope: None,
        }],
        approval: Some(ApprovalGate {
            required: true,
            message: None,
        }),
    };
    assert_eq!(
        format_gate_summary(&Some(gate)),
        "[structural + procedural + coverage + oracle + approval]"
    );
}

