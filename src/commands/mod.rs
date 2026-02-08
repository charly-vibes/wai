use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::cli::{Cli, Commands};
use crate::config::find_project_root;
use crate::error::WaiError;

mod add;
mod config_cmd;
mod handoff;
mod import;
mod init;
mod move_cmd;
mod new;
mod phase;
mod plugin;
mod search;
mod show;
mod status;
mod sync;
mod timeline;

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Init { name }) => init::run(name),
        Some(Commands::Status) => status::run(),
        Some(Commands::New(cmd)) => new::run(cmd),
        Some(Commands::Add(cmd)) => add::run(cmd),
        Some(Commands::Show { name }) => show::run(name),
        Some(Commands::Move(args)) => move_cmd::run(args),
        Some(Commands::Phase(sub)) => phase::run(sub),
        Some(Commands::Sync { status }) => sync::run(status),
        Some(Commands::Config(cmd)) => config_cmd::run(cmd),
        Some(Commands::Handoff(cmd)) => handoff::run(cmd),
        Some(Commands::Search {
            query,
            type_filter,
            project,
            regex,
            limit,
        }) => search::run(query, type_filter, project, regex, limit),
        Some(Commands::Timeline { project, from, to, reverse }) => timeline::run(project, from, to, reverse),
        Some(Commands::Plugin(cmd)) => plugin::run(cmd),
        Some(Commands::Import { path }) => import::run(path),
        Some(Commands::External(args)) => run_external(args),
        None => show_welcome(),
    }
}

fn show_welcome() -> Result<()> {
    use cliclack::{intro, outro};

    intro("wai - Workflow manager for AI-driven development").into_diagnostic()?;

    if find_project_root().is_none() {
        println!();
        println!(
            "  {} No project detected in current directory.",
            "○".dimmed()
        );
        println!();
        println!(
            "  {} wai init           Initialize in current directory",
            "→".cyan()
        );
        println!(
            "  {} wai new project    Create a new project",
            "→".cyan()
        );
        println!("  {} wai --help         Show all commands", "→".cyan());
    } else {
        println!();
        println!(
            "  {} wai status         Check project status",
            "→".cyan()
        );
        println!(
            "  {} wai phase          Show current project phase",
            "→".cyan()
        );
        println!(
            "  {} wai new project    Create a new project",
            "→".cyan()
        );
    }

    outro("Run 'wai <command> --help' for detailed usage").into_diagnostic()?;
    Ok(())
}

fn run_external(args: Vec<String>) -> Result<()> {
    let project_root = require_project()?;

    let plugin_name = &args[0];
    let command_name = args.get(1).map(|s| s.as_str());

    let plugins = crate::plugin::detect_plugins(&project_root);

    if let Some(cmd_name) = command_name
        && let Some(cmd) = crate::plugin::find_plugin_command(&plugins, plugin_name, cmd_name)
    {
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

pub fn require_project() -> Result<std::path::PathBuf> {
    find_project_root()
        .ok_or_else(|| WaiError::NotInitialized.into())
}
