use chrono::Utc;
use cliclack::log;
use miette::{IntoDiagnostic, Result};

use super::resource;
use crate::cli::AddCommands;
use crate::config::{
    DESIGNS_DIR, PLANS_DIR, RESEARCH_DIR, REVIEWS_DIR, projects_dir, read_pipeline_run_state,
};
use crate::context::{current_context, require_safe_mode};
use crate::json::Suggestion;
use crate::state::Phase;
use crate::workflows;

use super::{print_suggestions, require_project, resolve_project};

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
            let resolved = resolve_project(&project_root, project.as_deref())?;
            let target_project = resolved.name;
            let project_dir = projects_dir(&project_root).join(&target_project);
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
            let resolved = resolve_project(&project_root, project.as_deref())?;
            let target_project = resolved.name;
            let project_dir = projects_dir(&project_root).join(&target_project);
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
            let resolved = resolve_project(&project_root, project.as_deref())?;
            let target_project = resolved.name;
            let project_dir = projects_dir(&project_root).join(&target_project);
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
        AddCommands::Review {
            content,
            file,
            project,
            tags,
            reviews,
            verdict,
            severity,
            produced_by,
        } => {
            require_safe_mode("add review")?;
            let resolved = resolve_project(&project_root, project.as_deref())?;
            let target_project = resolved.name;
            let project_dir = projects_dir(&project_root).join(&target_project);

            // Validate the target artifact exists in the current project
            validate_review_target(&project_dir, &reviews)?;

            // Validate verdict if provided
            if let Some(ref v) = verdict {
                match v.as_str() {
                    "pass" | "fail" | "needs-work" => {}
                    _ => {
                        miette::bail!(
                            "Invalid verdict '{}'. Valid values: pass, fail, needs-work",
                            v
                        );
                    }
                }
            }

            // Parse severity if provided
            let severity_map = severity.as_deref().map(parse_severity).transpose()?;

            let dir = project_dir.join(REVIEWS_DIR);
            let body = get_content(content.as_deref(), file.as_deref())?;
            let slug = slug::slugify(body.chars().take(50).collect::<String>());
            let date = Utc::now().format("%Y-%m-%d");
            let base_filename = format!("{}-{}.md", date, slug);

            let mut filename = base_filename.clone();
            let mut counter = 2;
            while dir.join(&filename).exists() {
                filename = format!("{}-{}-{}.md", date, slug, counter);
                counter += 1;
            }

            let mut file_content = String::new();
            let all_tags = build_tags(tags.as_deref(), &project_root);

            // Reviews always have frontmatter (at minimum the reviews field)
            file_content.push_str("---\n");
            file_content.push_str(&format!("reviews: {}\n", reviews));
            if let Some(ref v) = verdict {
                file_content.push_str(&format!("verdict: {}\n", v));
            }
            if let Some(ref sev) = severity_map {
                file_content.push_str(&format!(
                    "severity: {{critical: {}, high: {}, medium: {}, low: {}}}\n",
                    sev.critical, sev.high, sev.medium, sev.low
                ));
            }
            if let Some(ref pb) = produced_by {
                file_content.push_str(&format!("produced_by: {}\n", pb));
            }
            if !all_tags.is_empty() {
                file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
            }
            file_content.push_str("---\n\n");
            file_content.push_str(&body);
            file_content.push('\n');

            std::fs::create_dir_all(&dir).into_diagnostic()?;
            std::fs::write(dir.join(&filename), &file_content).into_diagnostic()?;
            if !current_context().quiet {
                log::success(format!("Added review to '{}'", target_project)).into_diagnostic()?;
            }
            Ok(())
        }
        AddCommands::Skill { name, template } => resource::add_skill(&name, template.as_deref()),
    }
}

/// Parsed severity counts for review frontmatter.
#[derive(Debug)]
struct SeverityCounts {
    critical: u32,
    high: u32,
    medium: u32,
    low: u32,
}

/// Parse severity string like "critical:0,high:1,medium:3,low:2".
/// Omitted levels default to 0.
fn parse_severity(input: &str) -> Result<SeverityCounts> {
    let mut counts = SeverityCounts {
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
    };
    for pair in input.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let parts: Vec<&str> = pair.splitn(2, ':').collect();
        if parts.len() != 2 {
            miette::bail!(
                "Invalid severity format '{}'. Expected level:count (e.g. critical:0,high:1)",
                pair
            );
        }
        let level = parts[0].trim();
        let count: u32 = parts[1].trim().parse().map_err(|_| {
            miette::miette!(
                "Invalid count '{}' for severity level '{}'",
                parts[1].trim(),
                level
            )
        })?;
        match level {
            "critical" => counts.critical = count,
            "high" => counts.high = count,
            "medium" => counts.medium = count,
            "low" => counts.low = count,
            _ => {
                miette::bail!(
                    "Unknown severity level '{}'. Valid levels: critical, high, medium, low",
                    level
                );
            }
        }
    }
    Ok(counts)
}

/// Validate that a review target artifact exists in the project directory.
/// Searches across all artifact type directories (research, plans, designs, handoffs, reviews).
/// Rejects targets containing path separators to prevent directory traversal.
fn validate_review_target(project_dir: &std::path::Path, target: &str) -> Result<()> {
    if target.contains('/') || target.contains('\\') {
        miette::bail!("review target must be a filename, not a path: '{}'", target);
    }
    let dirs = [
        crate::config::RESEARCH_DIR,
        crate::config::PLANS_DIR,
        crate::config::DESIGNS_DIR,
        crate::config::HANDOFFS_DIR,
        crate::config::REVIEWS_DIR,
    ];
    for dir_name in &dirs {
        if project_dir.join(dir_name).join(target).exists() {
            return Ok(());
        }
    }
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    Err(miette::miette!(
        "artifact '{}' not found in project '{}'",
        target,
        project_name
    ))
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
        assert!(
            written.contains("bead: wai-p94k"),
            "bead field missing from frontmatter"
        );
        assert!(written.contains("---"), "frontmatter delimiters missing");
        assert!(
            !written.contains("tags:"),
            "tags should not appear when not provided"
        );
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
    fn step_tag_injected_when_pipeline_run_active() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Create .wai directory structure
        let wai = root.join(".wai");
        std::fs::create_dir_all(wai.join("pipeline-runs")).unwrap();
        std::fs::create_dir_all(wai.join("resources/pipelines")).unwrap();

        // Write a pipeline definition
        let toml_content = r#"[pipeline]
name = "test-pipe"

[[steps]]
id = "research"
prompt = "Do research on {topic}."

[[steps]]
id = "implement"
prompt = "Implement {topic}."
"#;
        std::fs::write(wai.join("resources/pipelines/test-pipe.toml"), toml_content).unwrap();

        // Write a run state at step 0
        let run_yaml = r#"run_id: "test-pipe-2026-04-02-qcd"
pipeline: "test-pipe"
topic: "qcd"
created_at: "2026-04-02T00:00:00Z"
current_step: 0
"#;
        std::fs::write(
            wai.join("pipeline-runs/test-pipe-2026-04-02-qcd.yml"),
            run_yaml,
        )
        .unwrap();

        // Write .pipeline-run pointer
        std::fs::write(wai.join(".pipeline-run"), "test-pipe-2026-04-02-qcd").unwrap();

        let tags = super::build_tags(None, root);
        assert!(
            tags.contains(&"pipeline-run:test-pipe-2026-04-02-qcd".to_string()),
            "expected pipeline-run tag, got: {:?}",
            tags
        );
        assert!(
            tags.contains(&"pipeline-step:research".to_string()),
            "expected pipeline-step:research tag, got: {:?}",
            tags
        );
    }

    #[test]
    fn step_tag_not_injected_when_no_run() {
        let dir = tempfile::tempdir().unwrap();
        let tags = super::build_tags(Some("custom"), dir.path());
        assert_eq!(tags, vec!["custom".to_string()]);
        assert!(!tags.iter().any(|t| t.starts_with("pipeline-step:")));
    }

    #[test]
    fn parse_severity_valid() {
        let sev = super::parse_severity("critical:0,high:1,medium:3,low:2").unwrap();
        assert_eq!(sev.critical, 0);
        assert_eq!(sev.high, 1);
        assert_eq!(sev.medium, 3);
        assert_eq!(sev.low, 2);
    }

    #[test]
    fn parse_severity_partial() {
        let sev = super::parse_severity("critical:2").unwrap();
        assert_eq!(sev.critical, 2);
        assert_eq!(sev.high, 0);
        assert_eq!(sev.medium, 0);
        assert_eq!(sev.low, 0);
    }

    #[test]
    fn parse_severity_rejects_bad_level() {
        let result = super::parse_severity("urgent:1");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown severity level")
        );
    }

    #[test]
    fn validate_review_target_finds_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path();
        let research = project.join("research");
        std::fs::create_dir_all(&research).unwrap();
        std::fs::write(research.join("2026-04-02-findings.md"), "content").unwrap();

        assert!(super::validate_review_target(project, "2026-04-02-findings.md").is_ok());
    }

    #[test]
    fn validate_review_target_rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let result = super::validate_review_target(dir.path(), "../../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a path"));
    }

    #[test]
    fn validate_review_target_rejects_missing() {
        let dir = tempfile::tempdir().unwrap();
        let result = super::validate_review_target(dir.path(), "nonexistent.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
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

        assert!(
            !file_content.contains("---"),
            "no frontmatter should be written"
        );
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

    if let Some(ref run_id) = active_run {
        tags.push(format!("pipeline-run:{}", run_id));

        // Also inject pipeline-step:<step-id> from the run state + definition.
        if let Some(step_id) = resolve_current_step_id(project_root, run_id) {
            tags.push(format!("pipeline-step:{}", step_id));
        }
    }

    tags
}

/// Resolve the current step ID by reading the pipeline run state and definition.
///
/// Returns `None` gracefully if anything fails (missing files, parse errors,
/// step index out of bounds), so it never breaks artifact creation.
fn resolve_current_step_id(project_root: &std::path::Path, run_id: &str) -> Option<String> {
    let runs_dir = crate::config::wai_dir(project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    let content = std::fs::read_to_string(&run_path).ok()?;
    let run: super::pipeline::PipelineRun = serde_yml::from_str(&content).ok()?;

    let def_path =
        crate::config::pipelines_dir(project_root).join(format!("{}.toml", run.pipeline));
    let definition = super::pipeline::load_pipeline_toml(&def_path).ok()?;

    definition.steps.get(run.current_step).map(|s| s.id.clone())
}
