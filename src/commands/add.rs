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
            corrects,
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
            let mut all_tags = build_tags(tags.as_deref(), &project_root);
            // When correcting another artifact, auto-add pipeline-addendum tag
            if let Some(ref corrects_path) = corrects
                && let Some(step_id) = resolve_pipeline_step_from_artifact(corrects_path)
            {
                warn_if_unlocked(corrects_path, &step_id)?;
                let addendum_tag = format!("pipeline-addendum:{}", step_id);
                if !all_tags.contains(&addendum_tag) {
                    all_tags.push(addendum_tag);
                }
            }
            if !all_tags.is_empty() || bead.is_some() || corrects.is_some() {
                file_content.push_str("---\n");
                if !all_tags.is_empty() {
                    file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
                }
                if let Some(ref bead_id) = bead {
                    file_content.push_str(&format!("bead: {}\n", bead_id));
                }
                if let Some(ref corrects_path) = corrects {
                    file_content.push_str(&format!("corrects: {}\n", corrects_path));
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
            corrects,
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

            let mut all_tags = build_tags(tags.as_deref(), &project_root);
            if let Some(ref corrects_path) = corrects
                && let Some(step_id) = resolve_pipeline_step_from_artifact(corrects_path)
            {
                warn_if_unlocked(corrects_path, &step_id)?;
                let addendum_tag = format!("pipeline-addendum:{}", step_id);
                if !all_tags.contains(&addendum_tag) {
                    all_tags.push(addendum_tag);
                }
            }
            if !all_tags.is_empty() || corrects.is_some() {
                file_content.push_str("---\n");
                if !all_tags.is_empty() {
                    file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
                }
                if let Some(ref corrects_path) = corrects {
                    file_content.push_str(&format!("corrects: {}\n", corrects_path));
                }
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
            corrects,
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

            let mut all_tags = build_tags(tags.as_deref(), &project_root);
            if let Some(ref corrects_path) = corrects
                && let Some(step_id) = resolve_pipeline_step_from_artifact(corrects_path)
            {
                warn_if_unlocked(corrects_path, &step_id)?;
                let addendum_tag = format!("pipeline-addendum:{}", step_id);
                if !all_tags.contains(&addendum_tag) {
                    all_tags.push(addendum_tag);
                }
            }
            if !all_tags.is_empty() || corrects.is_some() {
                file_content.push_str("---\n");
                if !all_tags.is_empty() {
                    file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
                }
                if let Some(ref corrects_path) = corrects {
                    file_content.push_str(&format!("corrects: {}\n", corrects_path));
                }
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
            corrects,
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
            let mut all_tags = build_tags(tags.as_deref(), &project_root);
            if let Some(ref corrects_path) = corrects
                && let Some(step_id) = resolve_pipeline_step_from_artifact(corrects_path)
            {
                warn_if_unlocked(corrects_path, &step_id)?;
                let addendum_tag = format!("pipeline-addendum:{}", step_id);
                if !all_tags.contains(&addendum_tag) {
                    all_tags.push(addendum_tag);
                }
            }

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
            if let Some(ref corrects_path) = corrects {
                file_content.push_str(&format!("corrects: {}\n", corrects_path));
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

    #[test]
    fn corrects_field_written_to_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let research_dir = dir.path().join("research");
        std::fs::create_dir_all(&research_dir).unwrap();

        let corrects_path = "research/2026-04-01-original.md";
        let body = "corrected notes";
        let slug = slug::slugify(body.chars().take(50).collect::<String>());
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let filename = format!("{}-{}.md", date, slug);

        let mut file_content = String::new();
        let all_tags: Vec<String> = vec![];
        let corrects: Option<String> = Some(corrects_path.to_string());
        // Replicate the frontmatter logic for corrects
        if !all_tags.is_empty() || corrects.is_some() {
            file_content.push_str("---\n");
            if !all_tags.is_empty() {
                file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
            }
            if let Some(ref cp) = corrects {
                file_content.push_str(&format!("corrects: {}\n", cp));
            }
            file_content.push_str("---\n\n");
        }
        file_content.push_str(body);
        file_content.push('\n');

        std::fs::write(research_dir.join(&filename), &file_content).unwrap();
        let written = std::fs::read_to_string(research_dir.join(&filename)).unwrap();
        assert!(
            written.contains("corrects: research/2026-04-01-original.md"),
            "corrects field missing from frontmatter"
        );
        assert!(written.contains("---"), "frontmatter delimiters missing");
    }

    #[test]
    fn resolve_pipeline_step_from_artifact_parses_tag() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("original.md");
        let content =
            "---\ntags: [pipeline-run:test-run, pipeline-step:research]\n---\n\noriginal content\n";
        std::fs::write(&artifact, content).unwrap();

        let result = super::resolve_pipeline_step_from_artifact(artifact.to_str().unwrap());
        assert_eq!(result, Some("research".to_string()));
    }

    #[test]
    fn resolve_pipeline_step_returns_none_without_step_tag() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("no-step.md");
        let content = "---\ntags: [some-tag]\n---\n\ncontent\n";
        std::fs::write(&artifact, content).unwrap();

        let result = super::resolve_pipeline_step_from_artifact(artifact.to_str().unwrap());
        assert_eq!(result, None);
    }

    #[test]
    fn resolve_pipeline_step_returns_none_without_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("no-fm.md");
        std::fs::write(&artifact, "just plain content\n").unwrap();

        let result = super::resolve_pipeline_step_from_artifact(artifact.to_str().unwrap());
        assert_eq!(result, None);
    }

    #[test]
    fn resolve_pipeline_step_returns_none_for_missing_file() {
        let result = super::resolve_pipeline_step_from_artifact("/nonexistent/path.md");
        assert_eq!(result, None);
    }

    #[test]
    fn corrects_auto_adds_pipeline_addendum_tag() {
        let dir = tempfile::tempdir().unwrap();
        // Create the corrected artifact with a pipeline-step tag
        let original = dir.path().join("original.md");
        let original_content =
            "---\ntags: [pipeline-run:run1, pipeline-step:implement]\n---\n\noriginal\n";
        std::fs::write(&original, original_content).unwrap();

        let corrects_path = original.to_str().unwrap();
        let mut all_tags: Vec<String> = vec!["custom".to_string()];
        // Replicate the corrects logic from the command handler
        if let Some(step_id) = super::resolve_pipeline_step_from_artifact(corrects_path) {
            let addendum_tag = format!("pipeline-addendum:{}", step_id);
            if !all_tags.contains(&addendum_tag) {
                all_tags.push(addendum_tag);
            }
        }

        assert!(
            all_tags.contains(&"pipeline-addendum:implement".to_string()),
            "expected pipeline-addendum:implement tag, got: {:?}",
            all_tags
        );
        assert!(
            all_tags.contains(&"custom".to_string()),
            "user tag should be preserved"
        );
    }

    #[test]
    fn has_lock_file_returns_true_when_lock_exists() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("2026-04-01-research.md");
        std::fs::write(&artifact, "content").unwrap();
        // Create a lock sidecar
        let lock_file = dir.path().join("2026-04-01-research.md.run-abc.lock");
        std::fs::write(&lock_file, "").unwrap();

        assert!(
            super::has_lock_file(artifact.to_str().unwrap()),
            "should detect lock file"
        );
    }

    #[test]
    fn has_lock_file_returns_false_when_no_lock() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("2026-04-01-research.md");
        std::fs::write(&artifact, "content").unwrap();

        assert!(
            !super::has_lock_file(artifact.to_str().unwrap()),
            "should not detect lock file when none exists"
        );
    }

    #[test]
    fn has_lock_file_returns_false_for_nonexistent_dir() {
        assert!(
            !super::has_lock_file("/nonexistent/dir/artifact.md"),
            "should return false for nonexistent directory"
        );
    }

    #[test]
    fn has_lock_file_ignores_non_lock_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("2026-04-01-research.md");
        std::fs::write(&artifact, "content").unwrap();
        // Create a non-lock sidecar (e.g. .bak)
        let bak_file = dir.path().join("2026-04-01-research.md.run-abc.bak");
        std::fs::write(&bak_file, "").unwrap();

        assert!(
            !super::has_lock_file(artifact.to_str().unwrap()),
            "should not match non-.lock sidecar files"
        );
    }

    #[test]
    fn warn_if_unlocked_does_not_error_when_quiet() {
        // When the artifact has no lock file the function should still succeed
        // (it only prints a warning). We set quiet mode to avoid cliclack output.
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("artifact.md");
        std::fs::write(&artifact, "content").unwrap();

        // Set quiet context so the cliclack warning call is skipped
        crate::context::set_context(crate::context::CliContext {
            json: false,
            no_input: false,
            yes: false,
            safe: false,
            verbose: 0,
            quiet: true,
        });
        let result = super::warn_if_unlocked(artifact.to_str().unwrap(), "research");
        assert!(result.is_ok(), "warn_if_unlocked should not error");
    }

    #[test]
    fn warn_if_unlocked_skips_when_locked() {
        // When a lock file exists, warn_if_unlocked should not warn at all
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("artifact.md");
        std::fs::write(&artifact, "content").unwrap();
        let lock = dir.path().join("artifact.md.run-xyz.lock");
        std::fs::write(&lock, "").unwrap();

        // Even without quiet mode, this should succeed because the lock exists
        // (no warning emitted)
        let result = super::warn_if_unlocked(artifact.to_str().unwrap(), "research");
        assert!(
            result.is_ok(),
            "warn_if_unlocked should not error when locked"
        );
    }
}

/// Check if an artifact has any associated .lock sidecar files.
///
/// Lock files follow the naming convention `<artifact-filename>.<run-id>.lock`
/// in the same directory as the artifact. Returns `true` if at least one such
/// file exists.
fn has_lock_file(artifact_path: &str) -> bool {
    let path = std::path::Path::new(artifact_path);
    let dir = match path.parent() {
        Some(d) => d,
        None => return false,
    };
    let filename = match path.file_name() {
        Some(f) => f.to_string_lossy(),
        None => return false,
    };
    let prefix = format!("{}.", filename);
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) && name.ends_with(".lock") {
                return true;
            }
        }
    }
    false
}

/// Warn when `--corrects` targets an artifact whose pipeline step is not locked.
///
/// This is informational only — the addendum is still created regardless.
fn warn_if_unlocked(corrects_path: &str, step_id: &str) -> Result<()> {
    if !has_lock_file(corrects_path) && !current_context().quiet {
        cliclack::log::warning(format!(
            "Step '{}' is not locked — consider editing the original artifact directly.",
            step_id
        ))
        .into_diagnostic()?;
    }
    Ok(())
}

/// Extract `pipeline-step:<id>` tag from an artifact's YAML frontmatter.
///
/// Returns `None` gracefully if the file cannot be read, has no frontmatter,
/// or contains no `pipeline-step:` tag.
fn resolve_pipeline_step_from_artifact(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    if !content.starts_with("---\n") {
        return None;
    }
    let end = content[4..].find("---\n")?;
    let fm = &content[4..4 + end];
    for line in fm.lines() {
        if line.starts_with("tags:") {
            for tag in line
                .trim_start_matches("tags:")
                .trim()
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
            {
                let tag = tag.trim();
                if let Some(step) = tag.strip_prefix("pipeline-step:") {
                    return Some(step.to_string());
                }
            }
        }
    }
    None
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
