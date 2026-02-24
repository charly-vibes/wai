use chrono::Utc;
use cliclack::log;
use miette::{IntoDiagnostic, Result};
use std::path::{Path, PathBuf};

use crate::cli::HandoffCommands;
use crate::config::{HANDOFFS_DIR, STATE_FILE, projects_dir};
use crate::context::require_safe_mode;
use crate::error::WaiError;
use crate::plugin;
use crate::state::ProjectState;

use super::require_project;

pub fn run(cmd: HandoffCommands) -> Result<()> {
    let project_root = require_project()?;

    match cmd {
        HandoffCommands::Create { project } => {
            require_safe_mode("create handoff")?;
            let path = create_handoff(&project_root, &project)?;
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            log::success(format!(
                "Created handoff for '{}' at handoffs/{}",
                project, filename
            ))
            .into_diagnostic()?;
            Ok(())
        }
    }
}

/// Create a handoff document for the given project and return the path to the created file.
///
/// The caller is responsible for calling `require_safe_mode("create handoff")` before
/// invoking this function.
pub fn create_handoff(project_root: &Path, project: &str) -> Result<PathBuf> {
    let proj_dir = projects_dir(project_root).join(project);

    if !proj_dir.exists() {
        return Err(WaiError::ProjectNotFound {
            name: project.to_string(),
        }
        .into());
    }

    let handoffs_dir = proj_dir.join(HANDOFFS_DIR);
    std::fs::create_dir_all(&handoffs_dir).into_diagnostic()?;

    // Load project state
    let state_path = proj_dir.join(STATE_FILE);
    let state = ProjectState::load(&state_path)?;

    let now = Utc::now();
    let date = now.format("%Y-%m-%d");
    let filename = format!("{}-session-end.md", date);

    // Check for duplicate filenames
    let mut final_filename = filename.clone();
    let mut counter = 1;
    while handoffs_dir.join(&final_filename).exists() {
        final_filename = format!("{}-session-end-{}.md", date, counter);
        counter += 1;
    }

    // Gather plugin context via hook system
    let mut plugin_context = String::new();
    let hook_outputs = plugin::run_hooks(project_root, "on_handoff_generate");
    for output in &hook_outputs {
        plugin_context.push_str(&format!("### {}\n\n```\n", output.label));
        plugin_context.push_str(&output.content);
        plugin_context.push_str("```\n\n");
    }

    // Generate handoff content
    let content = format!(
        "---\ndate: {date}\nproject: {project}\nphase: {phase}\n---\n\n\
         # Session Handoff\n\n\
         ## What Was Done\n\n\
         <!-- Summary of completed work -->\n\n\
         ## Key Decisions\n\n\
         <!-- Decisions made and rationale -->\n\n\
         ## Gotchas & Surprises\n\n\
         <!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->\n\n\
         ## What Took Longer Than Expected\n\n\
         <!-- Steps that needed multiple attempts. Commands that failed before the right one. -->\n\n\
         ## Open Questions\n\n\
         <!-- Unresolved questions -->\n\n\
         ## Next Steps\n\n\
         <!-- Prioritized list of what to do next -->\n\n\
         ## Context\n\n\
         {plugin_context}",
        date = date,
        project = project,
        phase = state.current,
        plugin_context = if plugin_context.is_empty() {
            "<!-- No plugin context available -->\n".to_string()
        } else {
            plugin_context
        },
    );

    let path = handoffs_dir.join(&final_filename);
    std::fs::write(&path, &content).into_diagnostic()?;
    Ok(path)
}
