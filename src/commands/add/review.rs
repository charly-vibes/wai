//! Review-specific parsing, validation, and frontmatter for `wai add review`.

use cliclack::log;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::config::{REVIEWS_DIR, projects_dir};
use crate::context::current_context;

use super::helpers;
use super::super::resolve_project;
use super::helpers::process_corrects;
use super::tags::build_tags;

/// Handle `wai add review`.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle(
    project_root: &Path,
    content: Option<String>,
    file: Option<String>,
    project: Option<String>,
    tags: Option<String>,
    reviews: String,
    verdict: Option<String>,
    severity: Option<String>,
    produced_by: Option<String>,
    corrects: Option<String>,
) -> Result<()> {
    crate::context::require_safe_mode("add review")?;
    let resolved = resolve_project(project_root, project.as_deref())?;
    let target_project = resolved.name;
    let project_dir = projects_dir(project_root).join(&target_project);

    // Validate the target artifact exists in the current project
    validate_review_target(&project_dir, &reviews)?;
    validate_verdict(verdict.as_deref())?;
    let severity_map = severity.as_deref().map(parse_severity).transpose()?;

    let dir = project_dir.join(REVIEWS_DIR);
    let body = helpers::get_content(content.as_deref(), file.as_deref())?;
    let filename = helpers::make_unique_filename(&dir, &body);

    let mut all_tags = build_tags(tags.as_deref(), project_root);
    process_corrects(corrects.as_deref(), &mut all_tags)?;

    let file_content = assemble_review_artifact(
        &body,
        &all_tags,
        &reviews,
        verdict.as_deref(),
        severity_map.as_ref(),
        produced_by.as_deref(),
        corrects.as_ref(),
    );

    std::fs::create_dir_all(&dir).into_diagnostic()?;
    std::fs::write(dir.join(&filename), &file_content).into_diagnostic()?;
    if !current_context().quiet {
        log::success(format!("Added review to '{}'", target_project)).into_diagnostic()?;
    }
    Ok(())
}

/// Parsed severity counts for review frontmatter.
#[derive(Debug)]
pub(super) struct SeverityCounts {
    critical: u32,
    high: u32,
    medium: u32,
    low: u32,
}

/// Parse severity string like "critical:0,high:1,medium:3,low:2".
/// Omitted levels default to 0.
pub(super) fn parse_severity(input: &str) -> Result<SeverityCounts> {
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
pub(super) fn validate_review_target(project_dir: &Path, target: &str) -> Result<()> {
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

/// Reject verdicts outside the accepted set.
pub(super) fn validate_verdict(verdict: Option<&str>) -> Result<()> {
    if let Some(v) = verdict {
        match v {
            "pass" | "fail" | "needs-work" => {}
            _ => {
                miette::bail!(
                    "Invalid verdict '{}'. Valid values: pass, fail, needs-work",
                    v
                );
            }
        }
    }
    Ok(())
}

/// Assemble review file content. Reviews always carry frontmatter (at minimum
/// the `reviews` field).
pub(super) fn assemble_review_artifact(
    body: &str,
    all_tags: &[String],
    reviews: &str,
    verdict: Option<&str>,
    severity: Option<&SeverityCounts>,
    produced_by: Option<&str>,
    corrects: Option<&String>,
) -> String {
    let mut file_content = String::from("---\n");
    file_content.push_str(&format!("reviews: {}\n", reviews));
    if let Some(v) = verdict {
        file_content.push_str(&format!("verdict: {}\n", v));
    }
    if let Some(sev) = severity {
        file_content.push_str(&format!(
            "severity: {{critical: {}, high: {}, medium: {}, low: {}}}\n",
            sev.critical, sev.high, sev.medium, sev.low
        ));
    }
    if let Some(pb) = produced_by {
        file_content.push_str(&format!("produced_by: {}\n", pb));
    }
    if let Some(corrects_path) = corrects {
        file_content.push_str(&format!("corrects: {}\n", corrects_path));
    }
    if !all_tags.is_empty() {
        file_content.push_str(&format!("tags: [{}]\n", all_tags.join(", ")));
    }
    file_content.push_str("---\n\n");
    file_content.push_str(body);
    file_content.push('\n');
    file_content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_severity_valid() {
        let sev = parse_severity("critical:0,high:1,medium:3,low:2").unwrap();
        assert_eq!(sev.critical, 0);
        assert_eq!(sev.high, 1);
        assert_eq!(sev.medium, 3);
        assert_eq!(sev.low, 2);
    }

    #[test]
    fn parse_severity_partial() {
        let sev = parse_severity("critical:2").unwrap();
        assert_eq!(sev.critical, 2);
        assert_eq!(sev.high, 0);
        assert_eq!(sev.medium, 0);
        assert_eq!(sev.low, 0);
    }

    #[test]
    fn parse_severity_rejects_bad_level() {
        let result = parse_severity("urgent:1");
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

        assert!(validate_review_target(project, "2026-04-02-findings.md").is_ok());
    }

    #[test]
    fn validate_review_target_rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let result = validate_review_target(dir.path(), "../../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a path"));
    }

    #[test]
    fn validate_review_target_rejects_missing() {
        let dir = tempfile::tempdir().unwrap();
        let result = validate_review_target(dir.path(), "nonexistent.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn validate_verdict_accepts_known_values() {
        assert!(validate_verdict(Some("pass")).is_ok());
        assert!(validate_verdict(Some("fail")).is_ok());
        assert!(validate_verdict(Some("needs-work")).is_ok());
        assert!(validate_verdict(None).is_ok());
    }

    #[test]
    fn validate_verdict_rejects_unknown() {
        let result = validate_verdict(Some("meh"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid verdict"));
    }

    #[test]
    fn review_frontmatter_always_includes_reviews_field() {
        let content = assemble_review_artifact("body", &[], "plan.md", None, None, None, None);
        assert!(content.starts_with("---\nreviews: plan.md\n"));
        assert!(content.contains("---\n\nbody\n"));
    }
}
