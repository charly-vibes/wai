//! Unit tests for pipeline definition loading, TOML parsing, and render_prompt.

use crate::commands::pipeline::definition::load_pipeline_toml;
use crate::commands::pipeline::gates::parse_frontmatter;
use crate::commands::pipeline::render_prompt;
use std::io::Write;
use tempfile::NamedTempFile;

pub(super) fn write_toml(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create tempfile");
    f.write_all(content.as_bytes()).expect("write toml");
    f
}

// ── render_prompt ──────────────────────────────────────────────────────

#[test]
fn render_prompt_substitutes_topic() {
    assert_eq!(render_prompt("Hello {topic}!", "world"), "Hello world!");
}

#[test]
fn render_prompt_no_placeholder_no_panic() {
    assert_eq!(render_prompt("Hello!", "world"), "Hello!");
}

#[test]
fn render_prompt_multiple_occurrences() {
    assert_eq!(
        render_prompt("Research {topic}. Focus on {topic} constraints.", "auth"),
        "Research auth. Focus on auth constraints."
    );
}

// ── load_pipeline_toml ─────────────────────────────────────────────────

#[test]
fn load_pipeline_toml_valid() {
    let toml = r#"
[pipeline]
name = "feature"
description = "A feature workflow"

[[steps]]
id = "research"
prompt = "Research {topic}: gather background."

[[steps]]
id = "implement"
prompt = "Implement {topic}."
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse valid TOML");
    assert_eq!(def.name, "feature");
    assert_eq!(def.description.as_deref(), Some("A feature workflow"));
    assert_eq!(def.steps.len(), 2);
    assert_eq!(def.steps[0].id, "research");
    assert_eq!(def.steps[1].id, "implement");
}

#[test]
fn load_pipeline_toml_no_description() {
    let toml = r#"
[pipeline]
name = "minimal"

[[steps]]
id = "go"
prompt = "Do {topic}."
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse minimal TOML");
    assert_eq!(def.description, None);
    assert_eq!(def.steps.len(), 1);
}

#[test]
fn load_pipeline_toml_rejects_duplicate_ids() {
    let toml = r#"
[pipeline]
name = "broken"

[[steps]]
id = "research"
prompt = "Research {topic}."

[[steps]]
id = "research"
prompt = "More research on {topic}."
"#;
    let f = write_toml(toml);
    let result = load_pipeline_toml(f.path());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("duplicate step id: research"),
        "expected 'duplicate step id: research' in error, got: {msg}"
    );
}

#[test]
fn load_pipeline_toml_rejects_empty_prompt() {
    let toml = r#"
[pipeline]
name = "broken"

[[steps]]
id = "research"
prompt = ""
"#;
    let f = write_toml(toml);
    let result = load_pipeline_toml(f.path());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("empty prompt for step: research"),
        "expected 'empty prompt for step: research' in error, got: {msg}"
    );
}

#[test]
fn load_pipeline_toml_rejects_whitespace_only_prompt() {
    let toml = "[pipeline]\nname = \"broken\"\n\n[[steps]]\nid = \"step1\"\nprompt = \"   \"\n";
    let f = write_toml(toml);
    let result = load_pipeline_toml(f.path());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("empty prompt for step: step1"),
        "expected 'empty prompt for step: step1' in error, got: {msg}"
    );
}

// ── gate TOML parsing ─────────────────────────────────────────────────

#[test]
fn load_pipeline_toml_with_gates() {
    let toml = r#"
[pipeline]
name = "gated"

[[steps]]
id = "generate"
prompt = "Generate {topic}."

[steps.gate.structural]
min_artifacts = 1
types = ["research"]

[steps.gate.procedural]
require_review = true
max_critical = 0

[steps.gate.approval]
required = true
message = "Review before advancing."

[[steps]]
id = "done"
prompt = "Wrap up."
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse gated pipeline");
    assert_eq!(def.steps.len(), 2);

    let gate = def.steps[0]
        .gate
        .as_ref()
        .expect("step 0 should have a gate");
    let sg = gate.structural.as_ref().unwrap();
    assert_eq!(sg.min_artifacts, 1);
    assert_eq!(sg.types, vec!["research"]);

    let pg = gate.procedural.as_ref().unwrap();
    assert!(pg.require_review);
    assert_eq!(pg.max_critical, Some(0));

    let ag = gate.approval.as_ref().unwrap();
    assert!(ag.required);
    assert_eq!(ag.message.as_deref(), Some("Review before advancing."));

    assert!(def.steps[1].gate.is_none());
}

#[test]
fn load_pipeline_toml_with_oracles() {
    let toml = r#"
[pipeline]
name = "oracle-test"

[[steps]]
id = "check"
prompt = "Check {topic}."

[[steps.gate.oracles]]
name = "dim-analysis"
timeout = 60

[[steps.gate.oracles]]
name = "custom"
command = "python check.py"
scope = "all"
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse oracle pipeline");
    let gate = def.steps[0].gate.as_ref().expect("should have gate");
    assert_eq!(gate.oracles.len(), 2);
    assert_eq!(gate.oracles[0].name, "dim-analysis");
    assert_eq!(gate.oracles[0].timeout, Some(60));
    assert_eq!(gate.oracles[1].command.as_deref(), Some("python check.py"));
    assert_eq!(gate.oracles[1].scope.as_deref(), Some("all"));
}

// ── lock field ─────────────────────────────────────────────────────

#[test]
fn load_pipeline_toml_lock_defaults_to_false() {
    let toml = r#"
[pipeline]
name = "no-lock"

[[steps]]
id = "step1"
prompt = "Do something."
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse pipeline without lock field");
    assert!(!def.steps[0].lock, "lock should default to false");
}

#[test]
fn load_pipeline_toml_lock_true() {
    let toml = r#"
[pipeline]
name = "locked"

[[steps]]
id = "step1"
prompt = "Do something."
lock = true

[[steps]]
id = "step2"
prompt = "Do something else."
"#;
    let f = write_toml(toml);
    let def = load_pipeline_toml(f.path()).expect("should parse pipeline with lock = true");
    assert!(def.steps[0].lock, "step1 lock should be true");
    assert!(!def.steps[1].lock, "step2 lock should default to false");
}

// ── frontmatter parsing ──────────────────────────────────────────────

#[test]
fn parse_frontmatter_extracts_tags_and_reviews() {
    let content =
        "---\ntags: [pipeline-run:test, pipeline-step:gen]\nreviews: findings.md\n---\n\nbody";
    let fm = parse_frontmatter(content);
    assert_eq!(fm.tags, vec!["pipeline-run:test", "pipeline-step:gen"]);
    assert_eq!(fm.reviews.as_deref(), Some("findings.md"));
}

#[test]
fn parse_frontmatter_extracts_severity() {
    let content = "---\nseverity: {critical: 2, high: 1, medium: 0, low: 5}\n---\n\nbody";
    let fm = parse_frontmatter(content);
    assert_eq!(fm.severity_critical, 2);
    assert_eq!(fm.severity_high, 1);
}

#[test]
fn parse_frontmatter_handles_no_frontmatter() {
    let content = "just some text";
    let fm = parse_frontmatter(content);
    assert!(fm.tags.is_empty());
    assert!(fm.reviews.is_none());
}
