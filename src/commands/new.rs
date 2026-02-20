use cliclack::log;
use miette::{IntoDiagnostic, Result};

use crate::cli::NewCommands;
use crate::config::{
    DESIGNS_DIR, HANDOFFS_DIR, PLANS_DIR, RESEARCH_DIR, STATE_FILE, area_path, project_path,
};
use crate::context::require_safe_mode;
use crate::error::WaiError;
use crate::guided_flows;
use crate::json::Suggestion;
use crate::plugin;
use crate::state::ProjectState;

use super::{print_suggestions, require_project};

pub fn run(cmd: NewCommands) -> Result<()> {
    let project_root = require_project()?;

    match cmd {
        NewCommands::Project { name, template: _ } => {
            require_safe_mode("create project")?;
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

            plugin::run_hooks(&project_root, "on_project_create");

            log::success(format!("Created project '{}'", name)).into_diagnostic()?;

            // Post-command suggestions for new project
            let suggestions = vec![
                Suggestion {
                    label: "Add research".to_string(),
                    command: "wai add research \"...\"".to_string(),
                },
                Suggestion {
                    label: "Check project phase".to_string(),
                    command: "wai phase".to_string(),
                },
                Suggestion {
                    label: "Check status".to_string(),
                    command: "wai status".to_string(),
                },
            ];
            print_suggestions(&suggestions);

            // Show guided flows for first-time users
            guided_flows::enhanced_init_guidance(&name)?;
            let _ = guided_flows::first_research_walkthrough();
            let _ = guided_flows::first_phase_walkthrough();

            Ok(())
        }
        NewCommands::Area { name } => {
            require_safe_mode("create area")?;
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
            require_safe_mode("create resource")?;
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
