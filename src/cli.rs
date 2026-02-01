use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "wai",
    about = "Workflow manager for AI-driven development",
    version,
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
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project or component
    #[command(subcommand)]
    New(NewCommands),

    /// Add items to the current context
    #[command(subcommand)]
    Add(AddCommands),

    /// Show information about projects, beads, or phases
    #[command(subcommand)]
    Show(ShowCommands),

    /// Move items between phases or states
    #[command(subcommand)]
    Move(MoveCommands),

    /// Initialize wai in the current directory
    Init {
        /// Project name (defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Check project status and suggest next steps
    Status,
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

    /// Create a new bead (work unit)
    Bead {
        /// Bead title
        title: String,

        /// Bead type (feature, fix, chore, etc.)
        #[arg(short = 't', long, default_value = "feature")]
        bead_type: String,
    },
}

#[derive(Subcommand)]
pub enum AddCommands {
    /// Add research notes or findings
    Research {
        /// Research content or file path
        content: String,

        /// Link to a specific bead
        #[arg(short, long)]
        bead: Option<String>,
    },

    /// Add a plugin to the project
    Plugin {
        /// Plugin name or path
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ShowCommands {
    /// Show project overview
    Project,

    /// Show all beads
    Beads {
        /// Filter by phase
        #[arg(short, long)]
        phase: Option<String>,
    },

    /// Show current phase
    Phase,
}

#[derive(Subcommand)]
pub enum MoveCommands {
    /// Move a bead to a different phase
    Bead {
        /// Bead identifier
        id: String,

        /// Target phase
        #[arg(short, long)]
        to: String,
    },
}
