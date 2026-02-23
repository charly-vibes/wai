use std::path::Path;
use std::time::SystemTime;

use miette::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::{ProjectConfig, STATE_FILE, wai_dir};
use crate::llm::detect_backend;

use super::require_project;

// ── Response parsing ───────────────────────────────────────────────────────────

/// Relevance level extracted from the LLM's artifact references.
#[derive(Debug, Clone, PartialEq)]
pub enum Relevance {
    High,
    Medium,
    Low,
}

impl Relevance {
    fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "high" => Some(Relevance::High),
            "medium" | "med" => Some(Relevance::Medium),
            "low" => Some(Relevance::Low),
            _ => None,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Relevance::High => "High",
            Relevance::Medium => "Medium",
            Relevance::Low => "Low",
        }
    }

    fn icon(&self) -> &str {
        match self {
            Relevance::High => "●",
            Relevance::Medium => "◐",
            Relevance::Low => "○",
        }
    }
}

/// A single artifact reference extracted from the LLM response.
#[derive(Debug, Clone)]
pub struct ArtifactRef {
    pub path: String,
    pub description: String,
    pub relevance: Option<Relevance>,
}

/// LLM response parsed into structured sections.
#[derive(Debug)]
pub struct ParsedResponse {
    pub answer: String,
    pub relevant_artifacts: Vec<ArtifactRef>,
    pub decision_chain: String,
    pub suggestions: Vec<String>,
    /// Original raw text from the LLM.
    pub raw: String,
}

/// Parse a raw LLM markdown response into structured sections.
///
/// Handles malformed output gracefully: if no `## ` headers are found, the
/// entire response is treated as the answer.
pub fn parse_response(raw: &str) -> ParsedResponse {
    let sections = split_sections(raw);

    let answer = if sections.is_empty() {
        // Completely malformed — treat whole text as answer
        raw.trim().to_string()
    } else {
        sections.get("Answer").cloned().unwrap_or_default()
    };

    let artifacts_text = sections
        .get("Relevant Artifacts")
        .cloned()
        .unwrap_or_default();
    let relevant_artifacts = parse_artifact_refs(&artifacts_text);

    let decision_chain = sections.get("Decision Chain").cloned().unwrap_or_default();

    let suggestions_text = sections.get("Suggestions").cloned().unwrap_or_default();
    let suggestions = parse_suggestions(&suggestions_text);

    ParsedResponse {
        answer,
        relevant_artifacts,
        decision_chain,
        suggestions,
        raw: raw.to_string(),
    }
}

/// Split markdown text into a map of `section_name → content` by `## ` headers.
fn split_sections(text: &str) -> std::collections::HashMap<String, String> {
    let mut sections = std::collections::HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_content = String::new();

    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(name) = current_name.take() {
                sections.insert(name, current_content.trim().to_string());
            }
            current_name = Some(heading.trim().to_string());
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if let Some(name) = current_name {
        sections.insert(name, current_content.trim().to_string());
    }
    sections
}

/// Parse artifact references from the `## Relevant Artifacts` section.
///
/// Recognises lines like:
/// - `- `.wai/projects/…/file.md` (High) — description`
/// - `- .wai/projects/…/file.md [Medium]: description`
/// - Lines without a recognisable path are skipped.
fn parse_artifact_refs(text: &str) -> Vec<ArtifactRef> {
    let mut refs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Strip leading list markers
        let content = line.trim_start_matches(['-', '*', '+']).trim();

        // Extract backtick-quoted path or plain path-like token
        let (path, rest) = if let Some(after_open) = content.strip_prefix('`') {
            if let Some(close) = after_open.find('`') {
                (
                    after_open[..close].to_string(),
                    after_open[close + 1..].trim(),
                )
            } else {
                extract_bare_path(content)
            }
        } else {
            extract_bare_path(content)
        };

        if path.is_empty() || !path.contains('/') {
            continue;
        }

        let relevance = extract_relevance(rest);
        let description = clean_description(rest);

        refs.push(ArtifactRef {
            path,
            description,
            relevance,
        });
    }
    refs
}

/// Extract the first whitespace-delimited token that looks like a path
/// (contains `/`) and return `(path, rest)`.
fn extract_bare_path(content: &str) -> (String, &str) {
    let trimmed = content.trim_start_matches('#').trim();
    if let Some(space) = trimmed.find(char::is_whitespace) {
        let token = &trimmed[..space];
        if token.contains('/') {
            return (token.to_string(), trimmed[space..].trim());
        }
    } else if trimmed.contains('/') {
        return (trimmed.to_string(), "");
    }
    (String::new(), content)
}

/// Look for `(High)`, `[High]`, `(Medium)`, etc. in a string.
fn extract_relevance(text: &str) -> Option<Relevance> {
    for word in text.split_whitespace() {
        let stripped =
            word.trim_matches(|c: char| c == '(' || c == ')' || c == '[' || c == ']' || c == ':');
        if let Some(r) = Relevance::from_str(stripped) {
            return Some(r);
        }
    }
    None
}

/// Remove relevance markers and leading punctuation to get a clean description.
fn clean_description(text: &str) -> String {
    // Strip leading brackets/parens relevance tokens then em-dash or colon separators
    let mut s = text.to_string();
    // Remove parenthesised or bracketed relevance markers
    for marker in &["(High)", "(Medium)", "(Low)", "[High]", "[Medium]", "[Low]"] {
        s = s.replace(marker, "");
    }
    // Strip leading —, -, :
    let trimmed = s.trim().trim_start_matches(['—', '-', ':']).trim();
    trimmed.to_string()
}

/// Parse bullet/numbered points from the `## Suggestions` section.
fn parse_suggestions(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let stripped = line
                .trim()
                .trim_start_matches(|c: char| {
                    c.is_ascii_digit() || c == '.' || c == '-' || c == '*' || c == '+'
                })
                .trim();
            if stripped.is_empty() {
                None
            } else {
                Some(stripped.to_string())
            }
        })
        .collect()
}

// ── Terminal formatter ─────────────────────────────────────────────────────────

fn separator() {
    println!("  {}", "─".repeat(58).dimmed());
}

fn section_header(title: &str) {
    separator();
    println!("  {}", title.bold());
    separator();
}

/// Pretty-print the parsed response to stdout with colors and icons.
pub fn format_terminal(response: &ParsedResponse, query: &str) {
    println!();
    println!("  {} {}", "◆".cyan(), query.bold());
    println!();

    // Answer
    section_header("Answer");
    println!();
    for line in response.answer.lines() {
        println!("  {}", line);
    }
    println!();

    // Relevant Artifacts
    if !response.relevant_artifacts.is_empty() {
        section_header("Relevant Artifacts");
        println!();
        for artifact in &response.relevant_artifacts {
            let relevance_display = match &artifact.relevance {
                Some(r) => format!("{} [{}]", r.icon(), r.as_str()),
                None => "○".to_string(),
            };
            let colored = match &artifact.relevance {
                Some(Relevance::High) => relevance_display.red().to_string(),
                Some(Relevance::Medium) => relevance_display.yellow().to_string(),
                Some(Relevance::Low) => relevance_display.green().to_string(),
                None => relevance_display.dimmed().to_string(),
            };
            // file:line format makes paths clickable in supporting terminals
            let clickable_path = format!("{}:1", artifact.path);
            println!("  {}  {}", colored, clickable_path.cyan());
            if !artifact.description.is_empty() {
                println!("     {}", artifact.description.dimmed());
            }
            println!();
        }
    }

    // Decision Chain
    if !response.decision_chain.is_empty() {
        section_header("Decision Chain");
        println!();
        for line in response.decision_chain.lines() {
            println!("  {}", line);
        }
        println!();
    }

    // Suggestions
    if !response.suggestions.is_empty() {
        section_header("Suggestions");
        println!();
        for suggestion in &response.suggestions {
            println!("  {} {}", "→".cyan(), suggestion);
        }
        println!();
    }
}

// ── JSON formatter ─────────────────────────────────────────────────────────────

/// Serialize the parsed response as JSON for machine-readable output.
pub fn format_json(response: &ParsedResponse, query: &str) -> String {
    use serde_json::{Value, json};

    let artifacts: Vec<Value> = response
        .relevant_artifacts
        .iter()
        .map(|a| {
            json!({
                "path": a.path,
                "relevance": a.relevance.as_ref().map(|r| r.as_str()),
                "description": a.description,
            })
        })
        .collect();

    let v = json!({
        "query": query,
        "answer": response.answer,
        "relevant_artifacts": artifacts,
        "decision_chain": response.decision_chain,
        "suggestions": response.suggestions,
    });

    serde_json::to_string_pretty(&v).unwrap_or_else(|_| response.raw.clone())
}

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
    // Read in Phase 4 output formatter
    #[allow(dead_code)]
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
        parts.push("*Note: Context was truncated to fit within token limits.*\n".to_string());
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

pub fn run(query: String, no_llm: bool, json: bool) -> Result<()> {
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

    if !json {
        println!();
        println!("  {} {}", "◆".cyan(), query.bold());
        println!("  {} Querying {} …", "○".dimmed(), backend.name());
    }

    let raw_response = match backend.complete(&prompt) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "  {} LLM error: {}. Falling back to search.",
                "⚠".yellow(),
                e
            );
            return super::search::run(query, None, None, false, None);
        }
    };

    let parsed = parse_response(&raw_response);

    if json {
        println!("{}", format_json(&parsed, &query));
    } else {
        format_terminal(&parsed, &query);
    }

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

    // ── parse_response ──

    #[test]
    fn parse_well_formed_response_extracts_all_sections() {
        let raw = "## Answer\nTOML is simple.\n## Relevant Artifacts\n- `.wai/a.md` (High) — key doc\n## Decision Chain\nResearch → Design\n## Suggestions\n- Use TOML everywhere";
        let p = parse_response(raw);
        assert_eq!(p.answer, "TOML is simple.");
        assert_eq!(p.decision_chain, "Research → Design");
        assert_eq!(p.suggestions, vec!["Use TOML everywhere"]);
    }

    #[test]
    fn parse_malformed_response_uses_raw_as_answer() {
        let raw = "No headers here, just plain text.";
        let p = parse_response(raw);
        assert_eq!(p.answer, raw);
        assert!(p.relevant_artifacts.is_empty());
        assert!(p.suggestions.is_empty());
    }

    #[test]
    fn parse_missing_section_is_empty() {
        let raw = "## Answer\nSome answer.\n## Suggestions\n- Do this";
        let p = parse_response(raw);
        assert!(p.relevant_artifacts.is_empty());
        assert_eq!(p.decision_chain, "");
        assert_eq!(p.suggestions, vec!["Do this"]);
    }

    // ── split_sections ──

    #[test]
    fn split_sections_handles_multiple_sections() {
        let text = "## Foo\nfoo content\n## Bar\nbar content";
        let sections = split_sections(text);
        assert_eq!(sections.get("Foo").map(|s| s.as_str()), Some("foo content"));
        assert_eq!(sections.get("Bar").map(|s| s.as_str()), Some("bar content"));
    }

    #[test]
    fn split_sections_empty_text_returns_empty_map() {
        let sections = split_sections("");
        assert!(sections.is_empty());
    }

    #[test]
    fn split_sections_text_without_headers_returns_empty_map() {
        let sections = split_sections("just plain text, no headers");
        assert!(sections.is_empty());
    }

    // ── extract_relevance ──

    #[test]
    fn extract_relevance_parses_parenthesised_high() {
        assert_eq!(
            extract_relevance("(High) — important"),
            Some(Relevance::High)
        );
    }

    #[test]
    fn extract_relevance_parses_bracketed_medium() {
        assert_eq!(
            extract_relevance("[Medium]: explanation"),
            Some(Relevance::Medium)
        );
    }

    #[test]
    fn extract_relevance_parses_low() {
        assert_eq!(
            extract_relevance("(Low) — less important"),
            Some(Relevance::Low)
        );
    }

    #[test]
    fn extract_relevance_returns_none_when_absent() {
        assert_eq!(extract_relevance("no relevance marker here"), None);
    }

    #[test]
    fn extract_relevance_case_insensitive() {
        assert_eq!(extract_relevance("(high)"), Some(Relevance::High));
        assert_eq!(extract_relevance("(MEDIUM)"), Some(Relevance::Medium));
    }

    // ── parse_artifact_refs ──

    #[test]
    fn parse_artifact_refs_extracts_backtick_path_and_relevance() {
        let text = "- `.wai/projects/why/research/2024-01-01.md` (High) — explains rationale";
        let refs = parse_artifact_refs(text);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, ".wai/projects/why/research/2024-01-01.md");
        assert_eq!(refs[0].relevance, Some(Relevance::High));
        assert!(refs[0].description.contains("explains rationale"));
    }

    #[test]
    fn parse_artifact_refs_extracts_bare_path() {
        let text = "- .wai/projects/why/design/arch.md [Medium]: architecture doc";
        let refs = parse_artifact_refs(text);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, ".wai/projects/why/design/arch.md");
        assert_eq!(refs[0].relevance, Some(Relevance::Medium));
    }

    #[test]
    fn parse_artifact_refs_skips_lines_without_paths() {
        let text = "- No path here\n- also no path\n- `.wai/foo/bar.md` (Low) — desc";
        let refs = parse_artifact_refs(text);
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn parse_artifact_refs_empty_text_returns_empty() {
        let refs = parse_artifact_refs("");
        assert!(refs.is_empty());
    }

    // ── parse_suggestions ──

    #[test]
    fn parse_suggestions_extracts_bullet_points() {
        let text = "- Update docs\n- Add tests\n- Refactor config";
        let suggestions = parse_suggestions(text);
        assert_eq!(
            suggestions,
            vec!["Update docs", "Add tests", "Refactor config"]
        );
    }

    #[test]
    fn parse_suggestions_strips_numbering() {
        let text = "1. First suggestion\n2. Second suggestion";
        let suggestions = parse_suggestions(text);
        assert_eq!(suggestions, vec!["First suggestion", "Second suggestion"]);
    }

    #[test]
    fn parse_suggestions_skips_blank_lines() {
        let text = "- A\n\n- B";
        let suggestions = parse_suggestions(text);
        assert_eq!(suggestions, vec!["A", "B"]);
    }

    // ── format_json ──

    #[test]
    fn format_json_contains_required_fields() {
        let response = ParsedResponse {
            answer: "Because TOML is simpler.".to_string(),
            relevant_artifacts: vec![ArtifactRef {
                path: ".wai/projects/p/research/r.md".to_string(),
                description: "key doc".to_string(),
                relevance: Some(Relevance::High),
            }],
            decision_chain: "Research → Design".to_string(),
            suggestions: vec!["Use TOML everywhere".to_string()],
            raw: String::new(),
        };
        let json = format_json(&response, "why TOML?");
        assert!(json.contains("\"query\""));
        assert!(json.contains("why TOML?"));
        assert!(json.contains("\"answer\""));
        assert!(json.contains("Because TOML is simpler."));
        assert!(json.contains("\"relevant_artifacts\""));
        assert!(json.contains("\"High\""));
        assert!(json.contains("\"decision_chain\""));
        assert!(json.contains("\"suggestions\""));
        assert!(json.contains("Use TOML everywhere"));
    }

    #[test]
    fn format_json_null_relevance_when_none() {
        let response = ParsedResponse {
            answer: String::new(),
            relevant_artifacts: vec![ArtifactRef {
                path: ".wai/projects/p/research/r.md".to_string(),
                description: String::new(),
                relevance: None,
            }],
            decision_chain: String::new(),
            suggestions: vec![],
            raw: String::new(),
        };
        let json = format_json(&response, "q");
        assert!(json.contains("\"relevance\": null"));
    }

    // ── Relevance ──

    #[test]
    fn relevance_as_str_roundtrips() {
        assert_eq!(Relevance::High.as_str(), "High");
        assert_eq!(Relevance::Medium.as_str(), "Medium");
        assert_eq!(Relevance::Low.as_str(), "Low");
    }

    #[test]
    fn relevance_from_str_accepts_med_alias() {
        assert_eq!(Relevance::from_str("med"), Some(Relevance::Medium));
    }
}
