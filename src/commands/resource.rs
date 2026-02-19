use miette::Result;

use crate::cli::{ResourceAddCommands, ResourceImportCommands, ResourceListCommands};

pub fn run_add(cmd: ResourceAddCommands) -> Result<()> {
    match cmd {
        ResourceAddCommands::Skill { name } => add_skill(&name),
    }
}

pub fn run_list(cmd: ResourceListCommands) -> Result<()> {
    match cmd {
        ResourceListCommands::Skills { json } => list_skills(json),
    }
}

pub fn run_import(cmd: ResourceImportCommands) -> Result<()> {
    match cmd {
        ResourceImportCommands::Skills { from } => import_skills(from),
    }
}

fn add_skill(name: &str) -> Result<()> {
    miette::bail!("Add skill '{}' not yet implemented", name);
}

fn list_skills(json: bool) -> Result<()> {
    if json {
        miette::bail!("List skills (JSON) not yet implemented");
    } else {
        miette::bail!("List skills not yet implemented");
    }
}

fn import_skills(from: Option<String>) -> Result<()> {
    let path = from.unwrap_or_else(|| ".".to_string());
    miette::bail!("Import skills from '{}' not yet implemented", path);
}
