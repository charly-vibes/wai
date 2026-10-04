//! Leaf subcommand enums split out of `crate::cli` to keep files below
//! pretender's file_lines threshold. Re-exported from `crate::cli`.

use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum NewCommands {
    /// Create a new project
    Project {
        /// Project name
        name: String,

        /// Project template
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Create a new area
    Area {
        /// Area name
        name: String,
    },

    /// Create a new resource
    Resource {
        /// Resource name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum AddCommands {
    /// Add research notes
    Research {
        /// Research content
        content: Option<String>,

        /// Import from file
        #[arg(short, long)]
        file: Option<String>,

        /// Associate with a project
        #[arg(short, long)]
        project: Option<String>,

        /// Add tags
        #[arg(short, long)]
        tags: Option<String>,

        /// Link to a bead/issue ID (adds bead field to frontmatter)
        #[arg(long)]
        bead: Option<String>,

        /// Path to an artifact this corrects (creates an addendum)
        #[arg(long)]
        corrects: Option<String>,

        /// Comma-separated repo-relative paths this artifact tracks for freshness
        #[arg(long)]
        tracks: Option<String>,
    },

    /// Add a plan document
    Plan {
        /// Plan content
        content: Option<String>,

        /// Import from file
        #[arg(short, long)]
        file: Option<String>,

        /// Associate with a project
        #[arg(short, long)]
        project: Option<String>,

        /// Comma-separated tags written as YAML frontmatter
        #[arg(short, long)]
        tags: Option<String>,

        /// Path to an artifact this corrects (creates an addendum)
        #[arg(long)]
        corrects: Option<String>,

        /// Comma-separated repo-relative paths this artifact tracks for freshness
        #[arg(long)]
        tracks: Option<String>,
    },

    /// Add a design document
    Design {
        /// Design content
        content: Option<String>,

        /// Import from file
        #[arg(short, long)]
        file: Option<String>,

        /// Associate with a project
        #[arg(short, long)]
        project: Option<String>,

        /// Comma-separated tags written as YAML frontmatter
        #[arg(short, long)]
        tags: Option<String>,

        /// Path to an artifact this corrects (creates an addendum)
        #[arg(long)]
        corrects: Option<String>,

        /// Comma-separated repo-relative paths this artifact tracks for freshness
        #[arg(long)]
        tracks: Option<String>,
    },

    /// Add a review artifact for an existing artifact
    Review {
        /// Review content
        content: Option<String>,

        /// Import from file
        #[arg(short, long)]
        file: Option<String>,

        /// Associate with a project
        #[arg(short, long)]
        project: Option<String>,

        /// Comma-separated tags written as YAML frontmatter
        #[arg(short, long)]
        tags: Option<String>,

        /// Target artifact filename this review covers (required)
        #[arg(long)]
        reviews: String,

        /// Review verdict: pass, fail, or needs-work
        #[arg(long)]
        verdict: Option<String>,

        /// Severity counts as comma-separated level:count pairs (e.g. critical:0,high:1,medium:3,low:2)
        #[arg(long)]
        severity: Option<String>,

        /// Skill that produced this review (informational only)
        #[arg(long)]
        produced_by: Option<String>,

        /// Path to an artifact this corrects (creates an addendum)
        #[arg(long)]
        corrects: Option<String>,
    },

    /// Scaffold a new agent skill file
    ///
    /// Skill names may be flat ("my-skill") or hierarchical ("category/action").
    /// Only one '/' separator is allowed; each segment must be lowercase
    /// letters, digits, and hyphens (no leading/trailing hyphens).
    ///
    /// Built-in templates: gather, create, tdd, rule-of-5, ubiquitous-language
    Skill {
        /// Skill name (e.g. "my-skill" or "issue/gather")
        name: String,

        /// Start from a built-in template.
        ///
        /// Valid templates:
        ///   gather    — research stub: wai search, codebase exploration, wai add research
        ///   create    — creation stub: retrieve plan, bd create items, wire dependencies
        ///   tdd       — TDD stub: RED/GREEN/REFACTOR loop with cargo test and commits
        ///   rule-of-5 — review stub: 5 passes with convergence check and APPROVED/NEEDS_CHANGES/NEEDS_HUMAN verdict
        ///   ubiquitous-language — terminology curation stub: read the root index first, then update only relevant context files
        #[arg(long, value_name = "TEMPLATE")]
        template: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ResourceCommands {
    /// Add a resource (skill, rule, context)
    #[command(subcommand)]
    Add(ResourceAddCommands),

    /// List resources
    #[command(subcommand)]
    List(ResourceListCommands),

    /// Import resources from a directory or archive
    #[command(subcommand)]
    Import(ResourceImportCommands),

    /// Install a skill globally or from another repository
    ///
    /// EXAMPLES
    ///   wai resource install issue/gather --global
    ///     Copies the skill from the current project into ~/.wai/resources/skills/
    ///
    ///   wai resource install issue/gather --from-repo ../other-project
    ///     Copies the skill from another repository into the current project's skills directory
    Install(ResourceInstallArgs),

    /// Export skills to a tar.gz archive for sharing
    ///
    /// EXAMPLES
    ///   wai resource export issue/gather impl/run --output skills.tar.gz
    Export(ResourceExportArgs),
}

#[derive(Subcommand)]
pub enum ResourceAddCommands {
    /// Add a skill
    ///
    /// Skill names may be flat ("my-skill") or hierarchical ("category/action").
    /// Only one '/' separator is allowed; each segment must be lowercase
    /// letters, digits, and hyphens (no leading/trailing hyphens).
    ///
    /// Built-in templates: gather, create, tdd, rule-of-5, ubiquitous-language
    Skill {
        /// Skill name (e.g. "my-skill" or "issue/gather")
        name: String,

        /// Start from a built-in template.
        ///
        /// Valid templates:
        ///   gather    — research stub: wai search, codebase exploration, wai add research
        ///   create    — creation stub: retrieve plan, bd create items, wire dependencies
        ///   tdd       — TDD stub: RED/GREEN/REFACTOR loop with cargo test and commits
        ///   rule-of-5 — review stub: 5 passes with convergence check and APPROVED/NEEDS_CHANGES/NEEDS_HUMAN verdict
        ///   ubiquitous-language — terminology curation stub: read the root index first, then update only relevant context files
        #[arg(long, value_name = "TEMPLATE")]
        template: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ResourceListCommands {
    /// List all skills
    Skills {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum ResourceImportCommands {
    /// Import skills from a directory
    Skills {
        /// Path to import skills from
        #[arg(long)]
        from: Option<String>,
    },

    /// Import skills from a tar.gz archive
    ///
    /// EXAMPLES
    ///   wai resource import archive skills.tar.gz
    ///   wai resource import archive skills.tar.gz --yes
    Archive {
        /// Path to the tar.gz archive to import
        file: String,

        /// Overwrite existing skills without prompting
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Args)]
pub struct ResourceInstallArgs {
    /// Skill name to install (e.g. "my-skill" or "issue/gather")
    pub skill: String,

    /// Install skill globally to ~/.wai/resources/skills/
    ///
    /// Copies the skill from the current project's skills directory into the
    /// global library, making it available in all projects.
    #[arg(long, conflicts_with = "from_repo")]
    pub global: bool,

    /// Copy skill from another repository into the current project
    ///
    /// Reads from <PATH>/.wai/resources/agent-config/skills/<skill>/SKILL.md
    #[arg(long, value_name = "PATH", conflicts_with = "global")]
    pub from_repo: Option<String>,
}

#[derive(Args)]
pub struct ResourceExportArgs {
    /// Skill names to export (e.g. "issue/gather" "impl/run")
    #[arg(value_name = "SKILL", required = true)]
    pub skills: Vec<String>,

    /// Output archive file path (e.g. skills.tar.gz)
    #[arg(long, value_name = "FILE")]
    pub output: String,
}

#[derive(Subcommand)]
pub enum PipelineCommands {
    /// Start a new TOML pipeline run
    ///
    /// Loads a TOML pipeline definition, generates a unique run ID, writes run
    /// state to `.wai/pipeline-runs/<run-id>.yml`, records the run ID in
    /// `.wai/resources/pipelines/.last-run`, then prints an env export line
    /// and the first step prompt.
    ///
    /// EXAMPLES
    ///   wai pipeline start feature --topic=auth-refactor
    ///   wai pipeline start review --topic="my feature"
    ///
    /// ENVIRONMENT
    ///   Sets WAI_PIPELINE_RUN in your shell when you run the printed export line.
    ///   `wai add` picks up the run ID automatically from `.wai/resources/pipelines/.last-run`.
    Start {
        /// Name of the pipeline to start (must be a .toml file in .wai/resources/pipelines/)
        name: String,

        /// Topic to use for {topic} substitution in step prompts
        #[arg(long)]
        topic: Option<String>,
    },

    /// Show status for the active pipeline run
    ///
    /// Resolves the active run from `WAI_PIPELINE_RUN` or the `.last-run`
    /// pointer file and prints the current step. With `--json`, emits
    /// machine-readable active-run context for agent integrations.
    Status,

    /// List all pipelines
    List,

    /// Scaffold a new TOML pipeline definition
    ///
    /// Creates `.wai/resources/pipelines/<name>.toml` with a minimal two-step
    /// template, or a built-in template. See `wai pipeline --help -v` for the
    /// current built-in template list. Edit the prompts, then start a run with:
    ///   wai pipeline start <name> --topic=<your-topic>
    ///
    /// EXAMPLES
    ///   wai pipeline init my-workflow
    ///   wai pipeline init tdd-ro5
    Init {
        /// Name for the new pipeline (creates <name>.toml)
        name: String,
    },

    /// Advance to the next step in the active pipeline run
    ///
    /// Resolves the active run from `WAI_PIPELINE_RUN` env var, falling back
    /// to the `.last-run` pointer file. Marks the current step complete,
    /// increments `current_step`, persists run state, then prints the next
    /// step prompt or a completion block with a `wai close` suggestion.
    ///
    /// EXAMPLES
    ///   wai pipeline next
    ///
    /// ENVIRONMENT
    ///   WAI_PIPELINE_RUN  When set, identifies the active run. Falls back to
    ///                     `.wai/resources/pipelines/.last-run` when not set.
    Next,

    /// Reprint the current step prompt (for session recovery after /clear)
    ///
    /// Resolves the active run from `WAI_PIPELINE_RUN` env var, falling back
    /// to the `.last-run` pointer file. Loads run state and pipeline
    /// definition, then reprints the current step prompt WITHOUT advancing
    /// the step counter. Pure read-only operation.
    ///
    /// Use this after a `/clear` or terminal loss to recover the current
    /// step context without losing your place.
    ///
    /// EXAMPLES
    ///   wai pipeline current
    ///
    /// ENVIRONMENT
    ///   WAI_PIPELINE_RUN  When set, identifies the active run. Falls back to
    ///                     `.wai/resources/pipelines/.last-run` when not set.
    Current {
        /// Output machine-readable JSON for the active run
        #[arg(long)]
        json: bool,
    },

    /// List and rank available TOML pipelines, optionally by keyword match
    ///
    /// Scans `.wai/resources/pipelines/` for `.toml` files. If a description
    /// is provided, ranks pipelines by keyword overlap (case-insensitive word
    /// matching against pipeline name and description). Ties are broken
    /// alphabetically. An empty string is treated as absent (no scoring).
    ///
    /// EXAMPLES
    ///   wai pipeline suggest
    ///   wai pipeline suggest "auth login flow"
    ///   wai pipeline suggest "database migration"
    Suggest {
        /// Optional description to filter/rank pipelines by keyword overlap
        description: Option<String>,
    },

    /// Record human approval for the current pipeline step
    ///
    /// Sets an approval timestamp in the run state. Required by steps that
    /// declare an approval gate. Approval is invalidated if new artifacts
    /// are created for the step after approval.
    ///
    /// EXAMPLES
    ///   wai pipeline approve
    Approve,

    /// Quarantine abandoned mid-flight runs (stale-run GC)
    ///
    /// Scans `.wai/pipeline-runs/` for mid-flight runs whose state-file mtime
    /// exceeds the stale threshold (`pipeline.staleDays` in config.toml,
    /// default 14). Default is a dry run listing candidates; `--yes` moves
    /// each to `pipeline-runs/stale/<run>.<timestamp>.yml` (moved, never
    /// deleted) and drops a pointer that references a quarantined run.
    ///
    /// EXAMPLES
    ///   wai pipeline gc
    ///   wai pipeline gc --yes
    Gc {
        /// Execute the quarantine (default: dry run)
        #[arg(long)]
        yes: bool,
    },

    /// Show detailed pipeline definition with steps and gate configuration
    ///
    /// Displays the pipeline name, description, metadata (when, skills),
    /// step list with gate summary per step, and oracle directory path.
    ///
    /// EXAMPLES
    ///   wai pipeline show scientific-research
    Show {
        /// Pipeline name to display
        name: String,
    },

    /// Show gate requirements and live status for a pipeline step
    ///
    /// With an active run (and no arguments), shows live status for the
    /// current step. Without an active run, the pipeline name is required
    /// and --step selects the step to display.
    ///
    /// EXAMPLES
    ///   wai pipeline gates
    ///   wai pipeline gates scientific-research --step=generate
    Gates {
        /// Pipeline name (required if no active run)
        name: Option<String>,

        /// Step ID to display gates for
        #[arg(long)]
        step: Option<String>,
    },

    /// Evaluate all gates for the current step without advancing
    ///
    /// Runs structural, procedural, oracle, and approval gate checks
    /// and reports per-tier status. Does NOT advance the step.
    /// Use --oracle to run a single oracle against all applicable artifacts.
    ///
    /// EXAMPLES
    ///   wai pipeline check
    ///   wai pipeline check --oracle=dimensional-analysis
    Check {
        /// Run only this oracle against all applicable artifacts
        #[arg(long)]
        oracle: Option<String>,
    },

    /// Validate pipeline TOML definitions for correctness
    ///
    /// Checks TOML structure, gate configuration, oracle paths, and metadata.
    /// Validates a specific pipeline by name, or all pipelines if no name given.
    ///
    /// EXAMPLES
    ///   wai pipeline validate scientific-research
    ///   wai pipeline validate
    Validate {
        /// Pipeline name (validates all if omitted)
        name: Option<String>,
    },

    /// Lock the current step's artifacts with SHA-256 hashes
    ///
    /// Computes SHA-256 hashes for all artifacts tagged with the current step,
    /// writes .lock sidecar files, and marks the step as locked in run state.
    ///
    /// EXAMPLES
    ///   wai pipeline lock
    Lock,

    /// Verify integrity of locked pipeline artifacts
    ///
    /// Recomputes SHA-256 hashes for all locked artifacts and compares
    /// against stored lock metadata. Exits non-zero on mismatch.
    ///
    /// EXAMPLES
    ///   wai pipeline verify
    Verify,
}
