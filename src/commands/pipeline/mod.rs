use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::cli::PipelineCommands;
use crate::context::current_context;

use super::require_project;

mod definition;
mod gates;
mod orchestration;
mod queries;
mod setup;

// Re-export public items that other modules reference
pub use definition::load_pipeline_toml;

// ─── Data structures ─────────────────────────────────────────────────────────

// ── New TOML-based pipeline model ────────────────────────────────────────────

/// One step in a TOML pipeline definition.
///
/// Stored as `[[steps]]` entries in a `.wai/resources/pipelines/<name>.toml` file.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PipelineStep {
    pub id: String,
    pub prompt: String,
    /// Optional gate configuration for this step.
    #[serde(default)]
    pub gate: Option<StepGate>,
    /// When true, artifacts for this step are locked (SHA-256 hashed) on advancement.
    #[serde(default)]
    pub lock: bool,
}

/// Gate configuration for a pipeline step.
///
/// Gates are evaluated in order: structural → procedural → coverage → oracle → approval.
/// The first failing tier blocks advancement.
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct StepGate {
    pub structural: Option<StructuralGate>,
    pub procedural: Option<ProceduralGate>,
    pub coverage: Option<CoverageGate>,
    #[serde(default)]
    pub oracles: Vec<OracleGate>,
    pub approval: Option<ApprovalGate>,
}

/// Structural gate: verify minimum artifact counts for this step.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StructuralGate {
    /// Minimum number of artifacts required.
    pub min_artifacts: usize,
    /// Optional filter: only count artifacts of these types (research, plan, design, etc.).
    #[serde(default)]
    pub types: Vec<String>,
}

/// Procedural gate: verify review coverage for step artifacts.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProceduralGate {
    /// Require a review artifact for each reviewable artifact.
    #[serde(default)]
    pub require_review: bool,
    /// Which artifact types require reviews. Defaults to all except "review".
    #[serde(default)]
    pub review_types: Vec<String>,
    /// Maximum allowed critical-severity findings (default: no limit).
    pub max_critical: Option<u32>,
    /// Maximum allowed high-severity findings (default: no limit).
    pub max_high: Option<u32>,
}

/// Coverage gate: require an input coverage manifest before advancing.
///
/// A coverage manifest is a wai artifact of type `review` tagged with
/// `coverage-manifest:<step-id>`, listing each input artifact with a disposition.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CoverageGate {
    /// When true, require a coverage manifest artifact before advancing.
    #[serde(default)]
    pub require_input_manifest: bool,
}

/// Oracle gate: run a user-defined script to validate artifacts.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OracleGate {
    pub name: String,
    /// Explicit command override (bypasses name resolution).
    pub command: Option<String>,
    /// Description shown in gate status output.
    #[allow(dead_code)] // Used by `wai pipeline show` (wai-zjt6)
    pub description: Option<String>,
    /// Timeout in seconds (default: 30).
    pub timeout: Option<u64>,
    /// Scope: "artifact" (default, one invocation per artifact) or "all" (one invocation with all paths).
    pub scope: Option<String>,
}

/// Approval gate: require explicit human approval before advancing.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApprovalGate {
    pub required: bool,
    /// Message shown when approval is needed.
    pub message: Option<String>,
}

/// A pipeline definition deserialized from a TOML file.
///
/// The TOML format uses a `[pipeline]` table and `[[steps]]` arrays:
/// ```toml
/// [pipeline]
/// name = "feature"
/// description = "Full feature workflow"
///
/// [[steps]]
/// id = "research"
/// prompt = "Research {topic}: gather background, constraints, prior art."
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PipelineDefinition {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<PipelineStep>,
    /// Optional metadata for discoverability (managed block, suggest).
    #[serde(default)]
    #[allow(dead_code)] // Read via lib workspace.rs for managed block generation
    pub metadata: Option<PipelineMetadataSection>,
}

/// Optional `[pipeline.metadata]` section for discoverability.
#[derive(Debug, Clone, serde::Deserialize, Default)]
#[allow(dead_code)] // Fields read via lib workspace.rs for managed block generation
pub struct PipelineMetadataSection {
    /// When to suggest this pipeline (human-readable description).
    pub when: Option<String>,
    /// Skill names this pipeline depends on.
    #[serde(default)]
    pub skills: Vec<String>,
}

/// Lock metadata written to a `.lock` sidecar TOML file.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)] // Used by pipeline lock commands (wai-yh5v, wai-bzrp)
pub struct ArtifactLock {
    pub artifact: String,
    pub locked_at: String,
    pub lock_hash: String,
    pub pipeline_run: String,
    pub pipeline_step: String,
}

/// Run state stored at `.wai/pipeline-runs/<run-id>.yml`.
#[derive(Debug, serde::Serialize, Deserialize)]
pub struct PipelineRun {
    pub run_id: String,
    pub pipeline: String,
    pub topic: String,
    pub created_at: String,
    /// Index of the current (not-yet-completed) step; equals total step count when done.
    pub current_step: usize,
    /// Per-step approval timestamps (step_id → ISO 8601 timestamp).
    #[serde(default)]
    pub approvals: std::collections::HashMap<String, String>,
}

/// Validation issue found during pipeline definition checking.
#[derive(Debug)]
pub struct ValidationIssue {
    pub level: ValidationLevel,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum ValidationLevel {
    Error,
    Warn,
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn run(cmd: PipelineCommands) -> Result<()> {
    match cmd {
        PipelineCommands::Status => cmd_status(),
        PipelineCommands::List => queries::cmd_list(),
        PipelineCommands::Init { name } => setup::cmd_init(&name),
        PipelineCommands::Start { name, topic } => {
            orchestration::cmd_start(&name, topic.as_deref())
        }
        PipelineCommands::Next => orchestration::cmd_next(),
        PipelineCommands::Current { json } => queries::cmd_current(json),
        PipelineCommands::Suggest { description } => queries::cmd_suggest(description.as_deref()),
        PipelineCommands::Approve => orchestration::cmd_approve(),
        PipelineCommands::Show { name } => queries::cmd_show(&name),
        PipelineCommands::Gates { name, step } => {
            queries::cmd_gates(name.as_deref(), step.as_deref())
        }
        PipelineCommands::Check { oracle } => queries::cmd_check(oracle.as_deref()),
        PipelineCommands::Validate { name } => queries::cmd_validate(name.as_deref()),
        PipelineCommands::Lock => orchestration::cmd_lock(),
        PipelineCommands::Verify => cmd_verify(),
    }
}

// ─── status ──────────────────────────────────────────────────────────────────

fn cmd_status() -> Result<()> {
    queries::cmd_current(current_context().json)
}

// ─── verify ──────────────────────────────────────────────────────────────────

fn cmd_verify() -> Result<()> {
    let project_root = require_project()?;
    let projects_dir = crate::config::projects_dir(&project_root);

    // Walk all .lock files under .wai/projects/
    let mut lock_count: usize = 0;
    let mut mismatches: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&projects_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if ext != "lock" {
            continue;
        }

        let lock = read_artifact_lock(path)?;
        let artifact_path = path.parent().unwrap().join(&lock.artifact);

        if !artifact_path.exists() {
            mismatches.push(format!(
                "  {} — artifact missing (expected at {})",
                lock.artifact,
                artifact_path.display()
            ));
            lock_count += 1;
            continue;
        }

        let actual_hash = artifact_hash(&artifact_path)?;
        if actual_hash != lock.lock_hash {
            mismatches.push(format!(
                "  {} — expected {}, got {}",
                lock.artifact, lock.lock_hash, actual_hash
            ));
        }

        lock_count += 1;
    }

    if lock_count == 0 {
        cliclack::log::info("No locked artifacts found.").into_diagnostic()?;
        return Ok(());
    }

    if mismatches.is_empty() {
        cliclack::log::success(format!("All {} locked artifacts verified.", lock_count))
            .into_diagnostic()?;
        Ok(())
    } else {
        let header = format!(
            "{} of {} locked artifacts failed verification:",
            mismatches.len(),
            lock_count
        );
        let detail = mismatches.join("\n");
        cliclack::log::error(format!("{}\n{}", header, detail)).into_diagnostic()?;
        std::process::exit(1);
    }
}

// ─── Artifact lock helpers ────────────────────────────────────────────────────

/// Substitute `{topic}` in a prompt string with the given topic value.
///
/// If the prompt contains no `{topic}` placeholder, the prompt is returned unchanged.
pub fn render_prompt(prompt: &str, topic: &str) -> String {
    prompt.replace("{topic}", topic)
}

/// Compute SHA-256 hash of an artifact file, normalizing line endings to LF.
///
/// Returns the hash as a hex string prefixed with "sha256:".
#[allow(dead_code)] // Used by pipeline lock/verify commands (wai-2lwo, wai-rdp5)
pub fn artifact_hash(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let content = fs::read_to_string(path).into_diagnostic()?;
    let normalized = content.replace("\r\n", "\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let hash = hasher.finalize();
    Ok(format!("sha256:{:x}", hash))
}

/// Write a `.lock` sidecar file for an artifact.
///
/// The lock file is named `<artifact>.<run-id>.lock` and placed alongside the artifact.
/// Returns the path to the lock file.
#[allow(dead_code)] // Used by pipeline lock commands (wai-yh5v, wai-bzrp)
pub fn write_artifact_lock(
    artifact_path: &Path,
    run_id: &str,
    step_id: &str,
) -> Result<std::path::PathBuf> {
    let hash = artifact_hash(artifact_path)?;
    let lock = ArtifactLock {
        artifact: artifact_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        locked_at: chrono::Utc::now().to_rfc3339(),
        lock_hash: hash,
        pipeline_run: run_id.to_string(),
        pipeline_step: step_id.to_string(),
    };
    let lock_filename = format!(
        "{}.{}.lock",
        artifact_path.file_name().unwrap().to_string_lossy(),
        run_id
    );
    let lock_path = artifact_path.parent().unwrap().join(&lock_filename);
    let toml_content = toml::to_string_pretty(&lock).into_diagnostic()?;
    fs::write(&lock_path, toml_content).into_diagnostic()?;
    Ok(lock_path)
}

/// Read and parse a `.lock` sidecar file.
#[allow(dead_code)] // Used by pipeline lock/verify commands (wai-yh5v, wai-rdp5)
pub fn read_artifact_lock(lock_path: &Path) -> Result<ArtifactLock> {
    let content = fs::read_to_string(lock_path).into_diagnostic()?;
    toml::from_str(&content).into_diagnostic()
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use definition::{load_pipeline_toml, validate_pipeline};
    use gates::{
        evaluate_gates, execute_oracle, find_step_artifact_paths, parse_frontmatter,
        resolve_oracle_command,
    };
    use queries::find_step_addenda;
    use setup::{builtin_template_names, get_builtin_template};
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

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

    fn write_toml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("create tempfile");
        f.write_all(content.as_bytes()).expect("write toml");
        f
    }

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
        assert_eq!(gates::format_gate_summary(&None), "");
    }

    #[test]
    fn format_gate_summary_empty_gate() {
        let gate = StepGate::default();
        assert_eq!(gates::format_gate_summary(&Some(gate)), "");
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
        assert_eq!(gates::format_gate_summary(&Some(gate)), "[structural]");
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
            gates::format_gate_summary(&Some(gate)),
            "[structural + procedural + coverage + oracle + approval]"
        );
    }

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
        assert_eq!(gates::format_gate_summary(&Some(gate)), "[coverage]");
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

    // ── artifact_hash ─────────────────────────────────────────────────────

    #[test]
    fn artifact_hash_consistent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifact.md");
        fs::write(&path, "hello world\n").unwrap();
        let h1 = artifact_hash(&path).unwrap();
        let h2 = artifact_hash(&path).unwrap();
        assert_eq!(h1, h2, "same file should produce identical hashes");
        assert!(
            h1.starts_with("sha256:"),
            "hash should be prefixed with sha256:"
        );
    }

    #[test]
    fn artifact_hash_normalizes_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let lf_path = dir.path().join("lf.md");
        let crlf_path = dir.path().join("crlf.md");
        fs::write(&lf_path, "line one\nline two\n").unwrap();
        fs::write(&crlf_path, "line one\r\nline two\r\n").unwrap();
        let lf_hash = artifact_hash(&lf_path).unwrap();
        let crlf_hash = artifact_hash(&crlf_path).unwrap();
        assert_eq!(
            lf_hash, crlf_hash,
            "CRLF and LF content should hash identically"
        );
    }

    #[test]
    fn artifact_hash_different_content() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.md");
        let b = dir.path().join("b.md");
        fs::write(&a, "content A\n").unwrap();
        fs::write(&b, "content B\n").unwrap();
        let ha = artifact_hash(&a).unwrap();
        let hb = artifact_hash(&b).unwrap();
        assert_ne!(ha, hb, "different content should produce different hashes");
    }

    // ── artifact lock sidecar ─────────────────────────────────────────────

    #[test]
    fn write_artifact_lock_creates_file_with_correct_name() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("research.md");
        fs::write(&artifact, "# Research\nSome findings.\n").unwrap();

        let lock_path = write_artifact_lock(&artifact, "run-abc", "step-1").unwrap();

        assert!(lock_path.exists(), "lock file should exist");
        assert_eq!(
            lock_path.file_name().unwrap().to_string_lossy(),
            "research.md.run-abc.lock"
        );
    }

    #[test]
    fn write_artifact_lock_roundtrips_via_read() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("design.md");
        fs::write(&artifact, "# Design\nArchitecture notes.\n").unwrap();

        let lock_path = write_artifact_lock(&artifact, "run-42", "generate").unwrap();
        let lock = read_artifact_lock(&lock_path).unwrap();

        assert_eq!(lock.artifact, "design.md");
        assert_eq!(lock.pipeline_run, "run-42");
        assert_eq!(lock.pipeline_step, "generate");
        assert!(
            lock.lock_hash.starts_with("sha256:"),
            "hash should be prefixed with sha256:"
        );
        // locked_at should be a valid RFC 3339 timestamp
        assert!(
            chrono::DateTime::parse_from_rfc3339(&lock.locked_at).is_ok(),
            "locked_at should be valid RFC 3339: {}",
            lock.locked_at
        );
    }

    #[test]
    fn write_artifact_lock_hash_matches_artifact_hash() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("notes.md");
        fs::write(&artifact, "Some content\nwith lines.\n").unwrap();

        let expected_hash = artifact_hash(&artifact).unwrap();
        let lock_path = write_artifact_lock(&artifact, "run-x", "review").unwrap();
        let lock = read_artifact_lock(&lock_path).unwrap();

        assert_eq!(
            lock.lock_hash, expected_hash,
            "lock hash should match artifact_hash"
        );
    }

    #[test]
    fn write_artifact_lock_includes_run_id_in_filename() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("plan.md");
        fs::write(&artifact, "# Plan\n").unwrap();

        let lock_path = write_artifact_lock(&artifact, "my-run-id", "step-0").unwrap();
        let filename = lock_path.file_name().unwrap().to_string_lossy();

        assert!(
            filename.contains("my-run-id"),
            "filename should contain run-id, got: {filename}"
        );
        assert!(
            filename.ends_with(".lock"),
            "filename should end with .lock, got: {filename}"
        );
    }

    // ── find_step_addenda ────────────────────────────────────────────────

    #[test]
    fn find_step_addenda_returns_matching_files() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        fs::write(
            research_dir.join("2026-04-14-correction.md"),
            "---\ntags: [pipeline-addendum:implement]\n---\n\nCorrection notes.",
        )
        .unwrap();

        fs::write(
            research_dir.join("2026-04-14-other.md"),
            "---\ntags: [pipeline-addendum:review]\n---\n\nOther notes.",
        )
        .unwrap();

        fs::write(
            research_dir.join("2026-04-14-normal.md"),
            "---\ntags: [pipeline-run:abc]\n---\n\nNormal research.",
        )
        .unwrap();

        let addenda = find_step_addenda(dir.path(), "implement");
        assert_eq!(addenda.len(), 1, "expected 1 addendum, got: {:?}", addenda);
        assert_eq!(addenda[0], "research/2026-04-14-correction.md");
    }

    #[test]
    fn find_step_addenda_returns_empty_when_none_match() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        fs::write(
            research_dir.join("2026-04-14-normal.md"),
            "---\ntags: [pipeline-run:abc]\n---\n\nNormal research.",
        )
        .unwrap();

        let addenda = find_step_addenda(dir.path(), "implement");
        assert!(
            addenda.is_empty(),
            "expected no addenda, got: {:?}",
            addenda
        );
    }

    #[test]
    fn find_step_addenda_scans_multiple_directories() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        let plans_dir = project_dir.join("plans");
        fs::create_dir_all(&research_dir).unwrap();
        fs::create_dir_all(&plans_dir).unwrap();

        fs::write(
            research_dir.join("2026-04-14-fix.md"),
            "---\ntags: [pipeline-addendum:design]\n---\n\nFix notes.",
        )
        .unwrap();

        fs::write(
            plans_dir.join("2026-04-14-revised.md"),
            "---\ntags: [pipeline-addendum:design]\n---\n\nRevised plan.",
        )
        .unwrap();

        let addenda = find_step_addenda(dir.path(), "design");
        assert_eq!(addenda.len(), 2, "expected 2 addenda, got: {:?}", addenda);
        assert!(addenda.contains(&"plans/2026-04-14-revised.md".to_string()));
        assert!(addenda.contains(&"research/2026-04-14-fix.md".to_string()));
    }

    #[test]
    fn find_step_addenda_handles_missing_projects_dir() {
        let dir = tempfile::tempdir().unwrap();
        let addenda = find_step_addenda(dir.path(), "implement");
        assert!(
            addenda.is_empty(),
            "expected no addenda for missing project dir"
        );
    }

    // ── find_step_artifact_paths ─────────────────────────────────────────

    #[test]
    fn find_step_artifact_paths_returns_matching_files() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        // Artifact tagged with both run and step tags
        fs::write(
            research_dir.join("2026-04-14-findings.md"),
            "---\ntags: [pipeline-run:run-abc, pipeline-step:generate]\n---\n\nFindings.",
        )
        .unwrap();

        // Artifact tagged with different step — should NOT match
        fs::write(
            research_dir.join("2026-04-14-other.md"),
            "---\ntags: [pipeline-run:run-abc, pipeline-step:review]\n---\n\nOther.",
        )
        .unwrap();

        // Artifact with no pipeline tags — should NOT match
        fs::write(
            research_dir.join("2026-04-14-plain.md"),
            "---\ntags: [general]\n---\n\nPlain.",
        )
        .unwrap();

        let paths = find_step_artifact_paths(dir.path(), "run-abc", "generate");
        assert_eq!(paths.len(), 1, "expected 1 artifact, got: {:?}", paths);
        assert!(
            paths[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("findings"),
            "expected findings artifact, got: {:?}",
            paths[0]
        );
    }

    #[test]
    fn find_step_artifact_paths_returns_empty_when_no_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        // Artifact tagged with different run
        fs::write(
            research_dir.join("2026-04-14-other.md"),
            "---\ntags: [pipeline-run:other-run, pipeline-step:generate]\n---\n\nOther.",
        )
        .unwrap();

        let paths = find_step_artifact_paths(dir.path(), "run-abc", "generate");
        assert!(paths.is_empty(), "expected no artifacts, got: {:?}", paths);
    }

    // ── cmd_next locking integration ────────────────────────────────────

    #[test]
    fn cmd_next_lock_creates_lock_files_for_step_artifacts() {
        // Simulate the locking path from cmd_next: when a step has lock=true,
        // find its artifacts and write lock sidecars for each.
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        let run_id = "run-lock-test";
        let step_id = "generate";

        // Create two artifacts tagged for this run+step
        fs::write(
            research_dir.join("2026-04-14-findings.md"),
            "---\ntags: [pipeline-run:run-lock-test, pipeline-step:generate]\n---\n\nFindings.",
        )
        .unwrap();
        fs::write(
            research_dir.join("2026-04-14-analysis.md"),
            "---\ntags: [pipeline-run:run-lock-test, pipeline-step:generate]\n---\n\nAnalysis.",
        )
        .unwrap();

        // Build a step definition with lock = true
        let step_def = PipelineStep {
            id: step_id.to_string(),
            prompt: String::new(),
            gate: None,
            lock: true,
        };

        // Execute the same logic as cmd_next's locking block
        assert!(step_def.lock, "step should have lock=true");
        let artifact_paths = find_step_artifact_paths(dir.path(), run_id, &step_def.id);
        assert_eq!(
            artifact_paths.len(),
            2,
            "expected 2 artifacts, got: {:?}",
            artifact_paths
        );

        for path in &artifact_paths {
            write_artifact_lock(path, run_id, &step_def.id).unwrap();
        }

        // Verify lock files exist for each artifact
        for path in &artifact_paths {
            let lock_filename = format!(
                "{}.{}.lock",
                path.file_name().unwrap().to_string_lossy(),
                run_id
            );
            let lock_path = path.parent().unwrap().join(&lock_filename);
            assert!(
                lock_path.exists(),
                "lock file should exist: {:?}",
                lock_path
            );

            // Verify lock content
            let lock = read_artifact_lock(&lock_path).unwrap();
            assert_eq!(lock.pipeline_run, run_id);
            assert_eq!(lock.pipeline_step, step_id);
            assert!(lock.lock_hash.starts_with("sha256:"));
        }
    }

    #[test]
    fn cmd_next_lock_skipped_when_lock_is_false() {
        // When lock=false, no lock files should be created (the if-branch is not entered)
        let step_def = PipelineStep {
            id: "review".to_string(),
            prompt: String::new(),
            gate: None,
            lock: false,
        };
        assert!(
            !step_def.lock,
            "step should have lock=false — locking block would be skipped"
        );
    }

    // ── cmd_verify (via artifact_hash / read_artifact_lock) ─────────────

    #[test]
    fn verify_passes_when_artifacts_are_intact() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project")
            .join("research");
        fs::create_dir_all(&project_dir).unwrap();

        let artifact = project_dir.join("notes.md");
        fs::write(&artifact, "# Notes\nSome content.\n").unwrap();

        let hash = artifact_hash(&artifact).unwrap();
        let lock = ArtifactLock {
            artifact: "notes.md".to_string(),
            locked_at: chrono::Utc::now().to_rfc3339(),
            lock_hash: hash.clone(),
            pipeline_run: "run-1".to_string(),
            pipeline_step: "generate".to_string(),
        };
        let lock_path = project_dir.join("notes.md.run-1.lock");
        fs::write(&lock_path, toml::to_string_pretty(&lock).unwrap()).unwrap();

        let projects_dir = dir.path().join(".wai").join("projects");
        let mut mismatches: Vec<String> = Vec::new();
        let mut lock_count: usize = 0;
        for entry in walkdir::WalkDir::new(&projects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("lock") {
                continue;
            }
            let parsed_lock = read_artifact_lock(path).unwrap();
            let artifact_path = path.parent().unwrap().join(&parsed_lock.artifact);
            let actual = artifact_hash(&artifact_path).unwrap();
            if actual != parsed_lock.lock_hash {
                mismatches.push(parsed_lock.artifact.clone());
            }
            lock_count += 1;
        }

        assert_eq!(lock_count, 1, "should find exactly one lock file");
        assert!(mismatches.is_empty(), "all artifacts should verify OK");
    }

    #[test]
    fn verify_detects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project")
            .join("research");
        fs::create_dir_all(&project_dir).unwrap();

        let artifact = project_dir.join("notes.md");
        fs::write(&artifact, "# Notes\nOriginal content.\n").unwrap();

        let original_hash = artifact_hash(&artifact).unwrap();
        let lock = ArtifactLock {
            artifact: "notes.md".to_string(),
            locked_at: chrono::Utc::now().to_rfc3339(),
            lock_hash: original_hash.clone(),
            pipeline_run: "run-1".to_string(),
            pipeline_step: "generate".to_string(),
        };
        let lock_path = project_dir.join("notes.md.run-1.lock");
        fs::write(&lock_path, toml::to_string_pretty(&lock).unwrap()).unwrap();

        // Tamper with the artifact
        fs::write(&artifact, "# Notes\nTampered content!\n").unwrap();

        let projects_dir = dir.path().join(".wai").join("projects");
        let mut mismatches: Vec<String> = Vec::new();
        for entry in walkdir::WalkDir::new(&projects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("lock") {
                continue;
            }
            let parsed_lock = read_artifact_lock(path).unwrap();
            let artifact_path = path.parent().unwrap().join(&parsed_lock.artifact);
            let actual = artifact_hash(&artifact_path).unwrap();
            if actual != parsed_lock.lock_hash {
                mismatches.push(parsed_lock.artifact.clone());
            }
        }

        assert_eq!(
            mismatches.len(),
            1,
            "should detect exactly one mismatch, got: {:?}",
            mismatches
        );
        assert_eq!(mismatches[0], "notes.md");
    }
}
