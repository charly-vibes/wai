use clap::{Args, CommandFactory, Parser, Subcommand};
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

        /// Include bd memories in search results
        #[arg(long)]
        include_memories: bool,
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

        /// Prompt for a short insight to save to bd memories
        #[arg(long)]
        remember: bool,
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

        /// Store top-level bullet points from the generated reflection as bd memories
        #[arg(long)]
        save_memories: bool,
    },

    /// Manage pipelines (ordered multi-step workflows)
    #[command(
        about = "Manage pipelines (ordered multi-step workflows)",
        long_about = "Pipelines chain prompt-driven steps into ordered workflows, tracking run\n\
            state and auto-tagging artifacts with the run ID.\n\n\
            EXAMPLES\n\
              wai pipeline init my-workflow\n\
              wai pipeline start my-workflow --topic=auth-refactor\n\
              wai pipeline next\n\
              wai pipeline current\n\
              wai pipeline suggest \"auth login\"\n\n\
            STATE FILE\n\
              `wai pipeline start` writes the active run ID to .wai/.pipeline-run so\n\
              `wai add` picks it up automatically — no export needed.\n\n\
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

    /// Scaffold a new agent skill file
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
    ///   `wai add` picks up the run ID automatically from `.wai/.pipeline-run`.
    Start {
        /// Name of the pipeline to start (must be a .toml file in .wai/resources/pipelines/)
        name: String,

        /// Topic to use for {topic} substitution in step prompts
        #[arg(long)]
        topic: Option<String>,
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
    Current,

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
