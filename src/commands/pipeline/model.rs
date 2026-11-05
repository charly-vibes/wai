//! Pipeline definition model types: steps, gates, runs, locks.

use serde::Deserialize;

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

/// One disclosed gate bypass recorded in a run's ledger (wai-giqg, R2 of
/// harden-orchestration-gates). Hooks cannot self-record a `--no-verify`
/// bypass — the flag skips them — so the agent records it explicitly via
/// `wai pipeline bypass <reason>`. Ledger-only: telemetry, not enforcement.
#[derive(Debug, serde::Serialize, Deserialize)]
pub struct BypassEntry {
    pub timestamp: String,
    pub reason: String,
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
    /// Epic coordination (wai-vx02.3): set on parent runs coordinating the
    /// child runs of a beads epic.
    #[serde(default)]
    pub epic: Option<String>,
    /// Ready child issue ids discovered at epic start (parent runs).
    #[serde(default)]
    pub child_issues: Vec<String>,
    /// Run ids of child runs spawned under this epic (parent runs).
    #[serde(default)]
    pub child_runs: Vec<String>,
    /// Path (project-root-relative) of the handoff artifact recorded for this
    /// run's topic via `wai handoff create` (wai-vx02.5).
    #[serde(default)]
    pub handoff_artifact: Option<String>,
    /// Disclosed gate bypasses recorded during this run (wai-giqg). Older run
    /// state files predate the ledger — absent means empty, not error.
    #[serde(default)]
    pub bypasses: Vec<BypassEntry>,
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
