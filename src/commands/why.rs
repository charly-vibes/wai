use std::path::Path;
use std::time::SystemTime;

use miette::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::{wai_dir, ProjectConfig, STATE_FILE};
use crate::llm::detect_backend;

use super::require_project;

// Approximate character budget: 100K tokens × 4 chars/token
const MAX_CONTEXT_CHARS: usize = 400_000;
const MAX_ARTIFACTS_WHEN_TRUNCATING: usize = 50;

// ── Data types ────────────────────────────────────────────────────────────────

/// A single artifact read from the `.wai/` directory tree.
#[derive(Debug, Clone)]
pub struct Artifact {
    /// Path relative to project root (e.g. `.wai/projects/foo/research/2024-01-01.md`).
    pub rel_path: String,
    pub kind: ArtifactKind,
    pub content: String,
    /// Used for recency-based sorting and truncation priority.
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactKind {
    Research,
    Design,
    Plan,
    Handoff,
}

impl ArtifactKind {
    fn from_path(path: &str) -> Option<Self> {
        if path.contains("/research/") {
            Some(ArtifactKind::Research)
        } else if path.contains("/designs/") {
            Some(ArtifactKind::Design)
        } else if path.contains("/plans/") {
            Some(ArtifactKind::Plan)
        } else if path.contains("/handoffs/") {
            Some(ArtifactKind::Handoff)
        } else {
            None
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ArtifactKind::Research => "research",
            ArtifactKind::Design => "design",
            ArtifactKind::Plan => "plan",
            ArtifactKind::Handoff => "handoff",
        }
    }
}

/// Lightweight project metadata included in prompts.
#[derive(Debug, Default)]
pub struct ProjectMeta {
    pub current_phase: Option<String>,
    pub recent_commits: Vec<String>,
}

/// Everything gathered before calling the LLM.
#[derive(Debug)]
pub struct GatheredContext {
    pub query: String,
    pub is_file_query: bool,
    pub artifacts: Vec<Artifact>,
    /// Git log for file queries; `None` for natural-language queries.
    pub git_context: Option<String>,
    pub meta: ProjectMeta,
    /// True when artifacts were truncated to fit the context budget.
    pub truncated: bool,
}

impl GatheredContext {
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

// ── File-path detection ───────────────────────────────────────────────────────

/// Return `true` if `query` looks like a file path rather than a natural-language question.
///
/// Heuristics (conservative — false negatives are fine, false positives are disruptive):
/// - File actually exists at the path
/// - Starts with `./`, `../`, or `src/`
/// - Contains `/` with no spaces (bare path component like `path/to/file.rs`)
pub fn detect_file_query(query: &str) -> bool {
    // Spaces mean it's almost certainly a question, not a path
    if query.contains(' ') {
        return false;
    }
    if Path::new(query).exists() {
        return true;
    }
    if query.starts_with("./") || query.starts_with("../") || query.starts_with("src/") {
        return true;
    }
    if query.contains('/') {
        return true;
    }
    false
}

// ── Context gathering ─────────────────────────────────────────────────────────

/// Gather all context needed to answer `query`.
pub fn gather_context(project_root: &Path, query: &str) -> GatheredContext {
    let is_file_query = detect_file_query(query);

    // Read artifacts, sorted most-recent first
    let mut artifacts = read_artifacts(project_root);
    artifacts.sort_by(|a, b| {
        b.modified
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .cmp(&a.modified.unwrap_or(SystemTime::UNIX_EPOCH))
    });

    let (artifacts, truncated) = truncate_context(artifacts, query, MAX_CONTEXT_CHARS);

    let git_context = if is_file_query {
        gather_git_file_context(query)
    } else {
        None
    };

    let meta = gather_meta(project_root);

    GatheredContext {
        query: query.to_string(),
        is_file_query,
        artifacts,
        git_context,
        meta,
        truncated,
    }
}

/// Read all `.md` artifacts from the `.wai/` tree, skipping hidden files and
/// non-artifact directories (resources, plugins, archives config files).
fn read_artifacts(project_root: &Path) -> Vec<Artifact> {
    let root = wai_dir(project_root);
    let mut artifacts = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x == "md")
                .unwrap_or(false)
        })
    {
        // Skip dot-files (e.g. .state)
        if entry
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        let path_str = entry.path().to_string_lossy().to_string();
        let kind = match ArtifactKind::from_path(&path_str) {
            Some(k) => k,
            None => continue, // skip non-artifact dirs (resources, plugins, etc.)
        };

        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let modified = entry.metadata().ok().and_then(|m| m.modified().ok());

        let rel_path = entry
            .path()
            .strip_prefix(project_root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        artifacts.push(Artifact {
            rel_path,
            kind,
            content,
            modified,
        });
    }

    artifacts
}

/// Trim artifact list to stay within `max_chars`.
///
/// Priority order:
/// 1. Hard cap at `MAX_ARTIFACTS_WHEN_TRUNCATING` items (most recent first)
/// 2. Within that set, boost artifacts mentioning query terms
/// 3. Fill greedily until `max_chars` is exhausted
///
/// Returns `(selected, was_truncated)`.
pub fn truncate_context(
    artifacts: Vec<Artifact>,
    query: &str,
    max_chars: usize,
) -> (Vec<Artifact>, bool) {
    let total_chars: usize = artifacts.iter().map(|a| a.content.len()).sum();
    if total_chars <= max_chars {
        return (artifacts, false);
    }

    // Hard count cap (already sorted by recency)
    let candidates: Vec<Artifact> = artifacts
        .into_iter()
        .take(MAX_ARTIFACTS_WHEN_TRUNCATING)
        .collect();

    // Score by query-term relevance then fill greedily
    let query_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(String::from)
        .collect();

    let mut scored: Vec<(usize, Artifact)> = candidates
        .into_iter()
        .map(|a| {
            let score = score_relevance(&a.content, &query_terms);
            (score, a)
        })
        .collect();
    // Higher score first; ties keep recency order (stable sort)
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    let mut selected = Vec::new();
    let mut chars_used = 0;

    for (_, artifact) in scored {
        if chars_used + artifact.content.len() > max_chars {
            break;
        }
        chars_used += artifact.content.len();
        selected.push(artifact);
    }

    (selected, true)
}

fn score_relevance(content: &str, terms: &[String]) -> usize {
    let lower = content.to_lowercase();
    terms.iter().filter(|t| lower.contains(t.as_str())).count()
}

// ── Git context ───────────────────────────────────────────────────────────────

/// Gather git log for a specific file. Returns `None` if git is unavailable,
/// the directory isn't a repo, or the file isn't tracked.
fn gather_git_file_context(file_path: &str) -> Option<String> {
    // Verify we're in a git repo
    let in_repo = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !in_repo {
        return None;
    }

    let log = std::process::Command::new("git")
        .args(["log", "--oneline", "-10", "--", file_path])
        .output()
        .ok()?;

    if !log.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&log.stdout).to_string();
    if text.trim().is_empty() {
        return None;
    }

    Some(format!("Git history for {}:\n{}", file_path, text.trim()))
}

// ── Project metadata ──────────────────────────────────────────────────────────

fn gather_meta(project_root: &Path) -> ProjectMeta {
    ProjectMeta {
        current_phase: read_first_project_phase(project_root),
        recent_commits: gather_recent_commits(),
    }
}

fn read_first_project_phase(project_root: &Path) -> Option<String> {
    let projects = crate::config::projects_dir(project_root);
    let entries = std::fs::read_dir(&projects).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let state_path = entry.path().join(STATE_FILE);
        if let Ok(state) = crate::state::ProjectState::load(&state_path) {
            return Some(state.current.to_string());
        }
    }
    None
}

fn gather_recent_commits() -> Vec<String> {
    std::process::Command::new("git")
        .args(["log", "--oneline", "-5"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

// ── Prompt builder ────────────────────────────────────────────────────────────

/// Build the prompt sent to the LLM from gathered context.
pub fn build_prompt(ctx: &GatheredContext) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push(
        "You are an oracle helping understand why code and decisions exist as they do.\n"
            .to_string(),
    );
    parts.push(format!("# User Question\n{}\n", ctx.query));

    // Project metadata
    let mut meta_lines = Vec::new();
    if let Some(ref phase) = ctx.meta.current_phase {
        meta_lines.push(format!("- Current phase: {}", phase));
    }
    if !ctx.meta.recent_commits.is_empty() {
        meta_lines.push(format!(
            "- Recent commits:\n{}",
            ctx.meta
                .recent_commits
                .iter()
                .map(|c| format!("  - {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    if !meta_lines.is_empty() {
        parts.push(format!("# Project Context\n{}\n", meta_lines.join("\n")));
    }

    // Artifacts — wrap in code fences and escape injection attempts
    if !ctx.artifacts.is_empty() {
        let mut artifact_text = String::from("# Available Artifacts\n");
        for artifact in &ctx.artifacts {
            artifact_text.push_str(&format!(
                "\n## {} ({})\n```\n{}\n```\n",
                artifact.rel_path,
                artifact.kind.label(),
                escape_artifact(&artifact.content),
            ));
        }
        parts.push(artifact_text);
    }

    // Git context for file queries
    if let Some(ref git) = ctx.git_context {
        parts.push(format!("# Git Context\n{}\n", git));
    }

    if ctx.truncated {
        parts.push(
            "*Note: Context was truncated to fit within token limits.*\n".to_string(),
        );
    }

    parts.push(
        "# Task\nIdentify 3-5 most relevant artifacts. Explain why each is relevant. \
        Synthesize a narrative showing how the decision evolved (research → design → plan). \
        Suggest concrete next steps.\n\n\
        Format your response as Markdown with these sections:\n\
        ## Answer\n## Relevant Artifacts\n## Decision Chain\n## Suggestions"
            .to_string(),
    );

    parts.join("\n")
}

/// Escape content to prevent triple-backtick fences from breaking the prompt structure.
fn escape_artifact(content: &str) -> String {
    content.replace("```", "~~~")
}

// ── Command entry point ───────────────────────────────────────────────────────

pub fn run(query: String, no_llm: bool) -> Result<()> {
    let project_root = require_project()?;

    if no_llm {
        return super::search::run(query, None, None, false, None);
    }

    let ctx = gather_context(&project_root, &query);

    // 2.6: Warn when no artifacts are present
    if ctx.is_empty() {
        println!();
        println!("  {} No artifacts found in .wai/", "⚠".yellow());
        println!(
            "  {} Add some first: {}",
            "→".cyan(),
            "wai add research \"your notes\"".bold()
        );
        println!();
        return Ok(());
    }

    if ctx.truncated {
        println!(
            "  {} Context truncated to {} most relevant artifacts",
            "○".dimmed(),
            ctx.artifacts.len()
        );
    }

    // Load config for LLM backend selection
    let why_cfg = ProjectConfig::load(&project_root)
        .ok()
        .and_then(|c| c.why)
        .unwrap_or_default();

    // Detect backend; fall back to search if none available
    let backend: Box<dyn crate::llm::LlmClient> = match detect_backend(&why_cfg) {
        Some(b) => b,
        None => {
            eprintln!(
                "  {} No LLM available. Falling back to `wai search`.",
                "⚠".yellow()
            );
            eprintln!(
                "  {} Set ANTHROPIC_API_KEY or install Ollama for synthesized answers.",
                "○".dimmed()
            );
            return super::search::run(query, None, None, false, None);
        }
    };

    // Build prompt and call the LLM
    let prompt = build_prompt(&ctx);

    println!();
    println!("  {} {}", "◆".cyan(), query.bold());
    println!(
        "  {} Querying {} …",
        "○".dimmed(),
        backend.name()
    );

    let response = match backend.complete(&prompt) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  {} LLM error: {}. Falling back to search.", "⚠".yellow(), e);
            return super::search::run(query, None, None, false, None);
        }
    };

    // Output will be formatted in Phase 4 (wai-qrg).
    // For now, print the raw response.
    println!();
    println!("{}", response);
    println!();

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── detect_file_query ──

    #[test]
    fn natural_language_question_is_not_file_query() {
        assert!(!detect_file_query("why use TOML for config?"));
        assert!(!detect_file_query("what is the design philosophy"));
        assert!(!detect_file_query("config vs yaml"));
    }

    #[test]
    fn path_with_slash_and_no_spaces_is_file_query() {
        assert!(detect_file_query("src/config.rs"));
        assert!(detect_file_query("path/to/file.rs"));
    }

    #[test]
    fn dotslash_prefix_is_file_query() {
        assert!(detect_file_query("./src/main.rs"));
        assert!(detect_file_query("../other/file.rs"));
    }

    #[test]
    fn src_prefix_is_file_query() {
        assert!(detect_file_query("src/commands/why.rs"));
    }

    #[test]
    fn question_with_slash_but_has_spaces_is_not_file_query() {
        assert!(!detect_file_query("config/yaml vs toml"));
    }

    #[test]
    fn existing_file_on_disk_is_file_query() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("my_module.rs");
        fs::write(&file, "fn main() {}").unwrap();
        // Use absolute path string
        assert!(detect_file_query(file.to_str().unwrap()));
    }

    // ── truncate_context ──

    fn make_artifact(kind: ArtifactKind, content: &str) -> Artifact {
        Artifact {
            rel_path: format!(".wai/projects/test/{}/file.md", kind.label()),
            kind,
            content: content.to_string(),
            modified: None,
        }
    }

    #[test]
    fn under_limit_returns_all_unmodified() {
        let artifacts = vec![
            make_artifact(ArtifactKind::Research, "small content"),
            make_artifact(ArtifactKind::Design, "also small"),
        ];
        let (result, truncated) = truncate_context(artifacts.clone(), "query", 1_000_000);
        assert!(!truncated);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn over_limit_returns_truncated_flag() {
        let big = "x".repeat(300_001);
        let artifacts = vec![
            make_artifact(ArtifactKind::Research, &big),
            make_artifact(ArtifactKind::Design, &big),
        ];
        let (_, truncated) = truncate_context(artifacts, "query", 300_000);
        assert!(truncated);
    }

    #[test]
    fn relevant_artifact_kept_over_irrelevant() {
        let relevant = make_artifact(ArtifactKind::Research, "TOML configuration design");
        let irrelevant = make_artifact(ArtifactKind::Design, "z".repeat(200_000).as_str());

        // Budget allows only one
        let budget = relevant.content.len() + 1;
        let (result, _) = truncate_context(vec![irrelevant, relevant], "TOML", budget);
        assert_eq!(result.len(), 1);
        assert!(result[0].content.contains("TOML"));
    }

    #[test]
    fn empty_artifacts_returns_empty_not_truncated() {
        let (result, truncated) = truncate_context(vec![], "anything", MAX_CONTEXT_CHARS);
        assert!(!truncated);
        assert!(result.is_empty());
    }

    // ── read_artifacts ──

    fn setup_wai_project(tmp: &TempDir) {
        let wai = tmp.path().join(".wai");
        let research = wai.join("projects").join("myproj").join("research");
        let designs = wai.join("projects").join("myproj").join("designs");
        let plans = wai.join("projects").join("myproj").join("plans");
        fs::create_dir_all(&research).unwrap();
        fs::create_dir_all(&designs).unwrap();
        fs::create_dir_all(&plans).unwrap();

        fs::write(research.join("2024-01-01-notes.md"), "research content").unwrap();
        fs::write(designs.join("2024-01-02-arch.md"), "design content").unwrap();
        fs::write(plans.join("2024-01-03-plan.md"), "plan content").unwrap();
        // A dot-file that should be ignored
        fs::write(research.join(".state"), "ignored").unwrap();
        // A non-md file that should be ignored
        fs::write(research.join("notes.txt"), "ignored").unwrap();
    }

    #[test]
    fn reads_artifacts_from_all_subdirs() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);
        let artifacts = read_artifacts(tmp.path());
        assert_eq!(artifacts.len(), 3, "should read research, design, plan");
    }

    #[test]
    fn artifact_kinds_are_detected_correctly() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);
        let mut artifacts = read_artifacts(tmp.path());
        artifacts.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        let kinds: Vec<&str> = artifacts.iter().map(|a| a.kind.label()).collect();
        assert!(kinds.contains(&"research"));
        assert!(kinds.contains(&"design"));
        assert!(kinds.contains(&"plan"));
    }

    #[test]
    fn empty_wai_dir_returns_no_artifacts() {
        let tmp = TempDir::new().unwrap();
        let wai = tmp.path().join(".wai");
        fs::create_dir_all(&wai).unwrap();
        let artifacts = read_artifacts(tmp.path());
        assert!(artifacts.is_empty());
    }

    // ── build_prompt ──

    fn make_ctx_for_prompt(query: &str, artifacts: Vec<Artifact>) -> GatheredContext {
        GatheredContext {
            query: query.to_string(),
            is_file_query: false,
            artifacts,
            git_context: None,
            meta: ProjectMeta::default(),
            truncated: false,
        }
    }

    #[test]
    fn prompt_contains_query() {
        let ctx = make_ctx_for_prompt("why use TOML?", vec![]);
        let prompt = build_prompt(&ctx);
        assert!(prompt.contains("why use TOML?"));
    }

    #[test]
    fn prompt_contains_artifact_content() {
        let ctx = make_ctx_for_prompt(
            "query",
            vec![make_artifact(ArtifactKind::Research, "TOML is simple")],
        );
        let prompt = build_prompt(&ctx);
        assert!(prompt.contains("TOML is simple"));
        assert!(prompt.contains("research"));
    }

    #[test]
    fn prompt_escapes_backtick_fences() {
        let ctx = make_ctx_for_prompt(
            "q",
            vec![make_artifact(
                ArtifactKind::Design,
                "code ```rust fn main() {}``` end",
            )],
        );
        let prompt = build_prompt(&ctx);
        assert!(!prompt.contains("```rust"));
        assert!(prompt.contains("~~~rust"));
    }

    #[test]
    fn prompt_includes_git_context_when_present() {
        let mut ctx = make_ctx_for_prompt("src/config.rs", vec![]);
        ctx.is_file_query = true;
        ctx.git_context = Some("Git history for src/config.rs:\nabc123 init".to_string());
        let prompt = build_prompt(&ctx);
        assert!(prompt.contains("Git history"));
        assert!(prompt.contains("abc123"));
    }

    #[test]
    fn prompt_includes_truncation_notice_when_truncated() {
        let mut ctx = make_ctx_for_prompt("q", vec![]);
        ctx.truncated = true;
        let prompt = build_prompt(&ctx);
        assert!(prompt.to_lowercase().contains("truncated"));
    }

    #[test]
    fn prompt_includes_task_format_sections() {
        let ctx = make_ctx_for_prompt("q", vec![]);
        let prompt = build_prompt(&ctx);
        assert!(prompt.contains("## Answer"));
        assert!(prompt.contains("## Decision Chain"));
        assert!(prompt.contains("## Suggestions"));
    }

    // ── gathered_context is_empty ──

    #[test]
    fn gathered_context_is_empty_when_no_artifacts() {
        let ctx = GatheredContext {
            query: "test".to_string(),
            is_file_query: false,
            artifacts: vec![],
            git_context: None,
            meta: ProjectMeta::default(),
            truncated: false,
        };
        assert!(ctx.is_empty());
    }

    #[test]
    fn gathered_context_not_empty_with_artifacts() {
        let ctx = GatheredContext {
            query: "test".to_string(),
            is_file_query: false,
            artifacts: vec![make_artifact(ArtifactKind::Research, "content")],
            git_context: None,
            meta: ProjectMeta::default(),
            truncated: false,
        };
        assert!(!ctx.is_empty());
    }
}
