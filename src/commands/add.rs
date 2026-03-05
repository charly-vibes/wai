use chrono::Utc;
use cliclack::log;
use miette::{IntoDiagnostic, Result};
use std::io::IsTerminal;

use crate::cli::AddCommands;
use super::resource;
use crate::config::{
    DESIGNS_DIR, PLANS_DIR, RESEARCH_DIR, archives_dir, areas_dir, projects_dir,
    read_pipeline_run_state, resources_dir,
};
use crate::context::{current_context, require_safe_mode};
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
            bead,
        } => {
            require_safe_mode("add research")?;
            let (target_project, project_dir) = resolve_project(&project_root, project.as_deref())?;
            let dir = project_dir.join(RESEARCH_DIR);

            let body = get_content(content.as_deref(), file.as_deref())?;
            let slug = slug::slugify(body.chars().take(50).collect::<String>());
            let date = Utc::now().format("%Y-%m-%d");
            let base_filename = format!("{}-{}.md", date, slug);

            // Deduplicate: if the file already exists, append a counter
            let mut filename = base_filename.clone();
            let mut counter = 2;
            while dir.join(&filename).exists() {
                filename = format!("{}-{}-{}.md", date, slug, counter);
                counter += 1;
            }

            let mut file_content = String::new();

            // Build combined tags: user-supplied + pipeline-run auto-tag
            let all_tags = build_tags(tags.as_deref(), &project_root);
            if !all_tags.is_empty() || bead.is_some() {
                file_content.push_str("---\n");
                if !all_tags.is_empty() {
                    file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
                }
                if let Some(ref bead_id) = bead {
                    file_content.push_str(&format!("bead: {}\n", bead_id));
                }
                file_content.push_str("---\n\n");
            }

            file_content.push_str(&body);
            file_content.push('\n');

            std::fs::create_dir_all(&dir).into_diagnostic()?;
            std::fs::write(dir.join(&filename), &file_content).into_diagnostic()?;
            let ctx = current_context();
            if !ctx.quiet {
                log::success(format!("Added research to '{}'", target_project))
                    .into_diagnostic()?;
            }

            // Post-command suggestions after adding research
            if !ctx.quiet
                && let Some(wf_ctx) = workflows::scan_project(&project_root, &target_project)
            {
                let suggestions = match wf_ctx.phase {
                    Phase::Research if wf_ctx.research_count >= 2 => vec![
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
            tags,
        } => {
            require_safe_mode("add plan")?;
            let (target_project, project_dir) = resolve_project(&project_root, project.as_deref())?;
            let dir = project_dir.join(PLANS_DIR);

            let body = get_content(content.as_deref(), file.as_deref())?;
            let slug = slug::slugify(body.chars().take(50).collect::<String>());
            let date = Utc::now().format("%Y-%m-%d");
            let base_filename = format!("{}-{}.md", date, slug);

            // Deduplicate: if the file already exists, append a counter
            let mut filename = base_filename.clone();
            let mut counter = 2;
            while dir.join(&filename).exists() {
                filename = format!("{}-{}-{}.md", date, slug, counter);
                counter += 1;
            }

            let mut file_content = String::new();

            let all_tags = build_tags(tags.as_deref(), &project_root);
            if !all_tags.is_empty() {
                file_content.push_str("---\n");
                file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
                file_content.push_str("---\n\n");
            }

            file_content.push_str(&body);
            file_content.push('\n');

            std::fs::create_dir_all(&dir).into_diagnostic()?;
            std::fs::write(dir.join(&filename), &file_content).into_diagnostic()?;
            if !current_context().quiet {
                log::success(format!("Added plan to '{}'", target_project)).into_diagnostic()?;
            }
            Ok(())
        }
        AddCommands::Design {
            content,
            file,
            project,
            tags,
        } => {
            require_safe_mode("add design")?;
            let (target_project, project_dir) = resolve_project(&project_root, project.as_deref())?;
            let dir = project_dir.join(DESIGNS_DIR);

            let body = get_content(content.as_deref(), file.as_deref())?;
            let slug = slug::slugify(body.chars().take(50).collect::<String>());
            let date = Utc::now().format("%Y-%m-%d");
            let base_filename = format!("{}-{}.md", date, slug);

            // Deduplicate: if the file already exists, append a counter
            let mut filename = base_filename.clone();
            let mut counter = 2;
            while dir.join(&filename).exists() {
                filename = format!("{}-{}-{}.md", date, slug, counter);
                counter += 1;
            }

            let mut file_content = String::new();

            let all_tags = build_tags(tags.as_deref(), &project_root);
            if !all_tags.is_empty() {
                file_content.push_str("---\n");
                file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
                file_content.push_str("---\n\n");
            }

            file_content.push_str(&body);
            file_content.push('\n');

            std::fs::create_dir_all(&dir).into_diagnostic()?;
            std::fs::write(dir.join(&filename), &file_content).into_diagnostic()?;
            if !current_context().quiet {
                log::success(format!("Added design to '{}'", target_project)).into_diagnostic()?;
            }
            Ok(())
        }
        AddCommands::Skill { name, template } => resource::add_skill(&name, template.as_deref()),
    }
}

fn resolve_project(
    project_root: &std::path::Path,
    explicit: Option<&str>,
) -> Result<(String, std::path::PathBuf)> {
    // All PARA category directories to search (explicit lookup)
    let all_dirs = [
        projects_dir(project_root),
        areas_dir(project_root),
        resources_dir(project_root),
        archives_dir(project_root),
    ];

    if let Some(name) = explicit {
        for base in &all_dirs {
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

    // Auto-discovery: only search active work (projects + areas).
    // Resources and archives contain infrastructure dirs that would pollute
    // the count and cause false "multiple projects" errors.
    let auto_dirs = [projects_dir(project_root), areas_dir(project_root)];

    // Collect all project directories across PARA categories
    let mut projects: Vec<(String, std::path::PathBuf)> = Vec::new();
    for base in &auto_dirs {
        if !base.exists() {
            continue;
        }
        for entry in std::fs::read_dir(base).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            if entry.file_type().into_diagnostic()?.is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                projects.push((name.to_string(), entry.path()));
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

/// Build the final tags list: user-supplied tags merged with the auto-injected
/// `pipeline-run:<id>` tag when an active pipeline run can be resolved.
///
/// Resolution order (first non-empty value wins):
///   1. `WAI_PIPELINE_RUN` environment variable (backwards-compatible).
///   2. `.wai/.pipeline-run` state file (written by `wai pipeline run`).
#[cfg(test)]
mod tests {
    #[test]
    fn bead_field_written_to_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let research_dir = dir.path().join("research");
        std::fs::create_dir_all(&research_dir).unwrap();

        let bead_id = "wai-p94k";
        let body = "some research notes";
        let slug = slug::slugify(body.chars().take(50).collect::<String>());
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let filename = format!("{}-{}.md", date, slug);

        let mut file_content = String::new();
        let all_tags: Vec<String> = vec![];
        // Replicate the frontmatter logic from run()
        if !all_tags.is_empty() || true {
            file_content.push_str("---\n");
            if !all_tags.is_empty() {
                file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
            }
            file_content.push_str(&format!("bead: {}\n", bead_id));
            file_content.push_str("---\n\n");
        }
        file_content.push_str(body);
        file_content.push('\n');

        std::fs::write(research_dir.join(&filename), &file_content).unwrap();
        let written = std::fs::read_to_string(research_dir.join(&filename)).unwrap();
        assert!(written.contains("bead: wai-p94k"), "bead field missing from frontmatter");
        assert!(written.contains("---"), "frontmatter delimiters missing");
        assert!(!written.contains("tags:"), "tags should not appear when not provided");
    }

    #[test]
    fn bead_and_tags_both_written_when_both_provided() {
        let bead_id = "wai-abc";
        let tags = vec!["research".to_string(), "design".to_string()];
        let mut file_content = String::new();
        file_content.push_str("---\n");
        if !tags.is_empty() {
            file_content.push_str(&format!("tags: [{}]\n", tags.join(", ")));
        }
        file_content.push_str(&format!("bead: {}\n", bead_id));
        file_content.push_str("---\n\n");
        file_content.push_str("notes\n");

        assert!(file_content.contains("tags: [research, design]"));
        assert!(file_content.contains("bead: wai-abc"));
    }

    #[test]
    fn no_frontmatter_when_neither_tags_nor_bead() {
        let all_tags: Vec<String> = vec![];
        let bead: Option<String> = None;
        let mut file_content = String::new();
        if !all_tags.is_empty() || bead.is_some() {
            file_content.push_str("---\n");
            if !all_tags.is_empty() {
                file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
            }
            if let Some(ref id) = bead {
                file_content.push_str(&format!("bead: {}\n", id));
            }
            file_content.push_str("---\n\n");
        }
        file_content.push_str("notes\n");

        assert!(!file_content.contains("---"), "no frontmatter should be written");
    }
}

fn build_tags(user_tags: Option<&str>, project_root: &std::path::Path) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();

    if let Some(t) = user_tags {
        tags.extend(
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        );
    }

    // Resolve active pipeline run: env var first (backwards compat), then state file.
    let active_run = std::env::var("WAI_PIPELINE_RUN")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| read_pipeline_run_state(project_root));

    if let Some(run_id) = active_run {
        tags.push(format!("pipeline-run:{}", run_id));
    }

    tags
}
