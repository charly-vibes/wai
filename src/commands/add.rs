use chrono::Utc;
use cliclack::log;
use miette::{IntoDiagnostic, Result};

use crate::cli::AddCommands;
use crate::config::{projects_dir, RESEARCH_DIR, PLANS_DIR, DESIGNS_DIR};
use crate::error::WaiError;

use super::require_project;

pub fn run(cmd: AddCommands) -> Result<()> {
    let project_root = require_project()?;

    match cmd {
        AddCommands::Research {
            content,
            file,
            project,
            tags,
        } => {
            let target_project = resolve_project(&project_root, project.as_deref())?;
            let dir = projects_dir(&project_root)
                .join(&target_project)
                .join(RESEARCH_DIR);

            let body = get_content(content.as_deref(), file.as_deref())?;
            let slug = slug::slugify(body.chars().take(50).collect::<String>());
            let date = Utc::now().format("%Y-%m-%d");
            let filename = format!("{}-{}.md", date, slug);

            let mut file_content = String::new();

            // Add frontmatter if tags provided
            if let Some(tags) = tags {
                file_content.push_str("---\n");
                file_content.push_str(&format!(
                    "tags: [{}]\n",
                    tags.split(',')
                        .map(|t| t.trim().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                file_content.push_str("---\n\n");
            }

            file_content.push_str(&body);
            file_content.push('\n');

            std::fs::write(dir.join(&filename), &file_content).into_diagnostic()?;
            log::success(format!("Added research to '{}'", target_project)).into_diagnostic()?;
            Ok(())
        }
        AddCommands::Plan {
            content,
            file,
            project,
        } => {
            let target_project = resolve_project(&project_root, project.as_deref())?;
            let dir = projects_dir(&project_root)
                .join(&target_project)
                .join(PLANS_DIR);

            let body = get_content(content.as_deref(), file.as_deref())?;
            let slug = slug::slugify(body.chars().take(50).collect::<String>());
            let date = Utc::now().format("%Y-%m-%d");
            let filename = format!("{}-{}.md", date, slug);

            let mut file_content = String::new();
            file_content.push_str(&body);
            file_content.push('\n');

            std::fs::write(dir.join(&filename), &file_content).into_diagnostic()?;
            log::success(format!("Added plan to '{}'", target_project)).into_diagnostic()?;
            Ok(())
        }
        AddCommands::Design {
            content,
            file,
            project,
        } => {
            let target_project = resolve_project(&project_root, project.as_deref())?;
            let dir = projects_dir(&project_root)
                .join(&target_project)
                .join(DESIGNS_DIR);

            let body = get_content(content.as_deref(), file.as_deref())?;
            let slug = slug::slugify(body.chars().take(50).collect::<String>());
            let date = Utc::now().format("%Y-%m-%d");
            let filename = format!("{}-{}.md", date, slug);

            let mut file_content = String::new();
            file_content.push_str(&body);
            file_content.push('\n');

            std::fs::write(dir.join(&filename), &file_content).into_diagnostic()?;
            log::success(format!("Added design to '{}'", target_project)).into_diagnostic()?;
            Ok(())
        }
    }
}

fn resolve_project(
    project_root: &std::path::Path,
    explicit: Option<&str>,
) -> Result<String> {
    if let Some(name) = explicit {
        let dir = projects_dir(project_root).join(name);
        if !dir.exists() {
            return Err(WaiError::ProjectNotFound {
                name: name.to_string(),
            }
            .into());
        }
        return Ok(name.to_string());
    }

    // Find the first project (or error if none)
    let proj_dir = projects_dir(project_root);
    if proj_dir.exists() {
        for entry in std::fs::read_dir(&proj_dir).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            if entry.file_type().into_diagnostic()?.is_dir()
                && let Some(name) = entry.file_name().to_str() {
                    return Ok(name.to_string());
                }
        }
    }

    Err(WaiError::NoProjectContext.into())
}

fn get_content(content: Option<&str>, file: Option<&str>) -> Result<String> {
    if let Some(path) = file {
        return std::fs::read_to_string(path).into_diagnostic();
    }
    if let Some(text) = content {
        return Ok(text.to_string());
    }
    Err(miette::miette!("Provide content or use --file to import from a file"))
}
