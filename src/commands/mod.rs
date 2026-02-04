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
        }) => search::run(query, type_filter, project),
        Some(Commands::Timeline { project }) => timeline::run(project),
        Some(Commands::Plugin(cmd)) => plugin::run(cmd),
        Some(Commands::Import { path }) => import::run(path),
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

pub fn require_project() -> Result<std::path::PathBuf> {
    find_project_root()
        .ok_or_else(|| WaiError::NotInitialized.into())
}
