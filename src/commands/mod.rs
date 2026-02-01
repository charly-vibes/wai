use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::cli::{Cli, Commands};
use crate::config::find_project_root;
use crate::error::WaiError;

mod init;
mod status;

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Init { name }) => init::run(name),
        Some(Commands::Status) => status::run(),
        Some(Commands::New(cmd)) => handle_new(cmd),
        Some(Commands::Add(cmd)) => handle_add(cmd),
        Some(Commands::Show(cmd)) => handle_show(cmd),
        Some(Commands::Move(cmd)) => handle_move(cmd),
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
        println!("  {} wai init           Initialize in current directory", "→".cyan());
        println!("  {} wai new project    Create a new project", "→".cyan());
        println!("  {} wai --help         Show all commands", "→".cyan());
    } else {
        println!();
        println!("  {} wai status         Check project status", "→".cyan());
        println!("  {} wai show beads     List all beads", "→".cyan());
        println!("  {} wai new bead       Create a new work unit", "→".cyan());
    }

    outro("Run 'wai <command> --help' for detailed usage").into_diagnostic()?;
    Ok(())
}

fn handle_new(cmd: crate::cli::NewCommands) -> Result<()> {
    use crate::cli::NewCommands;

    match cmd {
        NewCommands::Project { name, template } => {
            println!("Creating project: {} (template: {:?})", name, template);
            // TODO: Implement project creation
            Ok(())
        }
        NewCommands::Bead { title, bead_type } => {
            require_project()?;
            println!("Creating bead: {} (type: {})", title, bead_type);
            // TODO: Implement bead creation
            Ok(())
        }
    }
}

fn handle_add(cmd: crate::cli::AddCommands) -> Result<()> {
    use crate::cli::AddCommands;

    require_project()?;

    match cmd {
        AddCommands::Research { content, bead } => {
            println!("Adding research: {} (bead: {:?})", content, bead);
            // TODO: Implement research addition
            Ok(())
        }
        AddCommands::Plugin { name } => {
            println!("Adding plugin: {}", name);
            // TODO: Implement plugin addition
            Ok(())
        }
    }
}

fn handle_show(cmd: crate::cli::ShowCommands) -> Result<()> {
    use crate::cli::ShowCommands;

    require_project()?;

    match cmd {
        ShowCommands::Project => {
            println!("Showing project info...");
            // TODO: Implement project display
            Ok(())
        }
        ShowCommands::Beads { phase } => {
            println!("Showing beads (phase: {:?})...", phase);
            // TODO: Implement beads display
            Ok(())
        }
        ShowCommands::Phase => {
            println!("Showing current phase...");
            // TODO: Implement phase display
            Ok(())
        }
    }
}

fn handle_move(cmd: crate::cli::MoveCommands) -> Result<()> {
    use crate::cli::MoveCommands;

    require_project()?;

    match cmd {
        MoveCommands::Bead { id, to } => {
            println!("Moving bead {} to phase {}", id, to);
            // TODO: Implement bead movement
            Ok(())
        }
    }
}

fn require_project() -> Result<()> {
    if find_project_root().is_none() {
        return Err(WaiError::NotInitialized.into());
    }
    Ok(())
}
