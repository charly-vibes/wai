pub struct HelpContent {
    pub about: &'static str,
    pub examples: &'static [(&'static str, &'static str)],
    pub options: &'static [&'static str],
    pub advanced_options: &'static [&'static str],
    pub env_vars: &'static [(&'static str, &'static str)],
    pub internals: &'static [&'static str],
}

pub fn command_help(name: &str) -> Option<HelpContent> {
    match name {
        "status" => Some(HelpContent {
            about: "Check project status and suggest next steps",
            examples: &[
                ("wai status", "Quick project overview"),
                ("wai status -v", "Detailed status with section breakdowns"),
                ("wai status --json", "Machine-readable status output"),
            ],
            options: &[],
            advanced_options: &["--json    Output machine-readable JSON for scripting"],
            env_vars: &[
                ("NO_COLOR", "Disable colored output"),
                ("WAI_LOG", "Set log level (trace, debug, info, warn, error)"),
            ],
            internals: &[
                "Reads .wai/projects/*/.state for phase info",
                "Runs on_status plugin hooks for enrichment",
                "Scans openspec/changes/ for task progress",
            ],
        }),
        "init" => Some(HelpContent {
            about: "Initialize wai in the current directory",
            examples: &[
                ("wai init", "Initialize with directory name as project"),
                ("wai init --name my-project", "Initialize with custom name"),
            ],
            options: &[],
            advanced_options: &["--name <NAME>    Project name (defaults to directory name)"],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Creates .wai/ PARA directory structure",
                "Generates config.toml with project metadata",
                "Sets up agent-config with .projections.yml",
            ],
        }),
        "new" => Some(HelpContent {
            about: "Create a new project, area, or resource",
            examples: &[
                ("wai new project my-app", "Create a new project"),
                ("wai new area dev-standards", "Create a new area"),
                ("wai new resource cheatsheets", "Create a new resource"),
            ],
            options: &[],
            advanced_options: &["-t, --template <TPL>    Use a project template"],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Creates subdirectories under .wai/projects|areas|resources",
                "Initializes .state file with research phase for projects",
            ],
        }),
        "add" => Some(HelpContent {
            about: "Add artifacts (research, plans, designs) to a project",
            examples: &[
                (
                    "wai add research \"API design notes\"",
                    "Add research notes inline",
                ),
                (
                    "wai add research --file notes.md",
                    "Import research from file",
                ),
                ("wai add plan \"v2 migration plan\"", "Add a plan document"),
                (
                    "wai add design \"auth system design\"",
                    "Add a design document",
                ),
            ],
            options: &[],
            advanced_options: &[
                "-f, --file <PATH>       Import content from a file",
                "-p, --project <NAME>    Associate with a specific project",
                "-t, --tags <TAGS>       Comma-separated tags (merged with pipeline-run tag)",
            ],
            env_vars: &[
                ("NO_COLOR", "Disable colored output"),
                (
                    "WAI_PIPELINE_RUN",
                    "Auto-tag artifact with pipeline-run:<id> when set",
                ),
            ],
            internals: &[
                "Stores artifacts as timestamped files under project directories",
                "Tags are stored in YAML frontmatter",
                "WAI_PIPELINE_RUN tag is merged with any --tags value",
            ],
        }),
        "phase" => Some(HelpContent {
            about: "Show or change the current project phase",
            examples: &[
                ("wai phase show", "Display current phase"),
                ("wai phase next", "Advance to next phase"),
                ("wai phase back", "Return to previous phase"),
                ("wai phase set implement", "Jump to a specific phase"),
            ],
            options: &[],
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Phases: research → design → plan → implement → review → archive",
                "State persisted in .wai/projects/<name>/.state",
            ],
        }),
        "sync" => Some(HelpContent {
            about: "Sync agent configs to tool-specific locations",
            examples: &[
                ("wai sync", "Sync all agent configs"),
                (
                    "wai sync --status",
                    "Check sync status without modifying files",
                ),
            ],
            options: &[],
            advanced_options: &["--status    Only show sync status without modifying files"],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Reads .projections.yml for tool-specific mappings",
                "Copies/symlinks config files to target locations",
            ],
        }),
        "search" => Some(HelpContent {
            about: "Search across all artifacts",
            examples: &[
                ("wai search \"authentication\"", "Search all artifacts"),
                (
                    "wai search \"auth\" --type research",
                    "Search only research artifacts",
                ),
                ("wai search \"TODO\" --in my-app", "Search within a project"),
                (
                    "wai search \"api.*v2\" --regex",
                    "Search with regex pattern",
                ),
            ],
            options: &["-n, --limit <N>  Show up to N results (default: 20)"],
            advanced_options: &[
                "--type <TYPE>    Filter by artifact type (research, plan, design, handoff)",
                "--in <PROJECT>   Search within a specific project",
                "--regex          Treat query as regular expression",
                "--tag <TAG>      Filter by frontmatter tag (repeatable)",
                "--latest         Return only the most recently dated match",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Walks .wai/ directory tree recursively",
                "Uses regex crate for pattern matching",
            ],
        }),
        "show" => Some(HelpContent {
            about: "Show information about items",
            examples: &[
                ("wai show", "Show overview of all items"),
                ("wai show my-app", "Show details for a specific item"),
            ],
            options: &[],
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &["Aggregates data from projects, areas, and resources"],
        }),
        "doctor" => Some(HelpContent {
            about: "Diagnose workspace health",
            examples: &[
                ("wai doctor", "Run all health checks"),
                ("wai doctor --fix", "Automatically fix issues"),
                ("wai doctor --json", "Machine-readable diagnostics"),
            ],
            options: &[],
            advanced_options: &[
                "--fix     Automatically fix issues where possible",
                "--json    Output results as JSON",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Checks PARA directory structure integrity",
                "Validates config.toml syntax",
                "Verifies plugin tool availability in PATH",
                "Validates project .state files",
            ],
        }),
        "handoff" => Some(HelpContent {
            about: "Generate handoff documents",
            examples: &[(
                "wai handoff create my-app",
                "Generate handoff for a project",
            )],
            options: &[],
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Aggregates research, plans, and designs into a single document",
                "Includes phase history and timeline",
            ],
        }),
        "plugin" => Some(HelpContent {
            about: "Manage plugins",
            examples: &[
                ("wai plugin list", "List all available plugins"),
                ("wai plugin enable openspec", "Enable a plugin"),
                ("wai plugin disable beads", "Disable a plugin"),
            ],
            options: &[],
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Plugins are defined in .wai/plugins/",
                "Plugin detection based on marker files",
            ],
        }),
        "config" => Some(HelpContent {
            about: "Manage agent configuration files",
            examples: &[
                ("wai config list", "List all config files"),
                ("wai config add skill my-skill.md", "Add a skill config"),
                ("wai config edit skills/my-skill.md", "Edit a config file"),
            ],
            options: &[],
            advanced_options: &[],
            env_vars: &[("EDITOR", "Editor to use for wai config edit")],
            internals: &[
                "Configs stored in .wai/resources/agent-config/",
                "Types: skills, rules, context",
            ],
        }),
        "timeline" => Some(HelpContent {
            about: "View chronological timeline of artifacts",
            examples: &[
                ("wai timeline my-app", "View full project timeline"),
                (
                    "wai timeline my-app --from 2026-01-01",
                    "Timeline from a date",
                ),
                ("wai timeline my-app --reverse", "Oldest entries first"),
            ],
            options: &[],
            advanced_options: &[
                "--from <DATE>    Show entries from this date (YYYY-MM-DD)",
                "--to <DATE>      Show entries up to this date (YYYY-MM-DD)",
                "--reverse        Show oldest entries first",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Reads artifact timestamps from filenames",
                "Sorts by date, newest first by default",
            ],
        }),
        "move" => Some(HelpContent {
            about: "Move items between PARA categories",
            examples: &[
                (
                    "wai move old-project archives",
                    "Archive a completed project",
                ),
                ("wai move my-area projects", "Promote an area to a project"),
            ],
            options: &[],
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &["Moves directory between .wai/projects|areas|resources|archives"],
        }),
        "import" => Some(HelpContent {
            about: "Import existing tool configurations",
            examples: &[
                ("wai import .claude/", "Import Claude config files"),
                ("wai import .cursorrules", "Import Cursor rules"),
            ],
            options: &[],
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Copies files into .wai/resources/agent-config/",
                "Detects config type from file structure",
            ],
        }),
        "pipeline" => Some(HelpContent {
            about: "Manage pipelines (ordered multi-skill workflows)",
            examples: &[
                (
                    "wai pipeline create review --stages=\"gather:research,run:plan\"",
                    "Define a 2-stage pipeline",
                ),
                (
                    "wai pipeline run review --topic=my-feature",
                    "Start a pipeline run",
                ),
                (
                    "wai pipeline advance review-2026-02-25-my-feature",
                    "Advance to next stage",
                ),
                ("wai pipeline status review", "Show per-stage run status"),
                ("wai pipeline list", "List all pipelines"),
            ],
            options: &[],
            advanced_options: &[
                "create --stages <STAGES>    Comma-separated skill:artifact pairs",
                "run    --topic <SLUG>       Topic slug used in the run ID",
                "status --run <RUN-ID>       Filter status to a single run",
            ],
            env_vars: &[
                ("NO_COLOR", "Disable colored output"),
                (
                    "WAI_PIPELINE_RUN",
                    "Set by `wai pipeline run`; causes `wai add` to auto-tag artifacts",
                ),
            ],
            internals: &[
                "Pipeline definitions stored in .wai/resources/pipelines/<name>.yml",
                "Run state stored in .wai/resources/pipelines/<name>/runs/<id>.yml",
                "Artifact lookup uses pipeline-run:<id> frontmatter tag",
            ],
        }),
        "close" => Some(HelpContent {
            about: "Wrap up a session: create a handoff and show next steps",
            examples: &[
                ("wai close", "Close current session (auto-detects project)"),
                (
                    "wai close --project my-app",
                    "Close a specific project's session",
                ),
            ],
            options: &[],
            advanced_options: &[
                "-p, --project <NAME>    Project name (auto-detected when only one exists)",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Creates a handoff artifact under .wai/projects/<name>/",
                "Writes .wai/.pending-resume signal for wai prime to detect",
            ],
        }),
        "prime" => Some(HelpContent {
            about: "Orient yourself at session start: project, phase, last handoff, and suggested next step",
            examples: &[
                (
                    "wai prime",
                    "Orient for current session (auto-detects project)",
                ),
                (
                    "wai prime --project my-app",
                    "Orient for a specific project",
                ),
            ],
            options: &[],
            advanced_options: &[
                "-p, --project <NAME>    Project name (auto-detected when only one exists)",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Detects .wai/.pending-resume and shows RESUMING banner when present",
                "Reads the most recent handoff artifact for next-step suggestions",
            ],
        }),
        "why" => Some(HelpContent {
            about: "Ask why a decision was made (LLM-powered reasoning oracle)",
            examples: &[
                (
                    "wai why \"why use TOML for config?\"",
                    "Ask a natural language question",
                ),
                ("wai why src/config.rs", "Explain a specific file's history"),
                (
                    "wai why --no-llm \"auth design\"",
                    "Force fallback to wai search",
                ),
            ],
            options: &[],
            advanced_options: &[
                "--no-llm    Skip LLM and fall back to wai search",
                "--json      Output machine-readable JSON",
            ],
            env_vars: &[
                ("NO_COLOR", "Disable colored output"),
                ("ANTHROPIC_API_KEY", "API key for Claude LLM backend"),
            ],
            internals: &[
                "LLM backend configured via [llm] section in .wai/config.toml",
                "Backends: claude, claude-cli, agent (auto-detect), ollama",
                "Falls back to wai search when no LLM is available",
            ],
        }),
        "reflect" => Some(HelpContent {
            about: "Synthesize session context into project-specific AI guidance",
            examples: &[
                (
                    "wai reflect",
                    "Synthesize and inject into CLAUDE.md/AGENTS.md",
                ),
                ("wai reflect --dry-run", "Preview changes without writing"),
                (
                    "wai reflect --conversation chat.md",
                    "Include conversation transcript",
                ),
                ("wai reflect --output agents.md", "Write only to AGENTS.md"),
            ],
            options: &[],
            advanced_options: &[
                "-p, --project <NAME>         Project name (auto-detected when only one exists)",
                "-c, --conversation <FILE>    Path to conversation transcript (richest context)",
                "-o, --output <TARGET>        Output target: claude.md, agents.md, or both",
                "    --dry-run                Show what would change without writing",
                "-y, --yes                    Skip confirmation prompt",
            ],
            env_vars: &[
                ("NO_COLOR", "Disable colored output"),
                ("ANTHROPIC_API_KEY", "API key for Claude LLM backend"),
            ],
            internals: &[
                "Injects result into CLAUDE.md/AGENTS.md as a WAI:REFLECT block",
                "Context sources (ranked): conversation > handoffs > research/design/plan",
                "Reuses [llm] config from .wai/config.toml",
            ],
        }),
        "ls" => Some(HelpContent {
            about: "List all wai projects across workspaces",
            examples: &[
                ("wai ls", "Scan $HOME for wai workspaces (default depth 3)"),
                ("wai ls --root ~/dev", "Scan a custom root directory"),
                ("wai ls --depth 2", "Limit scan to 2 levels deep"),
                ("wai ls --timeout 5", "Stop scanning after 5 seconds"),
            ],
            options: &[],
            advanced_options: &[
                "-r, --root <PATH>    Root directory to scan (default: $HOME)",
                "-d, --depth <N>      Maximum scan depth (default: 3)",
                "-t, --timeout <S>    Stop scanning after this many seconds (default: 10)",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Walks the filesystem looking for .wai/ directories",
                "Reads phase and beads issue counts from each workspace",
            ],
        }),
        "tutorial" => Some(HelpContent {
            about: "Run the interactive quickstart tutorial",
            examples: &[("wai tutorial", "Start the interactive quickstart tutorial")],
            options: &[],
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &["Guides through init, new project, add artifact, and phase commands"],
        }),
        "resource" => Some(HelpContent {
            about: "Manage resources (skills, rules, context)",
            examples: &[
                ("wai resource add skill my-skill", "Add a new skill"),
                (
                    "wai resource add skill issue/gather --template gather",
                    "Add skill from template",
                ),
                ("wai resource list skills", "List all skills"),
                (
                    "wai resource install issue/gather --global",
                    "Install skill globally",
                ),
                (
                    "wai resource export issue/gather --output skills.tar.gz",
                    "Export skills to archive",
                ),
                (
                    "wai resource import skills --from ./other",
                    "Import skills from directory",
                ),
                (
                    "wai resource import archive skills.tar.gz",
                    "Import skills from archive",
                ),
            ],
            options: &[],
            advanced_options: &[
                "add skill --template <TPL>       Built-in templates: gather, create, tdd, rule-of-5",
                "install   --global               Install to ~/.wai/resources/skills/",
                "install   --from-repo <PATH>     Copy skill from another repository",
                "export    --output <FILE>        Output tar.gz archive path",
                "import archive --yes             Overwrite existing skills without prompting",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Skills stored in .wai/resources/agent-config/skills/<name>/SKILL.md",
                "Hierarchical skill names use one '/' separator (e.g. issue/gather)",
                "Global skills stored in ~/.wai/resources/skills/",
            ],
        }),
        "way" => Some(HelpContent {
            about: "Show repo hygiene and agent workflow conventions — skills, rules, best practices",
            examples: &[
                ("wai way", "Show hygiene status for the current repo"),
                (
                    "wai way --fix skills",
                    "Scaffold missing recommended agent skills",
                ),
                (
                    "wai way --json",
                    "Machine-readable output for CI integration",
                ),
            ],
            options: &[],
            advanced_options: &[
                "--fix <CHECK>    Scaffold missing items for a check (e.g. skills)",
                "--json           Output machine-readable JSON",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Covers 11 areas: task runners, git hooks, editor config, docs, AI instructions,",
                "  LLM context, agent skills, CI/CD, dev containers, and release pipelines",
                "Works in any directory — a wai workspace is not required",
                "Always exits successfully; shows recommendations without enforcing them",
            ],
        }),
        _ => None,
    }
}

pub fn render_main_help(verbose: u8) -> String {
    let mut out = String::new();

    out.push_str("wai /waɪ/ — know why it was built that way\n\n");

    out.push_str("QUICK START:\n");
    out.push_str("  wai init                    Initialize in current directory\n");
    out.push_str("  wai new project my-app      Create a new project\n");
    out.push_str("  wai status                  Check project status\n");
    out.push_str("  wai add research \"notes\"     Add research artifacts\n");
    out.push_str("  wai phase next              Advance project phase\n");
    out.push('\n');

    // COMMANDS: keep in alphabetical order
    out.push_str("COMMANDS:\n");
    out.push_str("  add       Add artifacts (research, plans, designs) to a project\n");
    out.push_str("  close     Wrap up a session and save handoff\n");
    out.push_str("  config    Manage agent configuration files\n");
    out.push_str("  doctor    Diagnose workspace health\n");
    out.push_str("  handoff   Generate handoff documents\n");
    out.push_str("  import    Import existing tool configurations\n");
    out.push_str("  init      Initialize wai in the current directory\n");
    out.push_str("  ls        List all wai projects across workspaces\n");
    out.push_str("  move      Move items between PARA categories\n");
    out.push_str("  new       Create a new project, area, or resource\n");
    out.push_str("  phase     Show or change the current project phase\n");
    out.push_str("  pipeline  Manage pipelines (ordered multi-skill workflows)\n");
    out.push_str("  plugin    Manage plugins\n");
    out.push_str("  prime     Orient yourself at session start\n");
    out.push_str("  reflect   Synthesize session context into AI guidance\n");
    out.push_str("  resource  Manage resources (skills, rules, context)\n");
    out.push_str("  search    Search across all artifacts\n");
    out.push_str("  show      Show information about items\n");
    out.push_str("  status    Check project status and suggest next steps\n");
    out.push_str("  sync      Sync agent configs to tool-specific locations\n");
    out.push_str("  timeline  View chronological timeline of artifacts\n");
    out.push_str("  tutorial  Run the interactive quickstart tutorial\n");
    out.push_str("  way       Check repository best practices\n");
    out.push_str("  why       Ask why a decision was made (LLM-powered)\n");
    out.push('\n');

    out.push_str("OPTIONS:\n");
    out.push_str("  -v, --verbose...  Increase output verbosity (-v, -vv, -vvv)\n");
    out.push_str("  -q, --quiet       Suppress non-error output\n");
    out.push_str("  -h, --help        Print help\n");
    out.push_str("  -V, --version     Print version\n");

    if verbose >= 1 {
        out.push_str("\nADVANCED OPTIONS:\n");
        out.push_str("      --json        Output machine-readable JSON\n");
        out.push_str("      --no-input    Disable interactive prompts\n");
        out.push_str("      --yes         Auto-confirm actions with defaults\n");
        out.push_str("      --safe        Run in read-only safe mode\n");
    }

    if verbose >= 2 {
        out.push_str("\nENVIRONMENT:\n");
        out.push_str("  NO_COLOR          Disable colored output\n");
        out.push_str("  WAI_LOG           Set log level (trace, debug, info, warn, error)\n");
        out.push_str("  EDITOR            Editor for interactive editing commands\n");
        out.push_str("  WAI_PIPELINE_RUN  Auto-tag `wai add` artifacts with pipeline-run:<id>\n");
    }

    if verbose >= 3 {
        out.push_str("\nINTERNALS:\n");
        out.push_str("  Project data stored in .wai/ using PARA method\n");
        out.push_str("  Config: .wai/config.toml\n");
        out.push_str("  State: .wai/projects/<name>/.state (YAML)\n");
        out.push_str("  Plugins: .wai/plugins/ with marker-file detection\n");
        out.push_str("  Agent configs: .wai/resources/agent-config/\n");
    }

    if verbose == 0 {
        out.push_str(
            "\nUse -v for advanced options, -vv for environment variables, -vvv for internals.\n",
        );
    }

    out.push_str("Run 'wai <command> --help' for more information on a command.\n");

    out
}

pub fn render_command_help(name: &str, verbose: u8) -> Option<String> {
    let help = command_help(name)?;

    let mut out = String::new();

    out.push_str(&format!("{}\n\n", help.about));

    if !help.examples.is_empty() {
        out.push_str("EXAMPLES:\n");
        for (cmd, desc) in help.examples {
            out.push_str(&format!("  {}{}# {}\n", cmd, padding(cmd, 38), desc));
        }
        out.push('\n');
    }

    if !help.options.is_empty() {
        out.push_str("OPTIONS:\n");
        for opt in help.options {
            out.push_str(&format!("  {}\n", opt));
        }
        out.push('\n');
    }

    if verbose >= 1 && !help.advanced_options.is_empty() {
        out.push_str("ADVANCED OPTIONS:\n");
        for opt in help.advanced_options {
            out.push_str(&format!("  {}\n", opt));
        }
        out.push('\n');
    }

    if verbose >= 2 && !help.env_vars.is_empty() {
        out.push_str("ENVIRONMENT:\n");
        for (var, desc) in help.env_vars {
            out.push_str(&format!("  {}{}  {}\n", var, padding(var, 14), desc));
        }
        out.push('\n');
    }

    if verbose >= 3 && !help.internals.is_empty() {
        out.push_str("INTERNALS:\n");
        for detail in help.internals {
            out.push_str(&format!("  {}\n", detail));
        }
        out.push('\n');
    }

    if verbose == 0 {
        out.push_str("Use -v for all options, -vv for env vars, -vvv for internals.\n");
    }

    Some(out)
}

fn padding(s: &str, target: usize) -> String {
    let len = s.len();
    if len >= target {
        "  ".to_string()
    } else {
        " ".repeat(target - len)
    }
}

pub fn try_render_help(args: &[String]) -> Option<String> {
    let has_help = args.iter().any(|a| a == "--help" || a == "-h");
    if !has_help {
        return None;
    }

    let mut verbose: u8 = 0;
    for arg in args.iter().skip(1) {
        if arg == "--verbose" {
            verbose = verbose.saturating_add(1);
        } else if arg.starts_with('-') && !arg.starts_with("--") {
            verbose = verbose.saturating_add(arg.chars().filter(|c| *c == 'v').count() as u8);
        }
    }

    let subcommand = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-') && *a != "help");

    match subcommand {
        Some(cmd) => render_command_help(cmd.as_str(), verbose),
        None => Some(render_main_help(verbose)),
    }
}
