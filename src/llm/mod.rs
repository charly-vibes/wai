use owo_colors::OwoColorize;

use crate::config::LlmConfig;

mod agent;
mod claude_api;
mod claude_cli;
mod ollama;

#[cfg(test)]
mod tests;

pub use agent::{AGENT_SENTINEL, AgentBackend};
pub use claude_api::ClaudeClient;
pub use claude_cli::{ClaudeCliClient, claude_binary_exists, in_agent_session};
pub use ollama::{OllamaClient, ollama_binary_exists};

// ── Trait ─────────────────────────────────────────────────────────────────────

/// LLM backend abstraction for the `wai why` reasoning oracle.
///
/// The command degrades gracefully to `wai search` when no backend is available.
pub trait LlmClient: Send + Sync {
    /// Send a prompt and return the LLM's response text.
    fn complete(&self, prompt: &str) -> Result<String, LlmError>;

    /// Check whether this backend is currently usable (key present, binary found, etc.).
    fn is_available(&self) -> bool;

    /// Human-readable name for display and diagnostics (e.g. "Claude (haiku)").
    fn name(&self) -> &str;

    /// The underlying model identifier used for cost estimation (e.g. "claude-haiku-3-5-20251001").
    fn model_id(&self) -> &str;
}

// ── Errors ────────────────────────────────────────────────────────────────────

/// Errors specific to LLM backend operations.
///
/// Mapped to `WaiError::Llm*` variants for miette diagnostics at the call site.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("API key is invalid or missing")]
    InvalidApiKey,

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("{0}")]
    Other(String),
}

// ── Model aliases ─────────────────────────────────────────────────────────────

/// Resolve a short model alias to the canonical model ID.
///
/// Allows `model = "haiku"` in config instead of the full versioned string.
pub fn resolve_model_alias(alias: &str) -> &str {
    match alias {
        "haiku" => "claude-haiku-3-5-20251001",
        "sonnet" => "claude-sonnet-4-6",
        other => other,
    }
}

// ── Claude client ─────────────────────────────────────────────────────────────

// ── Backend selection ─────────────────────────────────────────────────────────

/// Select the best available LLM backend given `LlmConfig`.
///
/// Priority (explicit config):
///   - `"claude"`     → Claude API (requires `ANTHROPIC_API_KEY`)
///   - `"claude-cli"` → Claude CLI binary; warns if inside a Claude Code session
///   - `"agent"`      → AgentBackend; warns if no agent session detected
///     (detection: WAI_AGENT, CLAUDECODE, or CURSOR_AGENT non-empty)
///   - `"ollama"`     → Ollama local model
///
/// Auto-detect priority:
///   - Inside an agent session (WAI_AGENT / CLAUDECODE / CURSOR_AGENT non-empty): API → Agent → Ollama
///   - Otherwise:                                                                  API → Claude CLI → Ollama
///
/// Returns `None` → caller should fall back to `wai search`.
pub fn detect_backend(cfg: &LlmConfig) -> Option<Box<dyn LlmClient>> {
    match cfg.llm.as_deref() {
        Some("claude") => {
            let client = ClaudeClient::from_config(cfg)?;
            Some(Box::new(client))
        }
        Some("claude-cli") => {
            // 3.9: Warn when inside an agent session — agent mode is more efficient.
            if in_agent_session() {
                eprintln!(
                    "  {} claude-cli selected but a Claude Code session is active.",
                    "⚠".yellow()
                );
                eprintln!(
                    "  {} Consider `llm = \"agent\"` in [llm] for zero-cost queries.",
                    "→".cyan()
                );
            }
            let client = ClaudeCliClient::from_config(cfg);
            if client.is_available() {
                Some(Box::new(client))
            } else {
                None
            }
        }
        Some("agent") => {
            // 3.7: Warn when not inside an agent session.
            if !in_agent_session() {
                eprintln!(
                    "  {} agent mode requires an active Claude Code session; no synthesized answer will be produced.",
                    "⚠".yellow()
                );
            }
            Some(Box::new(AgentBackend))
        }
        Some("ollama") => {
            let client = OllamaClient::from_config(cfg);
            if client.is_available() {
                Some(Box::new(client))
            } else {
                None
            }
        }
        // Auto-detect
        _ => auto_detect(cfg),
    }
}

/// Auto-detect the best available backend when `llm` is not configured.
fn auto_detect(cfg: &LlmConfig) -> Option<Box<dyn LlmClient>> {
    // 1. Inside agent session → Agent (zero-cost, no subprocess)
    if in_agent_session() {
        return Some(Box::new(AgentBackend));
    }
    // 2. Claude API (direct, fastest)
    if let Some(client) = ClaudeClient::from_config(cfg) {
        return Some(Box::new(client));
    }
    // 3. Claude CLI
    let cli = ClaudeCliClient::from_config(cfg);
    if cli.is_available() {
        return Some(Box::new(cli));
    }
    // 4. Ollama (local fallback)
    let ollama = OllamaClient::from_config(cfg);
    if ollama.is_available() {
        return Some(Box::new(ollama));
    }
    None
}

// ── Cost estimation ───────────────────────────────────────────────────────────

/// Estimate the USD cost of a Claude API call from raw character counts.
///
/// Uses ~4 chars per token as a rough heuristic.
/// Returns `None` for unrecognised models (e.g. Ollama local models).
pub fn estimate_cost(model: &str, input_chars: usize, output_chars: usize) -> Option<f64> {
    let (input_per_m, output_per_m) = if model.contains("haiku") {
        (0.80, 4.00)
    } else if model.contains("sonnet") {
        (3.00, 15.00)
    } else if model.contains("opus") {
        (15.00, 75.00)
    } else {
        return None;
    };

    let input_tokens = input_chars as f64 / 4.0;
    let output_tokens = output_chars as f64 / 4.0;
    Some((input_tokens * input_per_m + output_tokens * output_per_m) / 1_000_000.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────
