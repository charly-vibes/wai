use cliclack::{intro, outro};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::config::{find_project_root, ProjectConfig};
use crate::error::WaiError;

pub fn run() -> Result<()> {
    let project_root = find_project_root().ok_or(WaiError::NotInitialized)?;
    let config = ProjectConfig::load(&project_root)?;

    intro(format!("Project: {}", config.project.name.bold())).into_diagnostic()?;

    // TODO: Load actual bead counts from .wai/beads/
    let bead_counts = BeadCounts::default();

    println!();
    println!("  {} Beads", "◆".cyan());
    println!(
        "    {} draft  {} ready  {} in-progress  {} done",
        bead_counts.draft.to_string().dimmed(),
        bead_counts.ready.to_string().yellow(),
        bead_counts.in_progress.to_string().blue(),
        bead_counts.done.to_string().green(),
    );

    println!();
    println!("  {} Suggestions", "◆".cyan());
    
    if bead_counts.total() == 0 {
        println!("    {} Create your first bead: wai new bead \"Feature name\"", "→".dimmed());
    } else if bead_counts.ready > 0 {
        println!("    {} You have {} beads ready to implement", "→".dimmed(), bead_counts.ready);
        println!("    {} Start with: wai move bead <id> --to in-progress", "→".dimmed());
    } else if bead_counts.in_progress > 0 {
        println!("    {} {} beads in progress", "→".dimmed(), bead_counts.in_progress);
    }

    outro("Run 'wai show beads' for details").into_diagnostic()?;
    Ok(())
}

#[derive(Default)]
struct BeadCounts {
    draft: usize,
    ready: usize,
    in_progress: usize,
    done: usize,
}

impl BeadCounts {
    fn total(&self) -> usize {
        self.draft + self.ready + self.in_progress + self.done
    }
}
