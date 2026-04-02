use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::cli::{Cli, Commands};
use crate::config::{UserConfig, find_project_root, projects_dir};
use crate::context::current_context;
use crate::error::WaiError;
use crate::suggestions::SuggestionEngine;

mod add;
mod close;
mod config_cmd;
mod doctor;
mod handoff;
mod import;
mod init;
mod ls;
mod move_cmd;
mod new;
mod phase;
mod pipeline;
mod plugin;
mod prime;
mod project;
mod reflect;
mod resource;
mod search;
mod show;
mod status;
mod sync;
mod timeline;
mod way;
mod why;

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Init { name }) => init::run(name),
        Some(Commands::Status) => status::run(cli.verbose),
        Some(Commands::New(cmd)) => new::run(cmd),
        Some(Commands::Add(cmd)) => add::run(cmd),
        Some(Commands::Show { name }) => show::run(name),
        Some(Commands::Move(args)) => move_cmd::run(args),
        Some(Commands::Phase(args)) => phase::run(args),
        Some(Commands::Sync {
            status,
            dry_run,
            from_main,
        }) => sync::run(status, dry_run, from_main),
        Some(Commands::Config(cmd)) => config_cmd::run(cmd),
        Some(Commands::Handoff(cmd)) => handoff::run(cmd),
        Some(Commands::Search {
            query,
            type_filter,
            project,
            regex,
            limit,
            tag,
            latest,
            context,
            include_memories,
        }) => search::run(search::SearchArgs {
            query,
            type_filter,
            project,
            use_regex: regex,
            limit,
            tag_filter: tag,
            latest,
            context_size: context,
            include_memories,
        }),
        Some(Commands::Timeline {
            project,
            from,
            to,
            reverse,
        }) => timeline::run(project, from, to, reverse),
        Some(Commands::Plugin(cmd)) => plugin::run(cmd),
        Some(Commands::Doctor { fix }) => doctor::run(fix),
        Some(Commands::Way { topic, fix }) => way::run(topic, fix),
        Some(Commands::Import { path }) => import::run(path),
        Some(Commands::Resource(cmd)) => match cmd {
            crate::cli::ResourceCommands::Add(add_cmd) => resource::run_add(add_cmd),
            crate::cli::ResourceCommands::List(list_cmd) => resource::run_list(list_cmd),
            crate::cli::ResourceCommands::Import(import_cmd) => resource::run_import(import_cmd),
            crate::cli::ResourceCommands::Install(args) => resource::run_install(args),
            crate::cli::ResourceCommands::Export(args) => resource::run_export(args),
        },
        Some(Commands::Pipeline(cmd)) => pipeline::run(cmd),
        Some(Commands::Close { project, remember }) => close::run(project, remember),
        Some(Commands::Prime { project }) => prime::run(project),
        Some(Commands::Project(cmd)) => project::run(cmd),
        Some(Commands::Ls {
            root,
            depth,
            timeout,
        }) => ls::run(root, depth, timeout),
        Some(Commands::Tutorial) => crate::tutorial::run(),
        Some(Commands::Why {
            query,
            no_llm,
            json,
        }) => why::run(query, no_llm, json, cli.verbose),
        Some(Commands::Reflect {
            project,
            conversation,
            output,
            dry_run,
            yes,
            inject_content,
            save_memories,
        }) => reflect::run(reflect::ReflectArgs {
            project,
            conversation,
            output,
            dry_run,
            yes,
            inject_content,
            verbose: cli.verbose,
            save_memories,
        }),
        Some(Commands::External(args)) => run_external(args),
        None => show_welcome(),
    }
}

fn welcome_suggestions(project_detected: bool) -> Vec<crate::json::Suggestion> {
    if project_detected {
        vec![
            crate::json::Suggestion {
                label: "Check project status".to_string(),
                command: "wai status".to_string(),
            },
            crate::json::Suggestion {
                label: "Show current project phase".to_string(),
                command: "wai phase".to_string(),
            },
            crate::json::Suggestion {
                label: "Create a new project".to_string(),
                command: "wai new project".to_string(),
            },
        ]
    } else {
        vec![
            crate::json::Suggestion {
                label: "Initialize in current directory".to_string(),
                command: "wai init".to_string(),
            },
            crate::json::Suggestion {
                label: "Create a new project".to_string(),
                command: "wai new project".to_string(),
            },
            crate::json::Suggestion {
                label: "Show all commands".to_string(),
                command: "wai --help".to_string(),
            },
        ]
    }
}

fn show_welcome() -> Result<()> {
    use cliclack::{intro, outro};

    // Load user config; first-run initialization (writing the default config
    // file) is handled inside UserConfig::load, keeping show_welcome read-only.
    // If load fails (corrupt config), treat as first-run with default config.
    let user_config = UserConfig::load().unwrap_or_default();
    let is_first_run = !user_config.seen_tutorial;

    let context = current_context();

    if context.quiet {
        return Ok(());
    }

    if context.json {
        let project_detected = find_project_root().is_some();
        let suggestions = welcome_suggestions(project_detected);
        let payload = crate::json::WelcomePayload {
            welcome: "wai - Workflow manager for AI-driven development".to_string(),
            project_detected,
            suggestions,
            help_hint: "Run 'wai --help' for detailed usage".to_string(),
        };
        crate::output::print_json_line(&payload)?;
        return Ok(());
    }

    intro("wai - Workflow manager for AI-driven development").into_diagnostic()?;

    if find_project_root().is_none() {
        println!();
        println!(
            "  {} No project detected in current directory.",
            "○".dimmed()
        );

        // Show example workflow for new users
        if is_first_run {
            println!();
            println!("  {} Example workflow:", "○".cyan());
            println!("     1. wai init                    Set up workspace");
            println!("     2. wai new project \"mywork\"   Create your first project");
            println!("     3. wai add research \"notes\"    Capture your research");
            println!("     4. wai phase next              Advance to next phase");
            println!("     5. wai handoff create mywork   Save your progress");
        }

        println!();
        println!(
            "  {} wai init           Initialize in current directory",
            "→".cyan()
        );
        println!(
            "  {} wai tutorial       Run the quickstart tutorial",
            "→".cyan()
        );
        println!(
            "  {} wai way            Check repo best practices",
            "→".cyan()
        );
        println!("  {} wai --help         Show all commands", "→".cyan());

        if is_first_run {
            println!(
                "  {} Getting Started: Run 'wai tutorial' to learn wai",
                "→".cyan()
            );
        } else {
            println!("  {} Run 'wai --help' for detailed usage", "•".dimmed());
        }
    } else {
        println!();
        println!("  {} wai status         Check project status", "→".cyan());
        println!(
            "  {} wai phase          Show current project phase",
            "→".cyan()
        );
        println!("  {} wai new project    Create a new project", "→".cyan());
        println!(
            "  {} wai way            Check repo best practices",
            "→".cyan()
        );
        println!("  {} Run 'wai --help' for detailed usage", "•".dimmed());
    }

    outro("Run 'wai <command> --help' for detailed usage").into_diagnostic()?;
    Ok(())
}

fn run_external(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(WaiError::PluginNotFound {
            name: "".to_string(),
        }
        .into());
    }

    let plugin_name = &args[0];
    let command_name = args.get(1).map(|s| s.as_str());

    let cmd_names = crate::cli::wai_subcommand_names();
    let valid_commands: Vec<&str> = cmd_names.iter().map(|s| s.as_str()).collect();
    let patterns_owned = crate::cli::wai_subcommand_patterns();
    let valid_patterns: Vec<(&str, &str)> = patterns_owned
        .iter()
        .map(|(v, n)| (v.as_str(), n.as_str()))
        .collect();
    let engine = SuggestionEngine::new();

    // Check for typos and wrong-order BEFORE requiring the workspace.
    // This ensures "Did you mean?" hints are shown even outside a workspace,
    // giving better context rather than just "NotInitialized".
    if let Some(second) = args.get(1)
        && let Some(suggestion) = engine.suggest_order(plugin_name, second, &valid_patterns)
    {
        miette::bail!(
            "{}. {}",
            suggestion.message(),
            "Run 'wai --help' to see available commands."
        );
    }

    // Skip typo detection if the first arg matches a detected plugin name.
    let is_known_plugin = find_project_root().is_some_and(|root| {
        crate::plugin::detect_plugins(&root)
            .iter()
            .any(|p| p.def.name == *plugin_name && p.detected)
    });

    if !is_known_plugin && let Some(suggestion) = engine.suggest_typo(plugin_name, &valid_commands)
    {
        miette::bail!(
            "{}. {}",
            suggestion.message(),
            "Run 'wai --help' to see available commands."
        );
    }

    // Typo/order checks passed — this looks like a genuine plugin or unknown command.
    // Now require the workspace so we can look up plugin definitions.
    let project_root = require_project()?;
    let plugins = crate::plugin::detect_plugins(&project_root);

    if let Some(cmd_name) = command_name
        && let Some(cmd) = crate::plugin::find_plugin_command(&plugins, plugin_name, cmd_name)
    {
        let context = current_context();
        if context.safe && !cmd.read_only {
            return Err(WaiError::SafeModeViolation {
                action: format!("{} {}", plugin_name, cmd_name),
            }
            .into());
        }
        let extra_args: Vec<String> = args[2..].to_vec();
        let status =
            crate::plugin::execute_passthrough(&project_root, &cmd.passthrough, &extra_args)
                .into_diagnostic()?;
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        return Ok(());
    }

    // Check if the plugin exists at all (even without a matching command)
    let plugin_exists = plugins
        .iter()
        .any(|p| p.def.name == *plugin_name && p.detected);

    if plugin_exists {
        if let Some(cmd_name) = command_name {
            miette::bail!(
                "Plugin '{}' has no command '{}'. Run 'wai plugin list' to see available commands.",
                plugin_name,
                cmd_name
            );
        } else {
            miette::bail!(
                "Plugin '{}' requires a command. Run 'wai plugin list' to see available commands.",
                plugin_name
            );
        }
    } else {
        miette::bail!(
            "Unknown command '{}'. Run 'wai --help' to see available commands or 'wai plugin list' to see plugins.",
            plugin_name
        );
    }
}

pub fn require_project() -> Result<PathBuf> {
    find_project_root().ok_or_else(|| WaiError::NotInitialized.into())
}

/// Parse `bd stats` output and return a compact one-liner like "3 open issues (2 ready)".
///
/// Returns `None` when the content does not contain the expected fields.
pub(crate) fn beads_summary(content: &str) -> Option<String> {
    let (o, r) = beads_counts(content)?;
    Some(format!("{} open issues ({} ready)", o, r))
}

/// Parse `bd stats` output and return `(open, ready)` counts as raw numbers.
///
/// Returns `None` when the content does not contain both expected fields.
pub(crate) fn beads_counts(content: &str) -> Option<(u64, u64)> {
    let mut open: Option<u64> = None;
    let mut ready: Option<u64> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("Open:") {
            open = val.trim().parse().ok();
        } else if let Some(val) = trimmed.strip_prefix("Ready to Work:") {
            ready = val.trim().parse().ok();
        }
    }
    if let (Some(o), Some(r)) = (open, ready) {
        Some((o, r))
    } else {
        None
    }
}

/// How a project was resolved by [`resolve_project`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectSource {
    /// Explicit `--project` CLI flag.
    Flag,
    /// `WAI_PROJECT` environment variable.
    EnvVar,
    /// Auto-detected (exactly one project exists).
    AutoDetect,
    /// User selected interactively via cliclack prompt.
    Interactive,
}

/// Result of project resolution: the project name and how it was resolved.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedProject {
    pub name: String,
    #[allow(dead_code)] // Used by wai-82uh (resolution source display)
    pub source: ProjectSource,
}

/// Unified project resolution with deterministic priority order:
///
/// 1. `explicit` — the `--project` CLI flag (highest priority)
/// 2. `WAI_PROJECT` environment variable (empty string treated as unset)
/// 3. Auto-detect when exactly one project in `.wai/projects/`
/// 4. Interactive selector when TTY available and `--no-input` not set
/// 5. Error with guidance
///
/// Validates that the resolved project directory exists.
pub(crate) fn resolve_project(
    project_root: &Path,
    explicit: Option<&str>,
) -> Result<ResolvedProject> {
    // 1. Explicit --project flag
    if let Some(name) = explicit {
        let proj_dir = projects_dir(project_root).join(name);
        if !proj_dir.exists() {
            let available = list_projects(project_root);
            let available_str = if available.is_empty() {
                "none".to_string()
            } else {
                available.join(", ")
            };
            miette::bail!(
                "Project '{}' not found. Available projects: {}",
                name,
                available_str
            );
        }
        return Ok(ResolvedProject {
            name: name.to_string(),
            source: ProjectSource::Flag,
        });
    }

    // 2. WAI_PROJECT environment variable (empty string treated as unset)
    if let Ok(env_name) = std::env::var("WAI_PROJECT")
        && !env_name.is_empty()
    {
        let proj_dir = projects_dir(project_root).join(&env_name);
        if !proj_dir.exists() {
            let available = list_projects(project_root);
            let available_str = if available.is_empty() {
                "none".to_string()
            } else {
                available.join(", ")
            };
            miette::bail!(
                "WAI_PROJECT='{}' but project not found. Available projects: {}",
                env_name,
                available_str
            );
        }
        return Ok(ResolvedProject {
            name: env_name,
            source: ProjectSource::EnvVar,
        });
    }

    // 3-5. Auto-detect / interactive / error
    let mut projects = list_projects(project_root);
    projects.sort();

    match projects.len() {
        0 => miette::bail!("No projects found. Create one with `wai new project <name>`."),
        1 => Ok(ResolvedProject {
            name: projects.remove(0),
            source: ProjectSource::AutoDetect,
        }),
        _ => {
            let ctx = current_context();
            if ctx.no_input || !std::io::stdin().is_terminal() {
                return Err(WaiError::NonInteractive {
                    message: format!(
                        "Multiple projects found ({}). Use `--project <name>` or `export WAI_PROJECT=<name>` to specify one.",
                        projects.join(", ")
                    ),
                }
                .into());
            }
            let mut sel = cliclack::select("Multiple projects found — which one?");
            for name in &projects {
                sel = sel.item(name.clone(), name.as_str(), "");
            }
            let selected: String = sel.interact().into_diagnostic()?;
            Ok(ResolvedProject {
                name: selected,
                source: ProjectSource::Interactive,
            })
        }
    }
}

/// List project directory names under `.wai/projects/`.
pub(crate) fn list_projects(project_root: &Path) -> Vec<String> {
    let dir = projects_dir(project_root);
    if !dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}

/// Print workflow suggestions after a command completes.
///
/// Displays suggestions with the format:
/// ```
/// → suggestion.label: suggestion.command
/// ```
pub fn print_suggestions(suggestions: &[crate::json::Suggestion]) {
    if suggestions.is_empty() {
        return;
    }

    if current_context().quiet {
        return;
    }

    for suggestion in suggestions {
        println!(
            "  {} {}: {}",
            "→".dimmed(),
            suggestion.label,
            suggestion.command
        );
    }
}
