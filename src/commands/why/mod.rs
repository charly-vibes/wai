pub mod context;
pub mod parsing;

use miette::Result;
use owo_colors::OwoColorize;

use crate::config::{LlmConfig, ProjectConfig};
use crate::context::current_context;
use crate::error::WaiError;
use crate::llm::{
    AGENT_SENTINEL, LlmError, claude_binary_exists, detect_backend, ollama_binary_exists,
};

use super::require_project;

use context::{GatheredContext, gather_context};
use parsing::{ParsedResponse, Relevance, parse_response};

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

// ── Prompt builder ────────────────────────────────────────────────────────────

/// Build the prompt sent to the LLM from gathered context.
pub fn build_prompt(ctx: &GatheredContext) -> String {
    use context::MAX_CONTEXT_CHARS;

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

    // Artifacts — wrap in code fences and escape injection attempts.
    // Each artifact's content is individually capped at MAX_CONTEXT_CHARS so
    // a single massive file cannot blow out the prompt budget.
    if !ctx.artifacts.is_empty() {
        let mut artifact_text = String::from("# Available Artifacts\n");
        let mut chars_used: usize = artifact_text.len();
        for artifact in &ctx.artifacts {
            let raw = escape_artifact(&artifact.content);
            let (body, note) = if chars_used + raw.len() > MAX_CONTEXT_CHARS {
                let budget = MAX_CONTEXT_CHARS.saturating_sub(chars_used);
                if budget == 0 {
                    break;
                }
                let over = raw.len() - budget;
                let truncated = &raw[..budget];
                (
                    truncated.to_string(),
                    format!("\n[... truncated: {} chars over budget ...]", over),
                )
            } else {
                (raw, String::new())
            };
            let block = format!(
                "\n## {} ({})\n```\n{}{}\n```\n",
                artifact.rel_path,
                artifact.kind.label(),
                body,
                note,
            );
            chars_used += block.len();
            artifact_text.push_str(&block);
        }
        parts.push(artifact_text);
    }

    if let Some(ref memories) = ctx.memories {
        parts.push(format!(
            "# Stored Memories (bd)\n\
             These are persistent project memories. Use them as supplementary context \
             when answering the question:\n\n{}\n",
            memories
        ));
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

// ── Explicit-backend agent hint ───────────────────────────────────────────────

/// When an explicit backend fails and the system falls back to search inside a
/// Claude Code session, suggest agent mode as a zero-cost alternative.
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

// ── Fallback mode ─────────────────────────────────────────────────────────────

/// Controls behavior when no LLM is available or an LLM call fails.
#[derive(Debug, PartialEq)]
pub enum FallbackMode {
    /// Gracefully degrade to `wai search` (default).
    Search,
    /// Return an error; do not fall back.
    Error,
}

/// Determine fallback behavior from config.
pub fn fallback_mode(cfg: &LlmConfig) -> FallbackMode {
    match cfg.fallback.as_deref() {
        Some("error") => FallbackMode::Error,
        _ => FallbackMode::Search,
    }
}

// ── Privacy notice ────────────────────────────────────────────────────────────

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
pub fn mark_privacy_notice_shown(project_root: &std::path::Path) {
    if let Ok(mut config) = ProjectConfig::load(project_root) {
        let llm_cfg = config.llm.get_or_insert_with(LlmConfig::default);
        llm_cfg.privacy_notice_shown = Some(true);
        let _ = config.save(project_root);
    }
}

// ── README badge detection ────────────────────────────────────────────────────

/// Badge markdown snippet to recommend when a project has no wai badge.
pub const WAI_BADGE_MARKDOWN: &str = "[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)";

/// Return `true` if `content` appears to contain a wai badge.
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
        return super::search::run(super::search::SearchArgs {
            query,
            type_filter: None,
            project: None,
            use_regex: false,
            limit: None,
            tag_filter: Vec::new(),
            latest: false,
            context_size: 0,
            include_memories: false,
        });
    }

    let ctx = gather_context(&project_root, &query);

    // Warn when no artifacts are present
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
            return super::search::run(super::search::SearchArgs {
                query,
                type_filter: None,
                project: None,
                use_regex: false,
                limit: None,
                tag_filter: Vec::new(),
                latest: false,
                context_size: 0,
                include_memories: false,
            });
        }
    };

    // Show one-time privacy notice for external APIs (e.g. Claude)
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
            return super::search::run(super::search::SearchArgs {
                query,
                type_filter: None,
                project: None,
                use_regex: false,
                limit: None,
                tag_filter: Vec::new(),
                latest: false,
                context_size: 0,
                include_memories: false,
            });
        }
    };
    let elapsed_ms = start.elapsed().as_millis();

    let parsed = parse_response(&raw_response);

    if json {
        println!("{}", format_json(&parsed, &query));
    } else {
        format_terminal(&parsed, &query);
        // Show verbose diagnostics (timing, token estimates, cost, full prompt)
        if verbose > 0 {
            print_verbose_stats(
                verbose,
                elapsed_ms,
                &prompt,
                &raw_response,
                backend.model_id(),
            );
        }
        // Suggest adding a badge if README has none
        if !readme_has_wai_badge(&project_root) {
            print_badge_footer();
        }
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::context::{Artifact, ArtifactKind, GatheredContext, ProjectMeta, gather_context};
    use super::parsing::{ArtifactRef, ParsedResponse, Relevance, parse_response};
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn make_artifact(kind: ArtifactKind, content: &str) -> Artifact {
        Artifact {
            rel_path: format!(".wai/projects/test/{}/file.md", kind.label()),
            kind,
            content: content.to_string(),
            modified: None,
        }
    }

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
        fs::write(research.join(".state"), "ignored").unwrap();
        fs::write(research.join("notes.txt"), "ignored").unwrap();
    }

    fn make_ctx_for_prompt(query: &str, artifacts: Vec<Artifact>) -> GatheredContext {
        GatheredContext {
            query: query.to_string(),
            is_file_query: false,
            artifacts,
            git_context: None,
            meta: ProjectMeta::default(),
            truncated: false,
            memories: None,
        }
    }

    // ── build_prompt ──

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

    #[test]
    fn build_prompt_includes_memories_when_provided() {
        let ctx = GatheredContext {
            query: "why?".to_string(),
            is_file_query: false,
            artifacts: vec![],
            git_context: None,
            meta: ProjectMeta {
                current_phase: None,
                recent_commits: vec![],
            },
            truncated: false,
            memories: Some("- Use bd CLI for integration".to_string()),
        };
        let prompt = build_prompt(&ctx);
        assert!(prompt.contains("Stored Memories (bd)"));
        assert!(prompt.contains("bd CLI"));
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

    // ── fallback_mode ──

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

    // ── explicit_backend_agent_hint ──

    #[test]
    #[serial]
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
    #[serial]
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
    #[serial]
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

    // ── is_external_backend / privacy_notice_needed ──

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
        assert_eq!(config.llm_config().privacy_notice_shown, Some(true));
    }

    #[test]
    fn mark_privacy_notice_shown_no_panic_without_config() {
        let tmp = TempDir::new().unwrap();
        mark_privacy_notice_shown(tmp.path());
    }

    // ── content_has_wai_badge / readme_has_wai_badge ──

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
        assert!(lines[0].contains("1.00s"));
        assert!(lines[1].contains("400 chars"));
        assert!(lines[1].contains("100 chars"));
        assert!(lines[1].contains("100 tokens"));
        assert!(lines[1].contains("25 tokens"));
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

    // ── full pipeline integration test ──

    #[test]
    fn full_pipeline_gather_prompt_parse_and_format_json() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);

        let ctx = gather_context(tmp.path(), "why was this designed this way?");
        assert!(!ctx.artifacts.is_empty());

        let prompt = build_prompt(&ctx);
        assert!(prompt.contains("why was this designed this way?"));
        assert!(prompt.contains("research content"));

        let mock_response = "## Answer\n\
The design was chosen for simplicity and maintainability.\n\
## Relevant Artifacts\n\
- `.wai/projects/myproj/research/2024-01-01-notes.md` (High) — core rationale\n\
## Decision Chain\n\
Research → Design → Implementation\n\
## Suggestions\n\
- Review the research artifact for full context\n\
- Consider adding more design notes\n";

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
}
