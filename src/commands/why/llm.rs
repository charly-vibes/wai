//! LLM backend selection, error hints and privacy notice for `wai why`.

use owo_colors::OwoColorize;

use crate::config::{LlmConfig, ProjectConfig};
use crate::llm::LlmError;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub(crate) fn show_privacy_notice() {
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
