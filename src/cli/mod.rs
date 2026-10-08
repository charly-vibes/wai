use clap::{CommandFactory, Parser, Subcommand};
use genesis::guide::{CliFormat, CliVerbosity};
use std::path::PathBuf;

pub const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("WAI_GIT_COMMIT"),
    "-",
    env!("WAI_GIT_BRANCH"),
    env!("WAI_GIT_DIRTY"),
    ")"
);

mod subcommands;
pub use subcommands::*;

#[derive(Parser)]
#[command(
    name = "wai",
    about = "wai /waɪ/ — know why it was built that way",
    long_about = "wai /waɪ/ — pronounced like \"why\", also read as \"way\"\n\n\
        Most specs define what to build. Wai extends the workflow to also inform —\n\
        preserving the research, reasoning, and decisions that shaped the design.\n\n\
        Organizes artifacts using the PARA method (Projects, Areas, Resources, Archives)\n\
        with project phase tracking, agent config sync, handoff generation, and plugin integration.",
    version = VERSION,
    after_help = "ENVIRONMENT\n  \
        WAI_PROJECT       Session-scoped project binding (set via: eval $(wai project use <name>))\n  \
        WAI_PIPELINE_RUN  Override active pipeline run ID\n\n\
        Run 'wai <command> --help' for more information on a command."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output verbosity (-v, -vv, -vvv) and quiet mode
    #[command(flatten)]
    pub verbose: CliVerbosity,

    /// Output format (--json, --human)
    #[command(flatten)]
    pub format: CliFormat,

    /// Disable interactive prompts
    #[arg(long, global = true)]
    pub no_input: bool,

    /// Auto-confirm actions with defaults
    #[arg(long, global = true)]
    pub yes: bool,

    /// Run in read-only safe mode
    #[arg(long, global = true)]
    pub safe: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project, area, or resource
    #[command(subcommand)]
    New(NewCommands),

    /// Add artifacts (research, plans, designs) to a project
    #[command(subcommand)]
    Add(AddCommands),

    /// Show information about items
    Show {
        /// Item name to show details for (project, area, or resource name)
        name: Option<String>,
    },

    /// Move items between PARA categories
    #[command(name = "move")]
    Move(MoveArgs),

    /// Initialize wai in the current directory
    Init {
        /// Project name (defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Check project status and suggest next steps
    Status,

    /// Show or change the current project phase
    ///
    /// Resolves the target project via: --project flag, then WAI_PROJECT env var,
    /// then auto-detect if exactly one project exists.
    Phase(PhaseArgs),

    /// Sync agent configs to tool-specific locations.
    ///
    /// Reads projections from .wai/resources/agent-config/.projections.yml.
    /// Each projection maps source files to a target location using a strategy
    /// (symlink, inline, reference, copy).
    ///
    /// Built-in target: `claude-code` — translates hierarchical wai skills
    /// (skills/<category>/<action>/SKILL.md) into Claude Code slash commands
    /// (.claude/commands/<category>/<action>.md) with translated frontmatter.
    /// No strategy or sources required for this target.
    Sync {
        /// Only show sync status without modifying files
        #[arg(long)]
        status: bool,

        /// Preview operations without making any changes
        #[arg(long)]
        dry_run: bool,

        /// Sync .wai/areas/ and .wai/resources/ from the main git worktree
        #[arg(long)]
        from_main: bool,
    },

    /// Manage agent configuration files
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Generate handoff documents
    #[command(subcommand)]
    Handoff(HandoffCommands),

    /// Search across all artifacts
    Search(crate::commands::search::SearchArgs),

    /// View chronological timeline of artifacts
    Timeline {
        /// Project name
        project: String,

        /// Show only entries from this date onward (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// Show only entries up to this date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,

        /// Show oldest entries first
        #[arg(long)]
        reverse: bool,
    },

    /// Manage plugins
    #[command(subcommand)]
    Plugin(PluginCommands),

    /// Check wai workspace health
    #[command(
        about = "Check wai workspace health — validates .wai/ structure, config.toml, projections, and plugins. Run this when your workspace seems broken.",
        long_about = "Checks wai workspace health: .wai/ directory structure, config.toml validity,\n\
            schema version, projections, plugin tool availability, agent config sync,\n\
            project state, and agent instructions.\n\n\
            Exits with code 1 if any check fails. Use --fix to automatically repair\n\
            issues where possible.\n\n\
            For repo hygiene and agent workflow conventions (skills, best practices),\n\
            run 'wai way' instead — it works without a wai workspace."
    )]
    Doctor {
        /// Automatically fix issues where possible
        #[arg(long)]
        fix: bool,
    },

    /// Show repo hygiene and agent workflow conventions — skills, rules, best practices. Works without a wai workspace.
    #[command(
        about = "Show repo hygiene and agent workflow conventions — skills, rules, best practices. Works without a wai workspace.",
        long_about = "Shows repo hygiene status and agent workflow conventions for AI-friendly development.\n\n\
            Covers 11 areas: task runners (justfile, Makefile), git hooks (prek, pre-commit),\n\
            editor config, documentation (README, LICENSE, CONTRIBUTING, .gitignore), AI instructions\n\
            (CLAUDE.md, AGENTS.md), LLM context (llm.txt), agent skills, CI/CD, dev containers,\n\
            and release pipelines.\n\n\
            These are recommendations, not requirements — the command always exits successfully\n\
            and suggests improvements without enforcing them. Works in any directory; a wai\n\
            workspace is not required.\n\n\
            For wai workspace health (broken .wai/, config errors, plugin issues), run 'wai doctor' instead.\n\n\
            Use --fix skills to scaffold missing recommended agent skills.\n\
            Use --json for machine-readable output suitable for CI integration and automation."
    )]
    Way {
        /// Topic to discuss interactively (e.g. ci, hooks, coverage)
        #[arg(value_name = "TOPIC")]
        topic: Option<String>,

        /// Scaffold missing items for a check: skills
        #[arg(long, value_name = "CHECK")]
        fix: Option<String>,
    },

    /// Import existing tool configurations
    Import {
        /// Path to import from (e.g., .claude/, .cursorrules)
        path: String,
    },

    /// Manage resources (skills, rules, context)
    #[command(subcommand)]
    Resource(ResourceCommands),

    /// Run the interactive quickstart tutorial
    Tutorial,

    /// Wrap up a session: create a handoff and show next steps
    Close {
        /// Project name (auto-detected when only one project exists)
        #[arg(short, long)]
        project: Option<String>,

        /// Prompt for a short insight to save to bd memories
        #[arg(long)]
        remember: bool,

        /// Close even when an active pipeline run is still in progress
        /// (intentional abandonment)
        #[arg(long)]
        force: bool,
    },

    /// Orient yourself at session start: project, phase, last handoff, and suggested next step
    Prime {
        /// Project name (auto-detected when only one project exists)
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Manage project context
    #[command(subcommand)]
    Project(ProjectCommands),

    /// List all wai projects across workspaces (default root: $HOME)
    #[command(
        about = "List all wai projects across workspaces (default root: $HOME)",
        long_about = "Scans for wai workspaces under a root directory (default: $HOME) and\n\
            prints a one-line summary per project showing its phase and beads issue counts.\n\n\
            EXAMPLES\n\
              wai ls                    Scan $HOME (default, depth 3)\n\
              wai ls --root ~/dev       Scan a custom root directory\n\
              wai ls --depth 2          Limit scan to 2 levels deep\n\
              wai ls --timeout 5        Stop scanning after 5 seconds"
    )]
    Ls {
        /// Root directory to scan (default: $HOME)
        #[arg(short, long)]
        root: Option<PathBuf>,

        /// Maximum scan depth (default: 3)
        #[arg(short, long)]
        depth: Option<usize>,

        /// Stop scanning after this many seconds and show results found so far (default: 10)
        #[arg(short, long, default_value_t = 10)]
        timeout: u64,
    },

    /// Ask why a decision was made (LLM-powered reasoning oracle)
    #[command(
        about = "Ask why a decision was made (LLM-powered reasoning oracle)",
        long_about = "Queries your wai artifacts using an LLM to synthesize a coherent\n\
            narrative explaining why decisions were made.\n\n\
            QUERY TYPES\n\
              Natural language question:\n\
                wai why \"why use TOML for config?\"\n\
                wai why \"what drove the microservices decision?\"\n\
                wai why \"why was error handling designed this way?\"\n\n\
              File path (explains a specific file's history):\n\
                wai why src/config.rs\n\
                wai why ./src/commands/why.rs\n\n\
            CONFIGURATION (.wai/config.toml)\n\
              [llm]\n\
              llm     = \"claude\"       # \"claude\"|\"claude-cli\"|\"agent\"|\"ollama\" (auto-detect)\n\
              model   = \"haiku\"        # Claude: \"haiku\"/\"sonnet\"; Ollama: \"llama3.1:8b\"\n\
              api_key = \"sk-ant-...\"   # Claude API key (or use ANTHROPIC_API_KEY env var)\n\
              fallback = \"search\"      # On LLM unavailable: \"search\" (default) or \"error\"\n\n\
            LLM BACKENDS\n\
              Claude     — set ANTHROPIC_API_KEY or add api_key to [llm] in .wai/config.toml\n\
              Claude CLI — install Claude Code; use llm = \"claude-cli\"\n\
              Agent      — inside agent sessions; use llm = \"agent\" or let auto-detect pick it\n\
              Ollama     — install from https://ollama.com and run a local model\n\n\
            DETECTION PRIORITY\n\
              Inside an agent session (WAI_AGENT / CLAUDECODE / CURSOR_AGENT set):  API → Agent → Ollama\n\
              Outside an agent session:                                              API → Claude CLI → Ollama\n\n\
            ERROR CODES\n\
              wai::llm::invalid_api_key  — API key missing or rejected\n\
              wai::llm::rate_limit       — Rate limit hit; wait 60s or use Ollama\n\
              wai::llm::network_error    — Network unreachable\n\
              wai::llm::model_not_found  — Ollama model not pulled; run `ollama pull <model>`\n\
              wai::llm::not_available    — No LLM configured and fallback = \"error\"\n\n\
            Falls back to `wai search` if no LLM is available. Use --no-llm to force\n\
            the fallback without an error."
    )]
    Why {
        /// Natural language question or file path to explain
        query: String,

        /// Skip the LLM and fall back to `wai search` (useful for testing or offline use)
        #[arg(long)]
        no_llm: bool,

        /// Output machine-readable JSON instead of formatted text
        #[arg(long)]
        json: bool,
    },

    /// Synthesize session context into project-specific AI guidance
    #[command(
        about = "Synthesize session context into project-specific AI guidance",
        long_about = "Reads accumulated session context (handoffs, research, optional conversation\n\
            transcript) and asks an LLM to extract project-specific conventions, gotchas,\n\
            and patterns that AI assistants should know. Injects the result into CLAUDE.md\n\
            and/or AGENTS.md as a persistent WAI:REFLECT block.\n\n\
            USAGE\n\
              wai reflect                        Auto-detect project and output targets\n\
              wai reflect --conversation chat.md Include conversation transcript as richest input\n\
              wai reflect --output agents.md     Write only to AGENTS.md\n\
              wai reflect --dry-run              Show what would change without writing\n\
              wai reflect --yes                  Skip confirmation prompt\n\n\
            OUTPUT TARGETS\n\
              claude.md  — Write to CLAUDE.md only\n\
              agents.md  — Write to AGENTS.md only\n\
              both       — Write to both CLAUDE.md and AGENTS.md\n\
              (default: whichever target files already exist in the repo root)\n\n\
            CONTEXT SOURCES (ranked by richness)\n\
              1. Conversation transcript (--conversation <file>) — raw session detail\n\
              2. Handoff artifacts — session summaries and next steps\n\
              3. Research/design/plan artifacts — curated decisions\n\n\
            Reuses the [llm] config from .wai/config.toml — no separate setup."
    )]
    Reflect(crate::commands::reflect::ReflectArgs),

    /// File an issue against wai's upstream repo, with context attached.
    ///
    /// Gathers an environment context bundle, redacts secrets/paths, and files
    /// a GitHub issue via `gh` (with a fallback ladder). Use --dry-run to
    /// preview the title/body/labels and the exact `gh` line.
    Feedback(crate::commands::feedback::FeedbackArgs),

    /// Manage pipelines (ordered multi-step workflows)
    #[command(
        about = "Manage pipelines (ordered multi-step workflows)",
        long_about = "Pipelines chain prompt-driven steps into ordered workflows, tracking run\n\
            state and auto-tagging artifacts with the run ID.\n\n\
            EXAMPLES\n\
              wai pipeline init my-workflow\n\
              wai pipeline init tdd-ro5        # built-in template\n\
              wai pipeline init epic-orchestrator  # orchestrator + subagents loop\n\
              wai pipeline start my-workflow --topic=auth-refactor\n\
              wai pipeline next\n\
              wai pipeline current\n\
              wai pipeline suggest \"auth login\"\n\n\
            STATE FILE\n\
              `wai pipeline start` writes the active run ID to .wai/resources/pipelines/.last-run so\n\
              `wai add` picks it up automatically — no export needed.\n\n\
            ENVIRONMENT (optional override)\n\
              WAI_PIPELINE_RUN  When set, overrides the state file. Useful for running\n\
                                `wai add` from a subshell or script:\n\
                                  export WAI_PIPELINE_RUN=review-2026-02-25-my-feature"
    )]
    #[command(subcommand)]
    Pipeline(PipelineCommands),

    /// Manage the decision matrix (Design in Practice) for a project
    ///
    /// A plain directory tree under .wai/projects/<project>/designs/matrix/
    /// where approaches are directories, criteria are subdirectories, each
    /// cell is a fact.md plus one judgment marker, and problem.md anchors
    /// the matrix to the decision being made. The filesystem is the source
    /// of truth — fill cells by writing files directly.
    #[command(subcommand)]
    Matrix(MatrixCommands),

    /// Inspect and manage decision artifacts
    #[command(subcommand)]
    Artifacts(ArtifactsCommands),

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, powershell, elvish)
        shell: clap_complete::Shell,
    },

    /// Pass-through to plugin commands (e.g., wai beads list)
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Subcommand)]
pub enum ArtifactsCommands {
    /// Report stale and untracked decision artifacts
    Stale {
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum MatrixCommands {
    /// Initialize the decision matrix for a project
    ///
    /// Scaffolds problem.md (the decision being deliberated), criteria/,
    /// approaches/01-status-quo/, and an empty decision.md template.
    Init {
        /// The problem statement (A1): what decision are you trying to make?
        problem: String,

        /// Project name (overrides WAI_PROJECT; auto-detects when one project exists)
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Manage matrix criteria (rows)
    #[command(subcommand)]
    Criterion(MatrixCriterionCommands),

    /// Manage matrix approaches (columns)
    #[command(subcommand)]
    Approach(MatrixApproachCommands),

    /// Record the decision: write decision.md + scaffold a design doc
    ///
    /// Validates the approach directory exists, writes decision.md (selected
    /// approach, rationale, UTC timestamp, design doc pointer), and scaffolds
    /// designs/<date>-<slug>.md with a decision-time snapshot of the winning
    /// column's facts.
    Decide {
        /// Approach to select (e.g. "02-event-sourcing" or "event-sourcing")
        approach: String,

        /// Why this approach won — recorded verbatim
        rationale: String,

        /// Project name (overrides WAI_PROJECT; auto-detects when one project exists)
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Render the matrix as self-contained matrix.html (on demand)
    ///
    /// Pure function of the directory state: problem.md banner, criteria as
    /// rows, approaches as columns, judgment chips, assessment key. Never
    /// committed — regenerate any time.
    Render {
        /// Project name (overrides WAI_PROJECT; auto-detects when one project exists)
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Lint the matrix: structural errors + methodology warnings
    ///
    /// Structural (non-zero exit): rectangularity, one marker per filled
    /// cell, no empty fact.md, status-quo first. Methodology warnings (never
    /// block): all-green column, undistinguished columns, judgment-in-text,
    /// link-only cells, criteria-as-questions, status-quo-without-red,
    /// decided-with-unfilled-cells, empty problem.md, stale decision.
    Lint {
        /// Project name (overrides WAI_PROJECT; auto-detects when one project exists)
        #[arg(short, long)]
        project: Option<String>,
    },
}

impl MatrixCommands {
    /// `--project` flag shared by every leaf subcommand.
    pub fn project(&self) -> Option<&str> {
        match self {
            MatrixCommands::Init { project, .. } => project.as_deref(),
            MatrixCommands::Criterion(cmds) => match cmds {
                MatrixCriterionCommands::Add { project, .. } => project.as_deref(),
            },
            MatrixCommands::Approach(cmds) => match cmds {
                MatrixApproachCommands::Add { project, .. } => project.as_deref(),
            },
            MatrixCommands::Decide { project, .. } => project.as_deref(),
            MatrixCommands::Render { project } => project.as_deref(),
            MatrixCommands::Lint { project } => project.as_deref(),
        }
    }
}

#[derive(Subcommand)]
pub enum MatrixCriterionCommands {
    /// Add a criterion: definition file + an empty cell in every approach
    Add {
        /// Criterion name (e.g. "operational-cost")
        name: String,

        /// Project name (overrides WAI_PROJECT; auto-detects when one project exists)
        #[arg(short, long)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum MatrixApproachCommands {
    /// Add an approach: directory with _description.md + a cell for every criterion
    Add {
        /// Approach name (e.g. "event-sourcing")
        name: String,

        /// Project name (overrides WAI_PROJECT; auto-detects when one project exists)
        #[arg(short, long)]
        project: Option<String>,
    },
}

/// Kind of feedback an agent or human can file via `wai feedback`.
///
/// The verb is `feedback` (not `report`) — `report` is reserved in pretender
/// and espectacular. Each kind maps to a label set applied to the filed issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum FeedbackKind {
    /// A defect: something crashed, hung, or produced wrong output.
    Bug,
    /// A usability or workflow papercut (no defect, just friction).
    Friction,
    /// Missing, stale, or misleading documentation.
    #[value(name = "docs-gap")]
    DocsGap,
    /// A gap in the AIX (agent-instructions) surface — managed blocks, AGENTS.md, etc.
    #[value(name = "aix-gap")]
    AixGap,
    /// A feature idea or enhancement request.
    Idea,
}

impl FeedbackKind {
    /// The label set applied to the filed issue for this kind.
    pub fn labels(self) -> &'static [&'static str] {
        match self {
            FeedbackKind::Bug => &["bug", "feedback"],
            FeedbackKind::Friction => &["friction", "feedback"],
            FeedbackKind::DocsGap => &["docs", "feedback"],
            FeedbackKind::AixGap => &["aix", "feedback"],
            FeedbackKind::Idea => &["idea", "feedback"],
        }
    }

    /// A short human label for the kind, usable in derived issue titles.
    pub fn as_word(self) -> &'static str {
        match self {
            FeedbackKind::Bug => "bug",
            FeedbackKind::Friction => "friction",
            FeedbackKind::DocsGap => "docs-gap",
            FeedbackKind::AixGap => "aix-gap",
            FeedbackKind::Idea => "idea",
        }
    }
}

#[derive(Parser)]
pub struct MoveArgs {
    /// Item name to move
    pub item: String,

    /// Target category (archives, projects, areas, resources)
    pub target: String,
}

#[derive(clap::Args)]
pub struct PhaseArgs {
    /// Project name (overrides WAI_PROJECT env var; auto-detected when only one project exists)
    #[arg(short, long, global = true)]
    pub project: Option<String>,

    #[command(subcommand)]
    pub command: Option<PhaseCommands>,
}

#[derive(Subcommand)]
#[command(allow_external_subcommands = true)]
pub enum ProjectCommands {
    /// Set WAI_PROJECT for the current shell session
    ///
    /// Prints the appropriate export statement for your shell
    /// (override with --shell posix|fish; bash/zsh/sh count as posix).
    /// Use with eval: eval $(wai project use my-project)
    Use {
        /// Project name (omit to list available projects)
        name: Option<String>,

        /// Explicit shell override (posix or fish); beats SHELL detection
        #[arg(long)]
        shell: Option<String>,
    },

    /// Catch-all for wrong-order detection (e.g., `wai project new` → `wai new project`)
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Subcommand)]
pub enum PhaseCommands {
    /// Advance to the next phase
    Next,

    /// Set a specific phase
    Set {
        /// Target phase (research, design, plan, implement, review, archive)
        phase: String,
    },

    /// Go back to the previous phase
    Back,

    /// Show current phase (default when no subcommand)
    Show,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Add a config file (skill, rule, or context)
    Add {
        /// Type of config (skill, rule, context)
        config_type: String,

        /// File to add
        file: String,
    },

    /// List all config files
    List,

    /// Edit a config file in $EDITOR
    Edit {
        /// Path to config file (relative to agent-config dir, e.g. skills/my-skill.md)
        path: String,
    },
}

#[derive(Subcommand)]
pub enum HandoffCommands {
    /// Create a handoff document for a project
    Create {
        /// Project name
        project: String,
    },
}

#[derive(Subcommand)]
pub enum PluginCommands {
    /// List all plugins
    List,

    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },

    /// Manage plugin trust
    ///
    /// Approve a plugin's hooks: wai plugin trust <name>
    /// List approved digests: wai plugin trust --list
    /// Revoke approval:     wai plugin trust --revoke <digest>
    Trust {
        /// Plugin name to approve (omit with --list or --revoke)
        name: Option<String>,

        /// List all approved hook digests
        #[arg(long, conflicts_with_all = ["name", "revoke", "hook"])]
        list: bool,

        /// Revoke approval for a specific digest
        #[arg(long, value_name = "DIGEST", conflicts_with = "hook")]
        revoke: Option<String>,

        /// Approve only this specific hook (e.g. "on_status")
        #[arg(long, requires = "name")]
        hook: Option<String>,
    },
}

/// Returns the names of all top-level wai subcommands, derived from the [`Cli`] struct.
///
/// Used by typo detection in `run_external` so the list automatically stays in sync with
/// the `Commands` enum — no manual update needed when adding a new subcommand.
pub fn wai_subcommand_names() -> Vec<String> {
    Cli::command()
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect()
}

/// Derive all valid (verb, noun) subcommand patterns from the CLI struct.
///
/// Used by wrong-order detection in `run_external` — e.g. detects `wai research add`
/// and suggests `wai add research`. Derived automatically so no manual update is
/// needed when subcommands are added or renamed.
pub fn wai_subcommand_patterns() -> Vec<(String, String)> {
    Cli::command()
        .get_subcommands()
        .flat_map(|cmd| {
            let verb = cmd.get_name().to_string();
            cmd.get_subcommands()
                .map(move |sub| (verb.clone(), sub.get_name().to_string()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Build the [`genesis::guide::Guide`] scaffold for wai.
///
/// Assembles the tool name/version, one-line description, the command
/// registry (used for typo detection in `run_external`), and flags wai as a
/// config-bearing tool so the Guide and `config::default_registry()` agree.
/// Constructed once at startup and threaded through `commands::run`.
pub fn build_guide() -> genesis::guide::Guide {
    let names = wai_subcommand_names();
    let names_ref: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    genesis::guide::Guide::builder("wai", env!("CARGO_PKG_VERSION"))
        .about("Workflow manager for AI-driven development")
        .commands(&names_ref)
        .config::<crate::config::ProjectConfig>()
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_list_contains_all_known_commands() {
        let names = wai_subcommand_names();
        let expected = &[
            "new", "add", "show", "move", "init", "status", "phase", "sync", "config", "handoff",
            "search", "timeline", "plugin", "doctor", "way", "why", "import", "resource",
            "tutorial", "close", "prime", "ls", "reflect", "pipeline",
        ];
        for cmd in expected {
            assert!(
                names.iter().any(|n| n == cmd),
                "command '{cmd}' missing from derived list; was it removed from Commands?"
            );
        }
    }

    #[test]
    fn derived_list_excludes_external_catchall() {
        let names = wai_subcommand_names();
        assert!(
            !names.iter().any(|n| n == "external"),
            "external catch-all should not appear as a named command"
        );
    }

    #[test]
    fn build_guide_assembles_wai_scaffold() {
        let guide = build_guide();
        assert_eq!(guide.name(), "wai");
        assert_eq!(guide.version(), env!("CARGO_PKG_VERSION"));
        // The guide's command registry is populated with the derived subcommand
        // names and registered under the "wai" tool name (used for typo detection).
        let registered = guide.registry().for_tool("wai");
        let names = wai_subcommand_names();
        assert!(!names.is_empty());
        for cmd in &names {
            assert!(
                registered.contains(&cmd.as_str()),
                "command '{cmd}' should be registered in the guide's registry"
            );
        }
    }

    #[test]
    fn derived_patterns_contains_known_pairs() {
        let patterns = wai_subcommand_patterns();
        let expected: &[(&str, &str)] = &[
            ("new", "project"),
            ("new", "area"),
            ("new", "resource"),
            ("add", "research"),
            ("add", "plan"),
            ("add", "design"),
            ("add", "skill"),
            ("phase", "next"),
            ("phase", "set"),
            ("phase", "back"),
            ("pipeline", "list"),
            ("resource", "add"),
            ("config", "list"),
        ];
        for (verb, noun) in expected {
            assert!(
                patterns.iter().any(|(v, n)| v == verb && n == noun),
                "pattern ({verb:?}, {noun:?}) missing from derived patterns"
            );
        }
    }
}
