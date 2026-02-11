pub struct HelpContent {
    pub about: &'static str,
    pub examples: &'static [(&'static str, &'static str)],
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
            advanced_options: &[
                "-f, --file <PATH>       Import content from a file",
                "-p, --project <NAME>    Associate with a specific project",
                "-t, --tags <TAGS>       Add tags (research only)",
            ],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Stores artifacts as timestamped files under project directories",
                "Tags are stored in YAML frontmatter",
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
            advanced_options: &[
                "--type <TYPE>    Filter by artifact type (research, plan, design, handoff)",
                "--in <PROJECT>   Search within a specific project",
                "--regex          Treat query as regular expression",
                "-n, --limit <N>  Limit number of results",
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
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &["Aggregates data from projects, areas, and resources"],
        }),
        "doctor" => Some(HelpContent {
            about: "Diagnose workspace health",
            examples: &[
                ("wai doctor", "Run all health checks"),
                ("wai doctor --json", "Machine-readable diagnostics"),
            ],
            advanced_options: &["--json    Output results as JSON"],
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
            advanced_options: &[],
            env_vars: &[("NO_COLOR", "Disable colored output")],
            internals: &[
                "Copies files into .wai/resources/agent-config/",
                "Detects config type from file structure",
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

    out.push_str("COMMANDS:\n");
    out.push_str("  new       Create a new project, area, or resource\n");
    out.push_str("  add       Add artifacts (research, plans, designs) to a project\n");
    out.push_str("  show      Show information about items\n");
    out.push_str("  move      Move items between PARA categories\n");
    out.push_str("  init      Initialize wai in the current directory\n");
    out.push_str("  status    Check project status and suggest next steps\n");
    out.push_str("  phase     Show or change the current project phase\n");
    out.push_str("  sync      Sync agent configs to tool-specific locations\n");
    out.push_str("  config    Manage agent configuration files\n");
    out.push_str("  handoff   Generate handoff documents\n");
    out.push_str("  search    Search across all artifacts\n");
    out.push_str("  timeline  View chronological timeline of artifacts\n");
    out.push_str("  plugin    Manage plugins\n");
    out.push_str("  doctor    Diagnose workspace health\n");
    out.push_str("  import    Import existing tool configurations\n");
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
        out.push_str("  NO_COLOR    Disable colored output\n");
        out.push_str("  WAI_LOG     Set log level (trace, debug, info, warn, error)\n");
        out.push_str("  EDITOR      Editor for interactive editing commands\n");
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
