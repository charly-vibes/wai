//! `wai add` command: append research, plan, design, review, and skill artifacts.

use miette::Result;

use super::resource;
use crate::cli::AddCommands;

use super::require_project;

mod artifact;
mod helpers;
mod research;
mod review;
mod tags;

pub fn run(cmd: AddCommands) -> Result<()> {
    let project_root = require_project()?;

    match cmd {
        AddCommands::Research { content, file, project, tags, bead, corrects, tracks } => {
            research::handle(&project_root, content, file, project, tags, bead, corrects, tracks)
        }
        AddCommands::Plan { content, file, project, tags, corrects, tracks } => {
            artifact::handle_plan(&project_root, content, file, project, tags, corrects, tracks)
        }
        AddCommands::Design { content, file, project, tags, corrects, tracks } => {
            artifact::handle_design(&project_root, content, file, project, tags, corrects, tracks)
        }
        AddCommands::Review { content, file, project, tags, reviews, verdict, severity, produced_by, corrects } => {
            review::handle(
                &project_root, content, file, project, tags, reviews,
                verdict, severity, produced_by, corrects,
            )
        }
        AddCommands::Skill { name, template } => resource::add_skill(&name, template.as_deref()),
    }
}
