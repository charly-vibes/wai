use cliclack::log;
use miette::{IntoDiagnostic, Result};

use crate::cli::NewCommands;
use crate::config::{
    project_path, area_path,
    RESEARCH_DIR, PLANS_DIR, DESIGNS_DIR, HANDOFFS_DIR, STATE_FILE,
};
use crate::error::WaiError;
use crate::state::ProjectState;

use super::require_project;

pub fn run(cmd: NewCommands) -> Result<()> {
    let project_root = require_project()?;

    match cmd {
        NewCommands::Project { name, template: _ } => {
            let proj_dir = project_path(&project_root, &name);

            if proj_dir.exists() {
                return Err(WaiError::ProjectExists {
                    path: proj_dir.display().to_string(),
                }
                .into());
            }

            // Create project directory structure
            std::fs::create_dir_all(proj_dir.join(RESEARCH_DIR)).into_diagnostic()?;
            std::fs::create_dir_all(proj_dir.join(PLANS_DIR)).into_diagnostic()?;
            std::fs::create_dir_all(proj_dir.join(DESIGNS_DIR)).into_diagnostic()?;
            std::fs::create_dir_all(proj_dir.join(HANDOFFS_DIR)).into_diagnostic()?;

            // Initialize state file
            let state = ProjectState::default();
            state.save(&proj_dir.join(STATE_FILE))?;

            log::success(format!("Created project '{}'", name)).into_diagnostic()?;
            println!("  → wai phase              View current phase");
            println!("  → wai add research ...   Add research notes");
            Ok(())
        }
        NewCommands::Area { name } => {
            let area_dir = area_path(&project_root, &name);

            if area_dir.exists() {
                return Err(WaiError::ProjectExists {
                    path: area_dir.display().to_string(),
                }
                .into());
            }

            std::fs::create_dir_all(area_dir.join(RESEARCH_DIR)).into_diagnostic()?;
            std::fs::create_dir_all(area_dir.join(PLANS_DIR)).into_diagnostic()?;

            log::success(format!("Created area '{}'", name)).into_diagnostic()?;
            Ok(())
        }
        NewCommands::Resource { name } => {
            let res_dir = crate::config::resource_path(&project_root, &name);

            if res_dir.exists() {
                return Err(WaiError::ProjectExists {
                    path: res_dir.display().to_string(),
                }
                .into());
            }

            std::fs::create_dir_all(&res_dir).into_diagnostic()?;

            log::success(format!("Created resource '{}'", name)).into_diagnostic()?;
            Ok(())
        }
    }
}
