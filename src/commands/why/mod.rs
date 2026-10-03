pub mod badge;
pub mod context;
pub mod format;
pub mod llm;
pub mod parsing;
pub mod prompt;
mod shared;
pub mod stats;

pub(crate) use badge::print_badge_footer;
pub use badge::{WAI_BADGE_MARKDOWN, content_has_wai_badge, readme_has_wai_badge};
pub use format::{format_json, format_terminal};
pub(crate) use llm::show_privacy_notice;
pub use llm::{
    FallbackMode, explicit_backend_agent_hint, fallback_mode, llm_error_hint,
    mark_privacy_notice_shown, privacy_notice_needed,
};
pub use prompt::build_prompt;
pub(crate) use stats::print_verbose_stats;
#[cfg(test)]
use stats::verbose_stats_lines;

use miette::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use crate::config::{LlmConfig, ProjectConfig};
use crate::context::current_context;
use crate::error::WaiError;
use crate::llm::{
    AGENT_SENTINEL, LlmError, claude_binary_exists, detect_backend, ollama_binary_exists,
};

use super::require_project;

use context::{GatheredContext, gather_context};
use parsing::{ParsedResponse, parse_response};

pub fn run(query: String, no_llm: bool, json: bool, verbose: u8) -> Result<()> {
    // Merge local --json with global --json so both `wai why --json` and
    // `wai --json why` produce machine-readable output.
    let json = json || current_context().json;
    let project_root = require_project()?;

    if no_llm {
        return run_search_fallback(&query);
    }

    let ctx = gather_context(&project_root, &query);
    if ctx.is_empty() {
        print_no_artifacts_hint();
        return Ok(());
    }
    print_truncation_note(&ctx);

    // Load config for LLM backend selection
    let why_cfg = ProjectConfig::load(&project_root)
        .map(|c| c.llm_config().into_owned())
        .unwrap_or_default();

    let mode = fallback_mode(&why_cfg);

    // Detect backend; fall back to search (or error) if none available
    let Some(backend) = resolve_backend(&why_cfg, mode, &query)? else {
        return Ok(()); // search fallback already ran
    };

    show_privacy_notice_if_needed(&why_cfg, backend.name(), &project_root);

    // Build prompt and call the LLM
    let prompt = build_prompt(&ctx);
    print_query_banner(json, &query, backend.name());

    let start = std::time::Instant::now();
    let Some(raw_response) =
        complete_or_fall_back(backend.as_ref(), &prompt, mode, &why_cfg, &query)?
    else {
        // Agent sentinel (context already sent) or search fallback already ran.
        return Ok(());
    };

    let elapsed_ms = start.elapsed().as_millis();

    let parsed = parse_response(&raw_response);
    print_why_output(
        &parsed,
        &query,
        json,
        verbose,
        elapsed_ms,
        &prompt,
        &raw_response,
        backend.model_id(),
        &project_root,
    )?;

    Ok(())
}

/// Call the LLM; on the agent sentinel print a note and return None, on
/// failure either surface the error (FallbackMode::Error) or fall back to
/// `wai search` directly.
fn complete_or_fall_back(
    backend: &dyn crate::llm::LlmClient,
    prompt: &str,
    mode: FallbackMode,
    why_cfg: &LlmConfig,
    query: &str,
) -> Result<Option<String>> {
    match backend.complete(prompt) {
        Ok(r) if r == AGENT_SENTINEL => {
            // Agent backend wrote context to stdout; no further output needed.
            println!("  {} Context sent to your agent", "○".dimmed());
            Ok(None)
        }
        Ok(r) => Ok(Some(r)),
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
            if let Some(h) = explicit_backend_agent_hint(why_cfg) {
                eprintln!("  {} {}", "→".cyan(), h);
            }
            super::search::run(super::search::SearchArgs {
                query: query.to_string(),
                type_filter: None,
                project: None,
                regex: false,
                limit: None,
                tag: Vec::new(),
                latest: false,
                context_size: 0,
                include_memories: false,
            })?;
            Ok(None)
        }
    }
}

/// Plain `wai search` fallback used by --no-llm and LLM-unavailable paths.
fn run_search_fallback(query: &str) -> Result<()> {
    super::search::run(super::search::SearchArgs {
        query: query.to_string(),
        type_filter: None,
        project: None,
        regex: false,
        limit: None,
        tag: Vec::new(),
        latest: false,
        context_size: 0,
        include_memories: false,
    })
}

/// Resolve the LLM backend. `Ok(None)` means no backend was available and the
/// search fallback (or error) already ran/raised.
fn resolve_backend(
    why_cfg: &LlmConfig,
    mode: FallbackMode,
    query: &str,
) -> Result<Option<Box<dyn crate::llm::LlmClient>>> {
    if let Some(b) = detect_backend(why_cfg) {
        return Ok(Some(b));
    }
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
    if let Some(hint) = explicit_backend_agent_hint(why_cfg) {
        eprintln!("  {} {}", "→".cyan(), hint);
    }
    run_search_fallback(query)?;
    Ok(None)
}

fn print_no_artifacts_hint() {
    println!();
    println!("  {} No artifacts found in .wai/", "⚠".yellow());
    println!(
        "  {} Add some first: {}",
        "→".cyan(),
        "wai add research \"your notes\"".bold()
    );
    println!();
}

fn print_truncation_note(ctx: &GatheredContext) {
    if ctx.truncated {
        println!(
            "  {} Context truncated to {} most relevant artifacts",
            "○".dimmed(),
            ctx.artifacts.len()
        );
    }
}

/// Render the answer: JSON envelope, or terminal output with verbose stats
/// and a badge suggestion when applicable.
#[allow(clippy::too_many_arguments)]
fn print_why_output(
    parsed: &ParsedResponse,
    query: &str,
    json: bool,
    verbose: u8,
    elapsed_ms: u128,
    prompt: &str,
    raw_response: &str,
    model_id: &str,
    project_root: &Path,
) -> Result<()> {
    if json {
        println!("{}", format_json(parsed, query));
    } else {
        format_terminal(parsed, query);
        // Show verbose diagnostics (timing, token estimates, cost, full prompt)
        if verbose > 0 {
            print_verbose_stats(verbose, elapsed_ms, prompt, raw_response, model_id);
        }
        // Suggest adding a badge if README has none
        if !readme_has_wai_badge(project_root) {
            print_badge_footer();
        }
    }
    Ok(())
}

/// One-time privacy notice for external APIs (e.g. Claude).
fn show_privacy_notice_if_needed(why_cfg: &LlmConfig, backend_name: &str, project_root: &Path) {
    if privacy_notice_needed(why_cfg, backend_name) {
        show_privacy_notice();
        mark_privacy_notice_shown(project_root);
    }
}

fn print_query_banner(json: bool, query: &str, backend_name: &str) {
    if !json {
        println!();
        println!("  {} {}", "◆".cyan(), query.bold());
        println!("  {} Querying {} …", "○".dimmed(), backend_name);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::context::{Artifact, ArtifactKind, GatheredContext, ProjectMeta, gather_context};
    use super::llm::is_external_backend;
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
