//! Handlers for the research/plan/design variants of `wai add`.

use cliclack::log;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::config::{DESIGNS_DIR, PLANS_DIR, projects_dir};
use crate::context::{current_context, require_safe_mode};

use super::super::resolve_project;
use super::helpers;
use super::helpers::process_corrects;
use super::tags::build_tags;

/// Handle `wai add plan` / `wai add design`; `design` picks the artifact dir.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle(
    project_root: &Path,
    content: Option<String>,
    file: Option<String>,
    project: Option<String>,
    tags: Option<String>,
    corrects: Option<String>,
    tracks: Option<String>,
    design: bool,
) -> Result<()> {
    let (safe_mode_label, artifact_dir, label) = if design {
        ("add design", DESIGNS_DIR, "design")
    } else {
        ("add plan", PLANS_DIR, "plan")
    };
    add_simple_artifact(
        project_root,
        safe_mode_label,
        artifact_dir,
        label,
        content,
        file,
        project,
        tags,
        corrects,
        tracks,
    )
}

/// Shared flow for plan and design: identical shape, differing directory and label.
#[allow(clippy::too_many_arguments)]
fn add_simple_artifact(
    project_root: &Path,
    safe_mode_label: &str,
    artifact_dir: &str,
    label: &str,
    content: Option<String>,
    file: Option<String>,
    project: Option<String>,
    tags: Option<String>,
    corrects: Option<String>,
    tracks: Option<String>,
) -> Result<()> {
    require_safe_mode(safe_mode_label)?;
    let resolved = resolve_project(project_root, project.as_deref())?;
    let target_project = resolved.name;
    let dir = projects_dir(project_root)
        .join(&target_project)
        .join(artifact_dir);

    let body = helpers::get_content(content.as_deref(), file.as_deref())?;
    let filename = helpers::make_unique_filename(&dir, &body);

    let mut all_tags = build_tags(tags.as_deref(), project_root);
    process_corrects(corrects.as_deref(), &mut all_tags)?;
    let tracks_list = helpers::parse_tracks_arg(tracks.as_deref());

    let file_content =
        helpers::assemble_artifact(&body, &all_tags, None, corrects.as_ref(), &tracks_list);

    helpers::write_artifact(&dir, &filename, &file_content, project_root, &tracks_list)?;
    if !current_context().quiet {
        log::success(format!("Added {} to '{}'", label, target_project)).into_diagnostic()?;
    }
    Ok(())
}
