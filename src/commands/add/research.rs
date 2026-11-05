//! Handler for the research variant of `wai add`, including phase-aware suggestions.

use cliclack::log;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::config::{RESEARCH_DIR, projects_dir};
use crate::context::{current_context, require_safe_mode};
use crate::json::Suggestion;
use crate::state::Phase;
use crate::workflows;

use super::helpers;
use super::super::print_suggestions;
use super::super::resolve_project;
use super::helpers::process_corrects;
use super::tags::build_tags;

/// Handle `wai add research`.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle(
    project_root: &Path,
    content: Option<String>,
    file: Option<String>,
    project: Option<String>,
    tags: Option<String>,
    bead: Option<String>,
    corrects: Option<String>,
    tracks: Option<String>,
) -> Result<()> {
    require_safe_mode("add research")?;
    let resolved = resolve_project(project_root, project.as_deref())?;
    let target_project = resolved.name;
    let dir = projects_dir(project_root)
        .join(&target_project)
        .join(RESEARCH_DIR);

    let body = helpers::get_content(content.as_deref(), file.as_deref())?;
    let filename = helpers::make_unique_filename(&dir, &body);

    let mut all_tags = build_tags(tags.as_deref(), project_root);
    process_corrects(corrects.as_deref(), &mut all_tags)?;
    let tracks_list = helpers::parse_tracks_arg(tracks.as_deref());

    let file_content = helpers::assemble_artifact(
        &body,
        &all_tags,
        bead.as_ref(),
        corrects.as_ref(),
        &tracks_list,
    );

    helpers::write_artifact(&dir, &filename, &file_content, project_root, &tracks_list)?;
    if !current_context().quiet {
        log::success(format!("Added research to '{}'", target_project)).into_diagnostic()?;
    }
    research_suggestions(project_root, &target_project);
    Ok(())
}

/// Print phase-aware follow-up suggestions after adding research.
fn research_suggestions(project_root: &Path, target_project: &str) {
    let ctx = current_context();
    if ctx.quiet {
        return;
    }
    let Some(wf_ctx) = workflows::scan_project(project_root, target_project) else {
        return;
    };
    let suggestions = match wf_ctx.phase {
        Phase::Research if wf_ctx.research_count >= 2 => vec![
            suggest("Add more research", "wai add research \"...\""),
            suggest("Move to design phase", "wai phase set design"),
            suggest("Review research", "wai search \"research\""),
        ],
        Phase::Research => vec![
            suggest("Add more research", "wai add research \"...\""),
            suggest("Check phase", "wai phase"),
        ],
        _ => vec![
            suggest("Continue research", "wai add research \"...\""),
            suggest("Review research", "wai search \"research\""),
        ],
    };
    print_suggestions(&suggestions);
}

fn suggest(label: &str, command: &str) -> Suggestion {
    Suggestion {
        label: label.to_string(),
        command: command.to_string(),
    }
}
