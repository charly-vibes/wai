//! Repo-convention checks for `wai way`.

use genesis::doctor::CheckStatus;
use std::path::Path;

use super::core::{detect_doc_tool, parse_justfile_recipes};
use crate::commands::way::WayCheckEntry;

/// Detection result for `check_documentation`: critical missing files and
/// improvement suggestions.
struct DocumentationGaps {
    critical: Vec<&'static str>,
    suggestions: Vec<String>,
    doc_tool: Option<String>,
}

fn documentation_gaps(repo_root: &Path) -> DocumentationGaps {
    let readme = repo_root.join("README.md").exists();
    let license = repo_root.join("LICENSE").exists() || repo_root.join("LICENSE.md").exists();
    let contributing = repo_root.join("CONTRIBUTING.md").exists();
    let gitignore = repo_root.join(".gitignore").exists();

    // docs/ folder with at least one file
    let docs_dir = repo_root.join("docs");
    let has_docs_folder = docs_dir.is_dir()
        && std::fs::read_dir(&docs_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);

    // Language-specific doc tool
    let doc_tool = detect_doc_tool(repo_root);

    // just docs recipe
    let justfile = repo_root.join("justfile");
    let has_just_docs = if justfile.exists() {
        parse_justfile_recipes(&justfile).contains(&"docs".to_string())
    } else {
        false
    };

    let mut gaps = DocumentationGaps {
        critical: Vec::new(),
        suggestions: Vec::new(),
        doc_tool,
    };
    if !readme {
        gaps.critical.push("README.md");
    }
    if !gitignore {
        gaps.critical.push(".gitignore");
    }
    if !license {
        gaps.suggestions.push("LICENSE".to_string());
    }
    if !contributing {
        gaps.suggestions.push("CONTRIBUTING.md".to_string());
    }
    if !has_docs_folder {
        gaps.suggestions
            .push("docs/ folder with content".to_string());
    }
    if gaps.doc_tool.is_none() {
        gaps.suggestions
            .push("language doc tool (e.g. cargo doc, mkdocs, sphinx, typedoc, godoc)".to_string());
    }
    if justfile.exists() && !has_just_docs {
        gaps.suggestions.push("just docs recipe".to_string());
    }
    gaps
}

pub(crate) fn check_documentation(repo_root: &Path) -> WayCheckEntry {
    let name = "Project documentation";
    let intent = Some(
        "Provide essential project identity, onboarding, and a discoverable docs/ folder with generated API docs."
            .to_string(),
    );
    let success_criteria = Some(
        "README, .gitignore, a docs/ folder with content, a language doc tool, and a 'just docs' recipe are all present."
            .to_string(),
    );
    let mk = |status: CheckStatus, message: String, suggestion: Option<String>| -> WayCheckEntry {
        WayCheckEntry {
            name: name.to_string(),
            status,
            message,
            intent,
            success_criteria,
            suggestion,
        }
    };

    let gaps = documentation_gaps(repo_root);

    if !gaps.critical.is_empty() {
        let also = if gaps.suggestions.is_empty() {
            String::new()
        } else {
            format!("; also consider: {}", gaps.suggestions.join(", "))
        };
        return mk(
            CheckStatus::Warn,
            format!("Missing critical files: {}", gaps.critical.join(", ")),
            Some(format!("Add: {}{}", gaps.critical.join(", "), also)),
        );
    }

    if gaps.suggestions.is_empty() {
        mk(
            CheckStatus::Pass,
            format!(
                "Complete (doc tool: {})",
                gaps.doc_tool.as_deref().unwrap_or("detected")
            ),
            None,
        )
    } else {
        mk(
            CheckStatus::Pass,
            "Essential files present".to_string(),
            Some(format!("Consider adding: {}", gaps.suggestions.join(", "))),
        )
    }
}
/// Scan docs/src/SUMMARY.md for a markdown link whose target filename starts
/// with "status" and return the resolved (non-lowercased) path.
fn find_linked_status_file(docs_dir: &Path) -> Option<std::path::PathBuf> {
    let summary_path = docs_dir.join("src").join("SUMMARY.md");
    let summary = std::fs::read_to_string(&summary_path).ok()?;
    summary.lines().find_map(|line| {
        let lower = line.to_lowercase();
        if !lower.contains("](") {
            return None;
        }
        let after_bracket = lower.split("](").nth(1)?;
        let target = after_bracket.split(')').next()?;
        let filename = std::path::Path::new(target)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if !filename.starts_with("status") {
            return None;
        }
        // Return the actual (non-lowercased) path for existence check
        let actual_target = line.split("](").nth(1)?.split(')').next()?;
        Some(
            docs_dir.join("src").join(
                actual_target
                    .trim_start_matches("./")
                    .trim_start_matches('/'),
            ),
        )
    })
}

/// Outcome for a SUMMARY.md status-page link result.
fn summary_outcome(linked: Option<std::path::PathBuf>) -> (CheckStatus, String, Option<String>) {
    match linked {
        Some(path) if path.exists() => (
            CheckStatus::Pass,
            "Status page linked in docs/src/SUMMARY.md and file exists".to_string(),
            None,
        ),
        Some(path) => (
            CheckStatus::Warn,
            format!(
                "SUMMARY.md links to {} but file is missing",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            Some("Create the status.md file that SUMMARY.md references".to_string()),
        ),
        None => (
            CheckStatus::Warn,
            "No status page linked in docs".to_string(),
            Some(
                "Add docs/src/status.md and link it from docs/src/SUMMARY.md — \
                 an overview of what's implemented and how it relates to specs helps \
                 users and contributors understand scope at a glance"
                    .to_string(),
            ),
        ),
    }
}

/// Generic fallback for non-mdBook layouts: check for status.md under docs/
fn find_fallback_status_page(docs_dir: &Path) -> bool {
    let candidates = [
        docs_dir.join("status.md"),
        docs_dir.join("STATUS.md"),
        docs_dir.join("src").join("status.md"),
    ];
    candidates.iter().any(|p| p.exists())
}

pub(crate) fn check_docs_status_page(repo_root: &Path) -> WayCheckEntry {
    let name = "Implementation status page";
    let intent = Some(
        "An overview page showing what is implemented and how features relate to their specs."
            .to_string(),
    );
    let success_criteria = Some(
        "A status.md page exists in docs, is referenced in the table of contents, and the file is present on disk."
            .to_string(),
    );
    let mk = |status: CheckStatus, message: String, suggestion: Option<String>| -> WayCheckEntry {
        WayCheckEntry {
            name: name.to_string(),
            status,
            message,
            intent,
            success_criteria,
            suggestion,
        }
    };

    let docs_dir = repo_root.join("docs");

    // No docs directory at all — not applicable
    if !docs_dir.is_dir() {
        return mk(
            CheckStatus::Pass,
            "No docs directory — skipped".to_string(),
            None,
        );
    }

    // mdBook pattern: docs/src/SUMMARY.md must link to a file named status*.md
    if docs_dir.join("src").join("SUMMARY.md").exists() {
        let (status, message, suggestion) = summary_outcome(find_linked_status_file(&docs_dir));
        return mk(status, message, suggestion);
    }

    // Generic fallback for non-mdBook layouts: check for status.md under docs/
    if find_fallback_status_page(&docs_dir) {
        return mk(
            CheckStatus::Pass,
            "Status page found in docs/".to_string(),
            None,
        );
    }

    mk(
        CheckStatus::Warn,
        "No status page in docs".to_string(),
        Some(
            "Add a status.md to docs/ — an implementation overview helps users understand \
             what's built and how it relates to specs"
                .to_string(),
        ),
    )
}
/// Outcome for the openspec-in-docs check when both openspec/ and docs/ exist.
fn openspec_docs_outcome(repo_root: &Path) -> (CheckStatus, String, Option<String>) {
    // Check docs CI workflow for openspec inclusion.
    // Doc tools (mdBook, MkDocs, etc.) only package their own src directory,
    // so openspec/ content won't appear in the deployed site unless the CI
    // workflow explicitly copies or symlinks it into the build tree.
    if docs_workflow_includes_openspec(repo_root) {
        return (
            CheckStatus::Pass,
            "Docs workflow includes openspec in build".to_string(),
            None,
        );
    }

    // Check if openspec content is already inside the docs source tree
    // (e.g. symlinked or copied manually)
    let docs_dir = repo_root.join("docs");
    let specs_in_docs = docs_dir.join("src").join("specs").is_dir()
        || docs_dir.join("src").join("openspec").is_dir()
        || docs_dir.join("specs").is_dir();

    if specs_in_docs {
        return (
            CheckStatus::Pass,
            "Specs already in docs source tree".to_string(),
            None,
        );
    }

    (
        CheckStatus::Warn,
        "openspec not included in deployed docs".to_string(),
        Some(
            "Your docs workflow doesn't include openspec/ in the build. \
             Specs explain the design rationale behind features and help users \
             understand architectural decisions. Add a step to your docs CI \
             workflow that copies openspec/specs/ into the docs source tree \
             before building (e.g. `cp -r openspec/specs docs/src/specs`), \
             then link them from your table of contents."
                .to_string(),
        ),
    )
}

pub(crate) fn check_docs_openspec_inclusion(repo_root: &Path) -> WayCheckEntry {
    let name = "Specs in deployed docs";
    let intent = Some(
        "Include openspec specifications in deployed documentation so users can discover \
         the design rationale behind features."
            .to_string(),
    );
    let success_criteria = Some(
        "When both openspec/ and a docs CI workflow exist, the workflow copies or references \
         openspec/ into the built output so specs ship with the deployed site."
            .to_string(),
    );
    let mk = |status: CheckStatus, message: String, suggestion: Option<String>| -> WayCheckEntry {
        WayCheckEntry {
            name: name.to_string(),
            status,
            message,
            intent,
            success_criteria,
            suggestion,
        }
    };

    let openspec_dir = repo_root.join("openspec");
    let docs_dir = repo_root.join("docs");

    // Only relevant when both openspec and docs are present
    if !openspec_dir.is_dir() || !docs_dir.is_dir() {
        return mk(
            CheckStatus::Pass,
            "Not applicable (openspec or docs missing)".to_string(),
            None,
        );
    }

    let (status, message, suggestion) = openspec_docs_outcome(repo_root);
    mk(status, message, suggestion)
}

/// Check whether any docs-related CI workflow references openspec.
///
/// Scans `.github/workflows/` for files whose name contains "doc" or "page"
/// and checks whether the workflow body mentions "openspec".
fn docs_workflow_includes_openspec(repo_root: &Path) -> bool {
    let workflows_dir = repo_root.join(".github").join("workflows");
    let Ok(entries) = std::fs::read_dir(&workflows_dir) else {
        return false;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if (name.contains("doc") || name.contains("page"))
            && (name.ends_with(".yml") || name.ends_with(".yaml"))
            && let Ok(content) = std::fs::read_to_string(&path)
            && content.to_lowercase().contains("openspec")
        {
            return true;
        }
    }
    false
}

/// Minimum number of non-empty body lines for an artifact to count as substantive.
const ARTIFACT_BODY_LINE_THRESHOLD: usize = 5;

/// Directories under each project whose markdown artifacts are checked for stubs.
const ARTIFACT_SCAN_DIRS: [&str; 5] = ["research", "handoffs", "designs", "plans", "reviews"];

fn markdown_body_lines(path: &Path) -> Option<Vec<String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut lines = content.lines();
    let mut in_frontmatter = false;

    if lines.clone().next() == Some("---") {
        in_frontmatter = true;
        lines.next();
        for line in lines.by_ref() {
            if line.trim() == "---" {
                in_frontmatter = false;
                break;
            }
        }
    }

    let body: Vec<String> = if in_frontmatter {
        // Unterminated frontmatter — treat entire file (minus opening delimiter) as body.
        content.lines().skip(1).map(|l| l.to_string()).collect()
    } else {
        lines.map(|l| l.to_string()).collect()
    };

    Some(body.into_iter().filter(|l| !l.trim().is_empty()).collect())
}

/// A single body line in the pipeline-step record convention, e.g.
/// `RO5U: wai-fvhv.28; verdict=SHIP; ...` — intentionally terse, not a stub.
fn is_structured_record(line: &str) -> bool {
    let mut chars = line.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {}
        _ => return false,
    }
    for c in chars {
        if c == ':' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ' ')) {
            return false;
        }
    }
    false
}

/// Scan one project's artifact directories and append stub paths to `stubs`.
fn collect_project_stubs(project_dir: &Path, projects_dir: &Path, stubs: &mut Vec<String>) {
    for subdir in ARTIFACT_SCAN_DIRS {
        let dir = project_dir.join(subdir);
        if !dir.is_dir() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(&dir) else {
            continue;
        };
        for file in files.filter_map(|e| e.ok()) {
            let path = file.path();
            if path.extension().is_none_or(|e| e != "md") {
                continue;
            }
            let is_stub = markdown_body_lines(&path).is_some_and(|body| {
                if body.len() >= ARTIFACT_BODY_LINE_THRESHOLD {
                    return false;
                }
                // Short bodies are substantive when they follow the
                // pipeline record convention (UPPERCASE-KEY: ...).
                body.first()
                    .map(|l| !is_structured_record(l))
                    .unwrap_or(true)
            });
            if is_stub
                && let Some(name) = path
                    .strip_prefix(projects_dir)
                    .ok()
                    .and_then(|p| p.to_str())
            {
                stubs.push(name.to_string());
            }
        }
    }
}

/// Scan all projects under `.wai/projects/` for empty/stub markdown artifacts.
fn scan_for_stubs(projects_dir: &Path) -> Vec<String> {
    let mut stubs: Vec<String> = Vec::new();
    let Ok(projects) = std::fs::read_dir(projects_dir) else {
        return stubs;
    };
    for project in projects.filter_map(|e| e.ok()) {
        collect_project_stubs(&project.path(), projects_dir, &mut stubs);
    }
    stubs
}

pub(crate) fn check_artifact_stubs(repo_root: &Path) -> WayCheckEntry {
    let name = "Project artifact completeness";
    let intent = Some(
        "Research and handoff docs must capture their findings so context survives across sessions."
            .to_string(),
    );
    let success_criteria = Some(
        "No research/handoff markdown artifacts are empty or stub-only (frontmatter with fewer than 5 body lines)."
            .to_string(),
    );
    let mk = |status: CheckStatus, message: String, suggestion: Option<String>| -> WayCheckEntry {
        WayCheckEntry {
            name: name.to_string(),
            status,
            message,
            intent,
            success_criteria,
            suggestion,
        }
    };

    let projects_dir = repo_root.join(".wai").join("projects");
    if !projects_dir.is_dir() {
        return mk(
            CheckStatus::Pass,
            "No project artifacts found".to_string(),
            None,
        );
    }

    let stubs = scan_for_stubs(&projects_dir);

    if stubs.is_empty() {
        mk(
            CheckStatus::Pass,
            "Research and handoff artifacts contain substantive content".to_string(),
            None,
        )
    } else {
        mk(
            CheckStatus::Warn,
            format!(
                "{} empty/stub artifact(s) detected: {}",
                stubs.len(),
                stubs.join(", ")
            ),
            Some(
                "Fill in the findings for these artifacts, or delete them if the work was abandoned"
                    .to_string(),
            ),
        )
    }
}
