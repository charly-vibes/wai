mod archive;
mod metadata;
mod skills;
mod validation;

use miette::Result;
use std::path::PathBuf;

use crate::cli::{
    ResourceAddCommands, ResourceExportArgs, ResourceImportCommands, ResourceInstallArgs,
    ResourceListCommands,
};

// Re-export public items consumed by other modules
pub use metadata::parse_skill_frontmatter;

/// Forward `require_project` from the parent commands module so submodules
/// can call `super::require_project()`.
pub(super) fn require_project() -> Result<PathBuf> {
    super::require_project()
}

/// Called directly by `commands/add.rs` for `wai add skill`.
pub fn add_skill(name: &str, template_name: Option<&str>) -> Result<()> {
    skills::add_skill(name, template_name)
}

pub fn run_add(cmd: ResourceAddCommands) -> Result<()> {
    match cmd {
        ResourceAddCommands::Skill { name, template } => {
            eprintln!(
                "⚠ 'wai resource add skill' is deprecated. Use: wai add skill {}",
                name
            );
            skills::add_skill(&name, template.as_deref())
        }
    }
}

pub fn run_list(cmd: ResourceListCommands) -> Result<()> {
    match cmd {
        ResourceListCommands::Skills { json } => skills::list_skills(json),
    }
}

pub fn run_import(cmd: ResourceImportCommands) -> Result<()> {
    match cmd {
        ResourceImportCommands::Skills { from } => skills::import_skills(from),
        ResourceImportCommands::Archive { file, yes } => archive::import_skills_archive(&file, yes),
    }
}

pub fn run_install(args: ResourceInstallArgs) -> Result<()> {
    if args.global {
        skills::install_skill_global(&args.skill)
    } else if let Some(repo_path) = args.from_repo {
        skills::install_skill_from_repo(&args.skill, &repo_path)
    } else {
        miette::bail!(
            "Specify either --global (to install globally) or --from-repo <path> (to install from another repository)"
        )
    }
}

pub fn run_export(args: ResourceExportArgs) -> Result<()> {
    archive::export_skills(&args.skills, &args.output)
}
