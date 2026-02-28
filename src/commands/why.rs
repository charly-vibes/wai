use std::path::Path;
use std::time::SystemTime;

use miette::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::{LlmConfig, ProjectConfig, STATE_FILE, wai_dir};
use crate::context::current_context;
use crate::error::WaiError;
use crate::llm::{
    AGENT_SENTINEL, LlmError, claude_binary_exists, detect_backend, ollama_binary_exists,
};

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
        gather_git_file_context(query, project_root)
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
///
/// All git commands run with `project_root` as the working directory so that
/// relative paths are resolved correctly regardless of where wai was invoked.
fn gather_git_file_context(file_path: &str, project_root: &Path) -> Option<String> {
    // Verify project_root is inside a git repo.
    let in_repo = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_root)
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !in_repo {
        return None;
    }

    // Resolve the path so it works when git runs from project_root.
    // If the path can be canonicalized (file exists), make it relative to
    // project_root so git can match it against its index. If it can't be
    // canonicalized (e.g. non-existent path given as hint), use as-is.
    let resolved: String = Path::new(file_path)
        .canonicalize()
        .ok()
        .and_then(|abs| abs.strip_prefix(project_root).ok().map(|r| r.to_path_buf()))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_path.to_string());

    let log = std::process::Command::new("git")
        .args(["log", "--oneline", "-10", "--", &resolved])
        .current_dir(project_root)
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
        recent_commits: gather_recent_commits(project_root),
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

fn gather_recent_commits(project_root: &Path) -> Vec<String> {
    std::process::Command::new("git")
        .args(["log", "--oneline", "-5"])
        .current_dir(project_root)
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

// ── Error messaging ───────────────────────────────────────────────────────────

/// Map an `LlmError` to a user-visible message and an optional remediation hint.
///
/// The hint text mirrors the `help(...)` strings on the `WaiError::Llm*` miette
/// diagnostic variants so the two stay consistent.
pub fn llm_error_hint(err: &LlmError) -> (String, Option<String>) {
    match err {
        LlmError::InvalidApiKey => (
            "API key is invalid or missing".to_string(),
            Some("Set ANTHROPIC_API_KEY or add `api_key` to [llm] in .wai/config.toml".to_string()),
        ),
        LlmError::RateLimit => (
            "Rate limit exceeded".to_string(),
            Some(
                "Wait 60 seconds and retry, or use Ollama for unlimited local queries".to_string(),
            ),
        ),
        LlmError::NetworkError(msg) => (
            format!("Network error: {}", msg),
            Some("Check your internet connection and retry".to_string()),
        ),
        LlmError::ModelNotFound(model) => (
            format!("Model '{}' not found", model),
            Some(format!("Run `ollama pull {}` to download the model", model)),
        ),
        LlmError::Other(msg) => (msg.clone(), None),
    }
}

// ── Explicit-backend agent hint (7.1) ────────────────────────────────────────

/// When an explicit backend (`[llm] llm = "claude"` or `"ollama"`) fails and
/// the system falls back to search inside a Claude Code session, return a hint
/// suggesting agent mode as a zero-cost alternative.
///
/// Returns `None` when the hint is not applicable (auto-detect config, or not
/// inside a Claude Code session).
pub fn explicit_backend_agent_hint(cfg: &LlmConfig) -> Option<String> {
    let is_explicit = matches!(cfg.llm.as_deref(), Some("claude") | Some("ollama"));
    if is_explicit && crate::llm::in_agent_session() {
        Some(
            "You're in a Claude Code session — try `llm = \"agent\"` in [llm] for zero-cost queries."
                .to_string(),
        )
    } else {
        None
    }
}

// ── Fallback mode (6.4) ───────────────────────────────────────────────────────

/// Controls behavior when no LLM is available or an LLM call fails.
#[derive(Debug, PartialEq)]
pub enum FallbackMode {
    /// Gracefully degrade to `wai search` (default).
    Search,
    /// Return an error; do not fall back.
    Error,
}

/// Determine fallback behavior from config.
///
/// `fallback = "error"` → propagate errors; anything else → fall back to search.
pub fn fallback_mode(cfg: &LlmConfig) -> FallbackMode {
    match cfg.fallback.as_deref() {
        Some("error") => FallbackMode::Error,
        _ => FallbackMode::Search,
    }
}

// ── Privacy notice (6.5 / 6.6) ───────────────────────────────────────────────

/// Return `true` if the backend sends data to an external API (e.g. Claude).
pub fn is_external_backend(backend_name: &str) -> bool {
    backend_name == "Claude" || backend_name == "Claude CLI" || backend_name == "Agent"
}

/// Return `true` if the one-time privacy notice must be shown before this query.
pub fn privacy_notice_needed(why_cfg: &LlmConfig, backend_name: &str) -> bool {
    is_external_backend(backend_name) && why_cfg.privacy_notice_shown != Some(true)
}

/// Display the one-time privacy notice to stderr.
fn show_privacy_notice() {
    eprintln!();
    eprintln!("  {} Privacy Notice", "◆".cyan().bold());
    eprintln!("  Your query and project artifacts will be sent to the Claude API (Anthropic).");
    eprintln!(
        "  {} Anthropic privacy policy: https://www.anthropic.com/privacy",
        "→".cyan()
    );
    eprintln!(
        "  {} Set privacy_notice_shown = true in the [llm] section of",
        "○".dimmed()
    );
    eprintln!("     .wai/config.toml to suppress this notice in future.");
    eprintln!();
}

/// Persist `privacy_notice_shown = true` to the project config.
///
/// Best-effort: silently ignores I/O errors so a missing or broken config never
/// blocks the query from proceeding.
pub fn mark_privacy_notice_shown(project_root: &std::path::Path) {
    if let Ok(mut config) = ProjectConfig::load(project_root) {
        // Write to whichever section is present. If only the legacy `[why]`
        // section exists, update it in place so we don't silently migrate the
        // config format. Otherwise use (or create) the canonical `[llm]` section.
        if config.llm.is_none() && config.why.is_some() {
            let why_cfg = config.why.get_or_insert_with(LlmConfig::default);
            why_cfg.privacy_notice_shown = Some(true);
        } else {
            let llm_cfg = config.llm.get_or_insert_with(LlmConfig::default);
            llm_cfg.privacy_notice_shown = Some(true);
        }
        let _ = config.save(project_root);
    }
}

// ── README badge detection (8.7) ─────────────────────────────────────────────

/// Badge markdown snippet to recommend when a project has no wai badge.
pub const WAI_BADGE_MARKDOWN: &str = "[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)";

/// Return `true` if `content` appears to contain a wai badge.
///
/// Matches:
/// - Any line containing `![` (image/badge markdown) AND `wai` (case-insensitive)
/// - Any line containing `img.shields.io` AND `wai` (case-insensitive)
pub fn content_has_wai_badge(content: &str) -> bool {
    let lower = content.to_lowercase();
    for line in lower.lines() {
        let has_badge_syntax = line.contains("![") || line.contains("img.shields.io");
        if has_badge_syntax && line.contains("wai") {
            return true;
        }
    }
    false
}

/// Return `true` when the project's README already has a wai badge, OR when
/// there is no README (so we don't nag users without one).
pub fn readme_has_wai_badge(project_root: &std::path::Path) -> bool {
    let candidates = ["README.md", "README.rst", "README.txt", "README"];
    for name in &candidates {
        let path = project_root.join(name);
        if path.exists() {
            return match std::fs::read_to_string(&path) {
                Ok(content) => content_has_wai_badge(&content),
                Err(_) => true, // can't read → don't nag
            };
        }
    }
    // No README found — don't suggest adding a badge
    true
}

/// Print a badge recommendation footer to stdout.
fn print_badge_footer() {
    println!();
    separator();
    println!();
    println!(
        "  {} No wai badge in README — add one to let others know:",
        "○".dimmed()
    );
    println!();
    println!("  {}", WAI_BADGE_MARKDOWN.dimmed());
    println!();
}

// ── Verbose diagnostics ───────────────────────────────────────────────────────

/// Build the verbose diagnostic lines shown after an LLM call.
///
/// `-v`   → elapsed time
/// `-vv`  → + prompt/response char counts, estimated tokens, estimated cost
/// `-vvv` → + full prompt text
pub fn verbose_stats_lines(
    verbose: u8,
    elapsed_ms: u128,
    prompt: &str,
    response: &str,
    model_id: &str,
) -> Vec<String> {
    if verbose == 0 {
        return vec![];
    }

    let elapsed_s = elapsed_ms as f64 / 1000.0;
    let mut lines = vec![format!("  {} {:.2}s", "○".dimmed(), elapsed_s)];

    if verbose >= 2 {
        let input_chars = prompt.len();
        let output_chars = response.len();
        let input_tokens = input_chars / 4;
        let output_tokens = output_chars / 4;
        lines.push(format!(
            "  {} prompt {} chars (~{} tokens), response {} chars (~{} tokens)",
            "◇".dimmed(),
            input_chars,
            input_tokens,
            output_chars,
            output_tokens,
        ));
        if let Some(cost) = crate::llm::estimate_cost(model_id, input_chars, output_chars) {
            lines.push(format!("  {} ~${:.4} estimated", "◇".dimmed(), cost));
        }
    }

    if verbose >= 3 {
        lines.push(String::new());
        lines.push(format!("  {} Full prompt:", "◇".dimmed()));
        lines.push("  ─────────────────────────────────────────".to_string());
        for line in prompt.lines() {
            lines.push(format!("  {}", line));
        }
        lines.push("  ─────────────────────────────────────────".to_string());
    }

    lines
}

fn print_verbose_stats(
    verbose: u8,
    elapsed_ms: u128,
    prompt: &str,
    response: &str,
    model_id: &str,
) {
    let lines = verbose_stats_lines(verbose, elapsed_ms, prompt, response, model_id);
    if !lines.is_empty() {
        println!();
        for line in lines {
            println!("{}", line);
        }
    }
}

// ── Command entry point ───────────────────────────────────────────────────────

pub fn run(query: String, no_llm: bool, json: bool, verbose: u8) -> Result<()> {
    // Merge local --json with global --json so both `wai why --json` and
    // `wai --json why` produce machine-readable output.
    let json = json || current_context().json;
    let project_root = require_project()?;

    if no_llm {
        return super::search::run(query, None, None, false, None, Vec::new(), false, 0);
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
        .map(|c| c.llm_config().into_owned())
        .unwrap_or_default();

    let mode = fallback_mode(&why_cfg);

    // Detect backend; fall back to search (or error) if none available
    let backend: Box<dyn crate::llm::LlmClient> = match detect_backend(&why_cfg) {
        Some(b) => b,
        None => {
            if mode == FallbackMode::Error {
                return Err(WaiError::LlmNotAvailable.into());
            }
            if ollama_binary_exists() {
                // Ollama is installed but the model hasn't been pulled yet
                let model = why_cfg.model.as_deref().unwrap_or("llama3.1:8b");
                eprintln!(
                    "  {} Ollama is installed but model '{}' is not available. Falling back to search.",
                    "⚠".yellow(),
                    model
                );
                eprintln!(
                    "  {} Run: {}",
                    "○".dimmed(),
                    format!("ollama pull {}", model).bold()
                );
            } else if claude_binary_exists() {
                // claude binary found but couldn't be used (shouldn't normally happen)
                eprintln!(
                    "  {} No LLM available. Falling back to `wai search`.",
                    "⚠".yellow()
                );
                eprintln!(
                    "  {} Set ANTHROPIC_API_KEY or configure `[llm] llm = \"claude-cli\"` in .wai/config.toml",
                    "○".dimmed()
                );
            } else {
                eprintln!(
                    "  {} No LLM available. Falling back to `wai search`.",
                    "⚠".yellow()
                );
                eprintln!(
                    "  {} Install Claude Code, set ANTHROPIC_API_KEY, or install Ollama.",
                    "○".dimmed()
                );
            }
            if let Some(hint) = explicit_backend_agent_hint(&why_cfg) {
                eprintln!("  {} {}", "→".cyan(), hint);
            }
            return super::search::run(query, None, None, false, None, Vec::new(), false, 0);
        }
    };

    // 6.6: Show one-time privacy notice for external APIs (e.g. Claude)
    if privacy_notice_needed(&why_cfg, backend.name()) {
        show_privacy_notice();
        mark_privacy_notice_shown(&project_root);
    }

    // Build prompt and call the LLM
    let prompt = build_prompt(&ctx);

    if !json {
        println!();
        println!("  {} {}", "◆".cyan(), query.bold());
        println!("  {} Querying {} …", "○".dimmed(), backend.name());
    }

    let start = std::time::Instant::now();
    let raw_response = match backend.complete(&prompt) {
        Ok(r) if r == AGENT_SENTINEL => {
            // Agent backend wrote context to stdout; no further output needed.
            println!("  {} Context sent to your agent", "○".dimmed());
            return Ok(());
        }
        Ok(r) => r,
        Err(e) => {
            if mode == FallbackMode::Error {
                let wai_err = match &e {
                    LlmError::InvalidApiKey => WaiError::LlmInvalidApiKey,
                    LlmError::RateLimit => WaiError::LlmRateLimit,
                    LlmError::NetworkError(m) => WaiError::LlmNetworkError { message: m.clone() },
                    LlmError::ModelNotFound(m) => WaiError::LlmModelNotFound { model: m.clone() },
                    LlmError::Other(m) => WaiError::LlmNetworkError { message: m.clone() },
                };
                return Err(wai_err.into());
            }
            let (msg, hint) = llm_error_hint(&e);
            eprintln!("  {} {}. Falling back to search.", "⚠".yellow(), msg);
            if let Some(h) = hint {
                eprintln!("  {} {}", "○".dimmed(), h);
            }
            if let Some(h) = explicit_backend_agent_hint(&why_cfg) {
                eprintln!("  {} {}", "→".cyan(), h);
            }
            return super::search::run(query, None, None, false, None, Vec::new(), false, 0);
        }
    };
    let elapsed_ms = start.elapsed().as_millis();

    let parsed = parse_response(&raw_response);

    if json {
        println!("{}", format_json(&parsed, &query));
    } else {
        format_terminal(&parsed, &query);
        // 9.1: Show verbose diagnostics (timing, token estimates, cost, full prompt)
        if verbose > 0 {
            print_verbose_stats(
                verbose,
                elapsed_ms,
                &prompt,
                &raw_response,
                backend.model_id(),
            );
        }
        // 8.7: Suggest adding a badge if README has none
        if !readme_has_wai_badge(&project_root) {
            print_badge_footer();
        }
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

    // ── llm_error_hint ──

    #[test]
    fn llm_error_hint_rate_limit_mentions_wait_and_ollama() {
        let (msg, hint) = llm_error_hint(&LlmError::RateLimit);
        assert!(msg.to_lowercase().contains("rate"));
        let h = hint.expect("hint should be present");
        assert!(h.contains("60") || h.to_lowercase().contains("ollama"));
    }

    #[test]
    fn llm_error_hint_model_not_found_includes_pull_command() {
        let (msg, hint) = llm_error_hint(&LlmError::ModelNotFound("llama3.1:8b".to_string()));
        assert!(msg.contains("llama3.1:8b"));
        let h = hint.expect("hint should be present");
        assert!(h.contains("ollama pull"));
        assert!(h.contains("llama3.1:8b"));
    }

    #[test]
    fn llm_error_hint_invalid_api_key_mentions_api_key() {
        let (msg, hint) = llm_error_hint(&LlmError::InvalidApiKey);
        assert!(!msg.is_empty());
        let h = hint.expect("hint should be present");
        // Help text should mention how to configure the key
        assert!(h.to_uppercase().contains("ANTHROPIC_API_KEY") || h.contains("api_key"));
    }

    #[test]
    fn llm_error_hint_network_error_preserves_inner_message() {
        let (msg, hint) = llm_error_hint(&LlmError::NetworkError("timeout".to_string()));
        assert!(msg.contains("timeout"));
        assert!(hint.is_some());
    }

    #[test]
    fn llm_error_hint_other_returns_message_and_no_hint() {
        let (msg, hint) = llm_error_hint(&LlmError::Other("unexpected thing".to_string()));
        assert_eq!(msg, "unexpected thing");
        assert!(hint.is_none());
    }

    // ── fallback_mode (6.4) ──

    #[test]
    fn fallback_mode_default_is_search() {
        let cfg = LlmConfig::default();
        assert_eq!(fallback_mode(&cfg), FallbackMode::Search);
    }

    #[test]
    fn fallback_mode_explicit_search() {
        let cfg = LlmConfig {
            fallback: Some("search".to_string()),
            ..Default::default()
        };
        assert_eq!(fallback_mode(&cfg), FallbackMode::Search);
    }

    #[test]
    fn fallback_mode_explicit_error() {
        let cfg = LlmConfig {
            fallback: Some("error".to_string()),
            ..Default::default()
        };
        assert_eq!(fallback_mode(&cfg), FallbackMode::Error);
    }

    #[test]
    fn fallback_mode_unknown_value_defaults_to_search() {
        let cfg = LlmConfig {
            fallback: Some("unknown".to_string()),
            ..Default::default()
        };
        assert_eq!(fallback_mode(&cfg), FallbackMode::Search);
    }

    // ── explicit_backend_agent_hint (7.1) ──

    #[test]
    fn explicit_backend_failure_in_claude_code_suggests_agent_mode() {
        unsafe { std::env::set_var("CLAUDECODE", "1") };
        let cfg = LlmConfig {
            llm: Some("claude".to_string()),
            ..Default::default()
        };
        let hint = explicit_backend_agent_hint(&cfg);
        unsafe { std::env::remove_var("CLAUDECODE") };
        let h = hint.expect("hint should be present for explicit claude + CLAUDECODE");
        assert!(h.contains("agent"), "hint should mention agent mode");
    }

    #[test]
    fn explicit_ollama_failure_in_claude_code_suggests_agent_mode() {
        unsafe { std::env::set_var("CLAUDECODE", "1") };
        let cfg = LlmConfig {
            llm: Some("ollama".to_string()),
            ..Default::default()
        };
        let hint = explicit_backend_agent_hint(&cfg);
        unsafe { std::env::remove_var("CLAUDECODE") };
        let h = hint.expect("hint should be present for explicit ollama + CLAUDECODE");
        assert!(h.contains("agent"), "hint should mention agent mode");
    }

    #[test]
    fn auto_detect_backend_in_claude_code_no_hint() {
        unsafe { std::env::set_var("CLAUDECODE", "1") };
        let cfg = LlmConfig::default(); // llm = None → auto-detect
        let hint = explicit_backend_agent_hint(&cfg);
        unsafe { std::env::remove_var("CLAUDECODE") };
        assert!(
            hint.is_none(),
            "no hint for auto-detect config (agent is already preferred)"
        );
    }

    // ── is_external_backend / privacy_notice_needed (6.5 / 6.6) ──

    #[test]
    fn claude_backend_is_external() {
        assert!(is_external_backend("Claude"));
    }

    #[test]
    fn ollama_backend_is_not_external() {
        assert!(!is_external_backend("Ollama"));
    }

    #[test]
    fn unknown_backend_is_not_external() {
        assert!(!is_external_backend("mock"));
    }

    #[test]
    fn privacy_notice_needed_when_not_shown_and_claude() {
        let cfg = LlmConfig::default();
        assert!(privacy_notice_needed(&cfg, "Claude"));
    }

    #[test]
    fn privacy_notice_not_needed_when_shown_true() {
        let cfg = LlmConfig {
            privacy_notice_shown: Some(true),
            ..Default::default()
        };
        assert!(!privacy_notice_needed(&cfg, "Claude"));
    }

    #[test]
    fn privacy_notice_still_needed_when_shown_false() {
        let cfg = LlmConfig {
            privacy_notice_shown: Some(false),
            ..Default::default()
        };
        assert!(privacy_notice_needed(&cfg, "Claude"));
    }

    #[test]
    fn privacy_notice_not_needed_for_ollama() {
        let cfg = LlmConfig::default();
        assert!(!privacy_notice_needed(&cfg, "Ollama"));
    }

    #[test]
    fn agent_backend_is_external() {
        assert!(is_external_backend("Agent"));
    }

    #[test]
    fn privacy_notice_needed_for_agent_when_not_shown() {
        let cfg = LlmConfig::default();
        assert!(privacy_notice_needed(&cfg, "Agent"));
    }

    #[test]
    fn privacy_notice_not_needed_for_agent_when_shown() {
        let cfg = LlmConfig {
            privacy_notice_shown: Some(true),
            ..Default::default()
        };
        assert!(!privacy_notice_needed(&cfg, "Agent"));
    }

    // ── gather_context end-to-end (7.1 extended) ──

    #[test]
    fn gather_context_populates_artifacts_from_tmpdir() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);
        let ctx = gather_context(tmp.path(), "why was this designed this way?");
        assert!(!ctx.artifacts.is_empty(), "should find artifacts in tmpdir");
        assert!(
            !ctx.is_file_query,
            "natural language query is not a file query"
        );
        assert!(ctx.git_context.is_none(), "no git context for NL queries");
        assert!(!ctx.is_empty());
    }

    #[test]
    fn gather_context_marks_file_query_for_existing_path() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);
        // Create a real file so Path::exists() returns true
        let src_file = tmp.path().join("src").join("main.rs");
        fs::create_dir_all(src_file.parent().unwrap()).unwrap();
        fs::write(&src_file, "fn main() {}").unwrap();
        let ctx = gather_context(tmp.path(), src_file.to_str().unwrap());
        assert!(
            ctx.is_file_query,
            "absolute path to existing file is a file query"
        );
    }

    // ── full pipeline integration test (7.4) ──

    #[test]
    fn full_pipeline_gather_prompt_parse_and_format_json() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);

        // Gather context — should find the 3 artifacts from setup_wai_project
        let ctx = gather_context(tmp.path(), "why was this designed this way?");
        assert!(!ctx.artifacts.is_empty());

        // Build prompt — must contain the query and artifact content
        let prompt = build_prompt(&ctx);
        assert!(prompt.contains("why was this designed this way?"));
        assert!(prompt.contains("research content"));

        // Simulate a fixed LLM response (no real LLM call)
        let mock_response = "## Answer\n\
The design was chosen for simplicity and maintainability.\n\
## Relevant Artifacts\n\
- `.wai/projects/myproj/research/2024-01-01-notes.md` (High) — core rationale\n\
## Decision Chain\n\
Research → Design → Implementation\n\
## Suggestions\n\
- Review the research artifact for full context\n\
- Consider adding more design notes\n";

        // Parse the mock response
        let parsed = parse_response(mock_response);
        assert_eq!(
            parsed.answer,
            "The design was chosen for simplicity and maintainability."
        );
        assert_eq!(parsed.relevant_artifacts.len(), 1);
        assert_eq!(
            parsed.relevant_artifacts[0].relevance,
            Some(Relevance::High)
        );
        assert_eq!(parsed.decision_chain, "Research → Design → Implementation");
        assert_eq!(parsed.suggestions.len(), 2);

        // Format as JSON — verify all required fields are present and correct
        let json = format_json(&parsed, "why was this designed this way?");
        let v: serde_json::Value = serde_json::from_str(&json).expect("output must be valid JSON");
        assert_eq!(v["query"], "why was this designed this way?");
        assert!(v["answer"].as_str().unwrap().contains("simplicity"));
        assert_eq!(v["relevant_artifacts"].as_array().unwrap().len(), 1);
        assert_eq!(
            v["relevant_artifacts"][0]["relevance"].as_str().unwrap(),
            "High"
        );
        assert_eq!(v["suggestions"].as_array().unwrap().len(), 2);
    }

    // ── mark_privacy_notice_shown ──

    #[test]
    fn mark_privacy_notice_shown_updates_config() {
        let tmp = TempDir::new().unwrap();
        let wai_dir_path = tmp.path().join(".wai");
        fs::create_dir_all(&wai_dir_path).unwrap();
        let config_content = "[project]\nname = \"test\"\nversion = \"\"\ndescription = \"\"\n";
        fs::write(wai_dir_path.join("config.toml"), config_content).unwrap();

        mark_privacy_notice_shown(tmp.path());

        let config = crate::config::ProjectConfig::load(tmp.path()).unwrap();
        // mark_privacy_notice_shown writes to [llm] on fresh configs (no legacy [why]).
        assert_eq!(
            config.llm_config().privacy_notice_shown,
            Some(true)
        );
    }

    #[test]
    fn mark_privacy_notice_shown_no_panic_without_config() {
        let tmp = TempDir::new().unwrap();
        // No config file — should not panic
        mark_privacy_notice_shown(tmp.path());
    }

    // ── content_has_wai_badge / readme_has_wai_badge (8.7) ──

    #[test]
    fn badge_markdown_detected_in_content() {
        let content = "# My Project\n[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)\n";
        assert!(content_has_wai_badge(content));
    }

    #[test]
    fn shields_io_url_with_wai_detected() {
        let content = "![wai badge](https://img.shields.io/badge/wai-tracked-blue)\n";
        assert!(content_has_wai_badge(content));
    }

    #[test]
    fn content_without_wai_badge_returns_false() {
        let content = "# My Project\n\nSome description without any badge.\n";
        assert!(!content_has_wai_badge(content));
    }

    #[test]
    fn badge_detection_case_insensitive() {
        let content = "[![WAI](https://img.shields.io/badge/WAI-blue)](https://example.com)\n";
        assert!(content_has_wai_badge(content));
    }

    #[test]
    fn shields_io_without_wai_not_detected() {
        let content = "![ci](https://img.shields.io/badge/build-passing-green)\n";
        assert!(!content_has_wai_badge(content));
    }

    #[test]
    fn readme_has_wai_badge_returns_true_when_no_readme() {
        let tmp = TempDir::new().unwrap();
        // No README — should not nag
        assert!(readme_has_wai_badge(tmp.path()));
    }

    #[test]
    fn readme_has_wai_badge_detects_badge_in_readme() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("README.md"),
            "# Proj\n[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)\n",
        )
        .unwrap();
        assert!(readme_has_wai_badge(tmp.path()));
    }

    #[test]
    fn readme_has_wai_badge_returns_false_when_badge_missing() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("README.md"),
            "# My Project\n\nNo badge here.\n",
        )
        .unwrap();
        assert!(!readme_has_wai_badge(tmp.path()));
    }

    // ── verbose_stats_lines ──

    #[test]
    fn verbose_zero_returns_empty() {
        let lines = verbose_stats_lines(0, 1500, "prompt text", "response text", "mock");
        assert!(lines.is_empty());
    }

    #[test]
    fn verbose_one_returns_timing_only() {
        let lines = verbose_stats_lines(1, 2500, "prompt", "response", "mock");
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains("2.50s"),
            "expected timing, got: {}",
            lines[0]
        );
    }

    #[test]
    fn verbose_two_returns_timing_and_token_counts() {
        let prompt = "a".repeat(400);
        let response = "b".repeat(100);
        let lines = verbose_stats_lines(2, 1000, &prompt, &response, "mock");
        // line 0: timing
        assert!(lines[0].contains("1.00s"));
        // line 1: char/token counts (no cost for "mock" model)
        assert!(lines[1].contains("400 chars"));
        assert!(lines[1].contains("100 chars"));
        assert!(lines[1].contains("100 tokens")); // 400/4=100 input tokens
        assert!(lines[1].contains("25 tokens")); // 100/4=25 output tokens
        // no cost line for unknown model
        assert!(!lines.iter().any(|l| l.contains("estimated")));
    }

    #[test]
    fn verbose_two_includes_cost_for_claude_model() {
        let prompt = "a".repeat(4000);
        let response = "b".repeat(400);
        let lines = verbose_stats_lines(2, 1000, &prompt, &response, "claude-haiku-3-5-20251001");
        assert!(lines.iter().any(|l| l.contains("estimated")));
    }

    #[test]
    fn verbose_three_includes_full_prompt() {
        let prompt = "line one\nline two";
        let lines = verbose_stats_lines(3, 500, prompt, "resp", "mock");
        let joined = lines.join("\n");
        assert!(joined.contains("Full prompt"));
        assert!(joined.contains("line one"));
        assert!(joined.contains("line two"));
    }
}
