use chrono::Utc;
use cliclack::log;
use miette::{IntoDiagnostic, Result};
use std::io::IsTerminal;

use crate::cli::AddCommands;
use crate::config::{DESIGNS_DIR, PLANS_DIR, RESEARCH_DIR, areas_dir, archives_dir, projects_dir, resources_dir};
use crate::context::require_safe_mode;
use crate::error::WaiError;
use crate::json::Suggestion;
use crate::state::Phase;
use crate::workflows;

use super::{print_suggestions, require_project};

pub fn run(cmd: AddCommands) -> Result<()> {
    let project_root = require_project()?;

    match cmd {
        AddCommands::Research {
            content,
            file,
            project,
            tags,
        } => {
            require_safe_mode("add research")?;
            let (target_project, project_dir) = resolve_project(&project_root, project.as_deref())?;
            let dir = project_dir.join(RESEARCH_DIR);

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

            // Post-command suggestions after adding research
            if let Some(ctx) = workflows::scan_project(&project_root, &target_project) {
                let suggestions = match ctx.phase {
                    Phase::Research if ctx.research_count >= 2 => vec![
                        Suggestion {
                            label: "Add more research".to_string(),
                            command: "wai add research \"...\"".to_string(),
                        },
                        Suggestion {
                            label: "Move to design phase".to_string(),
                            command: "wai phase set design".to_string(),
                        },
                        Suggestion {
                            label: "Review research".to_string(),
                            command: "wai search \"research\"".to_string(),
                        },
                    ],
                    Phase::Research => vec![
                        Suggestion {
                            label: "Add more research".to_string(),
                            command: "wai add research \"...\"".to_string(),
                        },
                        Suggestion {
                            label: "Check phase".to_string(),
                            command: "wai phase".to_string(),
                        },
                    ],
                    _ => vec![
                        Suggestion {
                            label: "Continue research".to_string(),
                            command: "wai add research \"...\"".to_string(),
                        },
                        Suggestion {
                            label: "Review research".to_string(),
                            command: "wai search \"research\"".to_string(),
                        },
                    ],
                };
                print_suggestions(&suggestions);
            }

            Ok(())
        }
        AddCommands::Plan {
            content,
            file,
            project,
        } => {
            require_safe_mode("add plan")?;
            let (target_project, project_dir) = resolve_project(&project_root, project.as_deref())?;
            let dir = project_dir.join(PLANS_DIR);

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
            require_safe_mode("add design")?;
            let (target_project, project_dir) = resolve_project(&project_root, project.as_deref())?;
            let dir = project_dir.join(DESIGNS_DIR);

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
) -> Result<(String, std::path::PathBuf)> {
    // All PARA category directories to search
    let search_dirs = [
        projects_dir(project_root),
        areas_dir(project_root),
        resources_dir(project_root),
        archives_dir(project_root),
    ];

    if let Some(name) = explicit {
        for base in &search_dirs {
            let dir = base.join(name);
            if dir.exists() {
                return Ok((name.to_string(), dir));
            }
        }
        return Err(WaiError::ProjectNotFound {
            name: name.to_string(),
        }
        .into());
    }

    // Collect all project directories across PARA categories
    let mut projects: Vec<(String, std::path::PathBuf)> = Vec::new();
    for base in &search_dirs {
        if !base.exists() {
            continue;
        }
        for entry in std::fs::read_dir(base).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            if entry.file_type().into_diagnostic()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    projects.push((name.to_string(), entry.path()));
                }
            }
        }
    }
    projects.sort_by(|a, b| a.0.cmp(&b.0));

    match projects.len() {
        0 => Err(WaiError::NoProjectContext.into()),
        1 => Ok(projects.remove(0)),
        _ => {
            let ctx = crate::context::current_context();
            if ctx.no_input || !std::io::stdin().is_terminal() {
                let names: Vec<_> = projects.iter().map(|(n, _)| n.as_str()).collect();
                return Err(WaiError::NonInteractive {
                    message: format!(
                        "Multiple projects found ({}). Use --project <name> to specify one.",
                        names.join(", ")
                    ),
                }
                .into());
            }
            // Interactive selection via cliclack
            let mut sel = cliclack::select("Multiple projects found — which one?");
            for (name, path) in &projects {
                sel = sel.item((name.clone(), path.clone()), name.as_str(), "");
            }
            let selected: (String, std::path::PathBuf) = sel.interact().into_diagnostic()?;
            Ok(selected)
        }
    }
}

fn get_content(content: Option<&str>, file: Option<&str>) -> Result<String> {
    if let Some(path) = file {
        return std::fs::read_to_string(path).into_diagnostic();
    }
    if let Some(text) = content {
        return Ok(text.to_string());
    }
    Err(miette::miette!(
        "Provide content or use --file to import from a file"
    ))
}
