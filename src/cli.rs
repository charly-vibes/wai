use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("WAI_GIT_COMMIT"),
    "-",
    env!("WAI_GIT_BRANCH"),
    env!("WAI_GIT_DIRTY"),
    ")"
);

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
    after_help = "Run 'wai <command> --help' for more information on a command."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Increase output verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output machine-readable JSON
    #[arg(long, global = true)]
    pub json: bool,

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
    #[command(subcommand)]
    Phase(PhaseCommands),

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
    },

    /// Manage agent configuration files
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Generate handoff documents
    #[command(subcommand)]
    Handoff(HandoffCommands),

    /// Search across all artifacts
    Search {
        /// Search query (supports regex with --regex flag)
        query: String,

        /// Filter by artifact type (research, plan, design, handoff)
        #[arg(long = "type")]
        type_filter: Option<String>,

        /// Search within a specific project
        #[arg(long = "in")]
        project: Option<String>,

        /// Treat query as a regular expression
        #[arg(long)]
        regex: bool,

        /// Limit number of results shown
        #[arg(short = 'n', long)]
        limit: Option<usize>,

        /// Filter by tag (frontmatter-based; repeatable)
        #[arg(long)]
        tag: Vec<String>,

        /// Return only the most recently dated match
        #[arg(long)]
        latest: bool,

        /// Number of surrounding context lines to show (like grep -C)
        #[arg(short = 'C', long = "context", default_value_t = 0)]
        context: usize,
    },

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
    },

    /// Orient yourself at session start: project, phase, last handoff, and suggested next step
    Prime {
        /// Project name (auto-detected when only one project exists)
        #[arg(short, long)]
        project: Option<String>,
    },

    /// List all wai projects across workspaces
    #[command(
        about = "List all wai projects across workspaces",
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
              Agent      — inside Claude Code sessions; use llm = \"agent\" or let auto-detect pick it\n\
              Ollama     — install from https://ollama.com and run a local model\n\n\
            DETECTION PRIORITY\n\
              Inside a Claude Code session (CLAUDECODE set):   API → Agent → Ollama\n\
              Outside a Claude Code session:                   API → Claude CLI → Ollama\n\n\
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
    Reflect {
        /// Project name (auto-detected when only one project exists)
        #[arg(short, long)]
        project: Option<String>,

        /// Path to a plain-text conversation transcript (highest-priority context)
        #[arg(short, long, value_name = "FILE")]
        conversation: Option<PathBuf>,

        /// Output target: claude.md, agents.md, or both (default: auto-detect)
        #[arg(short, long, value_name = "TARGET")]
        output: Option<String>,

        /// Show what would change without writing
        #[arg(long)]
        dry_run: bool,

        /// Skip the confirmation prompt and write directly
        #[arg(short, long)]
        yes: bool,

        /// Inject pre-generated content directly (skips LLM call).
        /// Used in agent mode: the agent calls `wai reflect --inject-content "..."` after
        /// receiving the context block from an initial `wai reflect` run.
        #[arg(long, value_name = "CONTENT")]
        inject_content: Option<String>,
    },

    /// Manage pipelines (ordered multi-skill workflows)
    #[command(
        about = "Manage pipelines (ordered multi-skill workflows)",
        long_about = "Pipelines chain skills into ordered stages, tracking run state and\n\
            auto-tagging artifacts with the run ID.\n\n\
            EXAMPLES\n\
              wai pipeline create review --stages=\"issue/gather:research,impl/run:plan\"\n\
              wai pipeline run review --topic=my-feature\n\
              wai pipeline advance <run-id>\n\
              wai pipeline status review\n\n\
            STATE FILE\n\
              `wai pipeline run` writes the active run ID to .wai/.pipeline-run so\n\
              `wai add` picks it up automatically — no export needed. The file is\n\
              removed when `wai pipeline advance` completes the last stage.\n\n\
            ENVIRONMENT (optional override)\n\
              WAI_PIPELINE_RUN  When set, overrides the state file. Useful for running\n\
                                `wai add` from a subshell or script:\n\
                                  export WAI_PIPELINE_RUN=review-2026-02-25-my-feature"
    )]
    #[command(subcommand)]
    Pipeline(PipelineCommands),

    /// Pass-through to plugin commands (e.g., wai beads list)
    #[command(external_subcommand)]
    External(Vec<String>),
}

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
    },
}

#[derive(Parser)]
pub struct MoveArgs {
    /// Item name to move
    pub item: String,

    /// Target category (archives, projects, areas, resources)
    pub target: String,
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
    /// Built-in templates: gather, create, tdd, rule-of-5
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
    /// Create a new pipeline with ordered stages
    ///
    /// Each stage is specified as "skill:artifact-type". The skill name must
    /// exist in the project's skills directory. Artifact type is a label for
    /// the expected output (e.g. research, plan, design).
    ///
    /// EXAMPLE
    ///   wai pipeline create review \
    ///     --stages="issue/gather:research,impl/run:plan,impl/review:design"
    Create {
        /// Pipeline name
        name: String,

        /// Ordered stages as "skill:artifact-type,..." pairs
        ///
        /// Each pair is "skill-name:artifact-type" separated by commas.
        /// Example: "issue/gather:research,impl/run:plan"
        #[arg(long)]
        stages: String,
    },

    /// Start a new pipeline run
    ///
    /// Generates a run ID of the form "<pipeline>-<date>-<topic>" and persists
    /// initial run state. Outputs the run ID and a hint to set WAI_PIPELINE_RUN.
    ///
    /// ENVIRONMENT
    ///   After running, set WAI_PIPELINE_RUN to enable automatic artifact tagging:
    ///     export WAI_PIPELINE_RUN=<run-id>
    ///   Then use `wai add research/plan/design` — artifacts are tagged automatically.
    Run {
        /// Pipeline name
        name: String,

        /// Topic slug for this run (used in the run ID)
        #[arg(long)]
        topic: String,
    },

    /// Advance to the next stage of a pipeline run
    ///
    /// Marks the current stage complete (recording the artifact path if a
    /// pipeline-run-tagged artifact is found), then outputs a hint for the
    /// next stage. Errors if the run ID is unknown or all stages are already done.
    Advance {
        /// Run ID (e.g., review-2026-02-25-my-feature)
        run_id: String,
    },

    /// Show status of a pipeline's runs
    ///
    /// Lists all runs with per-stage completion status and artifact paths.
    /// Use --run to filter to a single run.
    Status {
        /// Pipeline name
        name: String,

        /// Show detail for a single run
        #[arg(long)]
        run: Option<String>,
    },

    /// List all pipelines
    List,

    /// Scaffold a new TOML pipeline definition
    ///
    /// Creates `.wai/resources/pipelines/<name>.toml` with a minimal two-step
    /// template. Edit the prompts, then start a run with:
    ///   wai pipeline start <name> --topic=<your-topic>
    ///
    /// EXAMPLE
    ///   wai pipeline init my-workflow
    Init {
        /// Name for the new pipeline (creates <name>.toml)
        name: String,
    },
}
