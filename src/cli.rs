use clap::{Parser, Subcommand};
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

    /// Sync agent configs to tool-specific locations
    Sync {
        /// Only show sync status without modifying files
        #[arg(long)]
        status: bool,
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

    /// Diagnose workspace health
    Doctor {
        /// Automatically fix issues where possible
        #[arg(long)]
        fix: bool,
    },

    /// Check repository best practices (the wai way)
    #[command(
        about = "Check repository best practices (the wai way)",
        long_about = "Validates your repository against best practices for AI-friendly development.\n\n\
            Checks 10 areas including task runners (justfile, Makefile), git hooks (prek, pre-commit),\n\
            editor config, documentation (README, LICENSE, CONTRIBUTING, .gitignore), AI instructions\n\
            (CLAUDE.md, AGENTS.md), CI/CD configuration, and dev containers.\n\n\
            These are recommendations, not requirements — the command always exits successfully\n\
            and suggests improvements without enforcing them. Works in any directory; wai\n\
            initialization is not required.\n\n\
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
              wai ls --depth 2          Limit scan to 2 levels deep"
    )]
    Ls {
        /// Root directory to scan (default: $HOME)
        #[arg(short, long)]
        root: Option<PathBuf>,

        /// Maximum scan depth (default: 3)
        #[arg(short, long)]
        depth: Option<usize>,
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
              [why]\n\
              llm     = \"claude\"       # \"claude\"|\"claude-cli\"|\"agent\"|\"ollama\" (auto-detect)\n\
              model   = \"haiku\"        # Claude: \"haiku\"/\"sonnet\"; Ollama: \"llama3.1:8b\"\n\
              api_key = \"sk-ant-...\"   # Claude API key (or use ANTHROPIC_API_KEY env var)\n\
              fallback = \"search\"      # On LLM unavailable: \"search\" (default) or \"error\"\n\n\
            LLM BACKENDS\n\
              Claude     — set ANTHROPIC_API_KEY or add api_key to [why] in .wai/config.toml\n\
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
            Reuses the [why] LLM config from .wai/config.toml — no separate setup."
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
    },

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
        #[arg(long)]
        project: Option<String>,
    },

    /// Add a design document
    Design {
        /// Design content
        content: Option<String>,

        /// Import from file
        #[arg(short, long)]
        file: Option<String>,

        /// Associate with a project
        #[arg(long)]
        project: Option<String>,
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

    /// Import resources
    #[command(subcommand)]
    Import(ResourceImportCommands),
}

#[derive(Subcommand)]
pub enum ResourceAddCommands {
    /// Add a skill
    Skill {
        /// Skill name
        name: String,
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
}
