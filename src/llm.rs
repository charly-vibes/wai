use serde::Deserialize;

use crate::config::WhyConfig;

#[cfg(test)]
use serial_test::serial;

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
        "sonnet" => "claude-sonnet-4-5",
        other => other,
    }
}

// ── Claude client ─────────────────────────────────────────────────────────────

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_API_VERSION: &str = "2023-06-01";
const CLAUDE_DEFAULT_MODEL: &str = "claude-haiku-3-5-20251001";
const CLAUDE_MAX_TOKENS: u32 = 2048;

pub struct ClaudeClient {
    api_key: String,
    model: String,
}

impl ClaudeClient {
    /// Create a new Claude client.
    ///
    /// `api_key` must be set; `model` is the resolved canonical model ID.
    pub fn new(api_key: String, model: String) -> Self {
        ClaudeClient { api_key, model }
    }

    /// Build from `WhyConfig`, falling back to `ANTHROPIC_API_KEY` env var.
    ///
    /// Returns `None` if no API key is available.
    pub fn from_config(cfg: &WhyConfig) -> Option<Self> {
        let api_key = cfg
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())?;

        let model = cfg
            .model
            .as_deref()
            .map(resolve_model_alias)
            .unwrap_or(CLAUDE_DEFAULT_MODEL)
            .to_string();

        Some(ClaudeClient::new(api_key, model))
    }
}

/// Minimal deserialisation of the Claude Messages API response.
#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContentBlock>,
    #[serde(default)]
    error: Option<ClaudeApiError>,
}

#[derive(Deserialize)]
struct ClaudeContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeApiError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

impl LlmClient for ClaudeClient {
    fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": CLAUDE_MAX_TOKENS,
            "messages": [{"role": "user", "content": prompt}]
        });

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", CLAUDE_API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let status = resp.status();
        let text = resp
            .text()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        match status.as_u16() {
            200 => {}
            401 => return Err(LlmError::InvalidApiKey),
            429 => return Err(LlmError::RateLimit),
            404 => {
                return Err(LlmError::ModelNotFound(self.model.clone()));
            }
            other => {
                return Err(LlmError::Other(format!("HTTP {}: {}", other, text)));
            }
        }

        let parsed: ClaudeResponse = serde_json::from_str(&text)
            .map_err(|e| LlmError::Other(format!("Failed to parse response: {}", e)))?;

        if let Some(err) = parsed.error {
            return Err(match err.error_type.as_str() {
                "authentication_error" => LlmError::InvalidApiKey,
                "rate_limit_error" => LlmError::RateLimit,
                _ => LlmError::Other(err.message),
            });
        }

        parsed
            .content
            .into_iter()
            .find(|b| b.block_type == "text")
            .and_then(|b| b.text)
            .ok_or_else(|| LlmError::Other("Empty response from Claude".to_string()))
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    fn name(&self) -> &str {
        "Claude"
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}

// ── Claude CLI client ─────────────────────────────────────────────────────────

/// Backend that delegates to the `claude` binary (Claude Code CLI) in print mode.
///
/// Requires no API key configuration — claude manages its own auth. Ideal for
/// users who already have Claude Code installed.
pub struct ClaudeCliClient;

impl ClaudeCliClient {
    pub fn from_config(_cfg: &WhyConfig) -> Self {
        ClaudeCliClient
    }
}

/// Return `true` when running inside a Claude Code agent session.
///
/// Claude Code sets `CLAUDECODE` to a non-empty value when an agent is active.
/// An empty string — used by [`ClaudeCliClient`] to bypass the nested-session
/// guard — is treated as false.
pub fn in_agent_session() -> bool {
    std::env::var("CLAUDECODE")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

/// Return `true` if the `claude` binary is on PATH.
pub fn claude_binary_exists() -> bool {
    std::process::Command::new("claude")
        .arg("--version")
        .env("CLAUDECODE", "") // bypass nested-session guard
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some()
}

impl LlmClient for ClaudeCliClient {
    fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        use std::io::Write;

        let mut child = std::process::Command::new("claude")
            .args(["-p", "--tools", "", "--no-session-persistence"])
            .env("CLAUDECODE", "") // bypass nested-session guard
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| LlmError::NetworkError(format!("Failed to spawn claude: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| LlmError::Other(e.to_string()))?;
        }

        let out = child
            .wait_with_output()
            .map_err(|e| LlmError::Other(e.to_string()))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            return Err(LlmError::Other(stderr));
        }

        let text = String::from_utf8_lossy(&out.stdout).to_string();
        if text.trim().is_empty() {
            return Err(LlmError::Other(
                "Empty response from claude CLI".to_string(),
            ));
        }

        Ok(text)
    }

    fn is_available(&self) -> bool {
        claude_binary_exists()
    }

    fn name(&self) -> &str {
        "Claude CLI"
    }

    fn model_id(&self) -> &str {
        "claude-cli"
    }
}

// ── Ollama client ─────────────────────────────────────────────────────────────

const OLLAMA_DEFAULT_MODEL: &str = "llama3.1:8b";

pub struct OllamaClient {
    model: String,
}

impl OllamaClient {
    pub fn new(model: String) -> Self {
        OllamaClient { model }
    }

    /// Build from `WhyConfig`.
    pub fn from_config(cfg: &WhyConfig) -> Self {
        let model = cfg
            .model
            .clone()
            .unwrap_or_else(|| OLLAMA_DEFAULT_MODEL.to_string());
        OllamaClient::new(model)
    }

    /// Return true if `ollama` binary is on PATH.
    fn ollama_binary_exists() -> bool {
        crate::llm::ollama_binary_exists()
    }

    /// Return true if the configured model is available locally.
    fn model_available(&self) -> bool {
        let output = std::process::Command::new("ollama")
            .args(["list"])
            .output()
            .ok();

        match output {
            Some(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout).to_lowercase();
                text.contains(&self.model.to_lowercase())
            }
            _ => false,
        }
    }
}

/// Return `true` if the `ollama` binary is on PATH.
pub fn ollama_binary_exists() -> bool {
    std::process::Command::new("ollama")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some()
}

impl LlmClient for OllamaClient {
    fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        // Use `ollama run <model>` via stdin
        let mut child = std::process::Command::new("ollama")
            .args(["run", &self.model])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| LlmError::NetworkError(format!("Failed to spawn ollama: {}", e)))?;

        use std::io::Write;
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| LlmError::Other(e.to_string()))?;
        }

        let out = child
            .wait_with_output()
            .map_err(|e| LlmError::Other(e.to_string()))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if stderr.contains("not found") || stderr.contains("pull") {
                return Err(LlmError::ModelNotFound(self.model.clone()));
            }
            return Err(LlmError::Other(stderr));
        }

        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn is_available(&self) -> bool {
        Self::ollama_binary_exists() && self.model_available()
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}

// ── Backend selection ─────────────────────────────────────────────────────────

/// Select the best available LLM backend given `WhyConfig`.
///
/// Priority:
/// 1. Explicit `llm = "claude"` / `"claude-cli"` / `"ollama"` in config
/// 2. Auto-detect: Claude API → Claude CLI → Ollama
/// 3. `None` → caller should fall back to `wai search`
pub fn detect_backend(cfg: &WhyConfig) -> Option<Box<dyn LlmClient>> {
    match cfg.llm.as_deref() {
        Some("claude") => {
            let client = ClaudeClient::from_config(cfg)?;
            Some(Box::new(client))
        }
        Some("claude-cli") => {
            let client = ClaudeCliClient::from_config(cfg);
            if client.is_available() {
                Some(Box::new(client))
            } else {
                None
            }
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
        _ => {
            // 1. Claude API (direct, fastest)
            if let Some(client) = ClaudeClient::from_config(cfg) {
                return Some(Box::new(client));
            }
            // 2. Claude CLI (zero-config for Claude Code users)
            let cli = ClaudeCliClient::from_config(cfg);
            if cli.is_available() {
                return Some(Box::new(cli));
            }
            // 3. Ollama (local fallback)
            let ollama = OllamaClient::from_config(cfg);
            if ollama.is_available() {
                return Some(Box::new(ollama));
            }
            None
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── in_agent_session ──

    #[test]
    #[serial]
    fn claudecode_set_to_one_returns_true() {
        unsafe { std::env::set_var("CLAUDECODE", "1") };
        assert!(in_agent_session());
        unsafe { std::env::remove_var("CLAUDECODE") };
    }

    #[test]
    #[serial]
    fn claudecode_empty_string_returns_false() {
        unsafe { std::env::set_var("CLAUDECODE", "") };
        assert!(!in_agent_session());
        unsafe { std::env::remove_var("CLAUDECODE") };
    }

    #[test]
    #[serial]
    fn claudecode_unset_returns_false() {
        unsafe { std::env::remove_var("CLAUDECODE") };
        assert!(!in_agent_session());
    }

    // ── resolve_model_alias ──

    #[test]
    fn alias_haiku_resolves_to_canonical_id() {
        assert_eq!(resolve_model_alias("haiku"), "claude-haiku-3-5-20251001");
    }

    #[test]
    fn alias_sonnet_resolves_to_canonical_id() {
        assert_eq!(resolve_model_alias("sonnet"), "claude-sonnet-4-5");
    }

    #[test]
    fn unknown_alias_passes_through_unchanged() {
        assert_eq!(resolve_model_alias("llama3.1:8b"), "llama3.1:8b");
        assert_eq!(resolve_model_alias("claude-opus-4-5"), "claude-opus-4-5");
    }

    // ── LlmError display ──

    #[test]
    fn llm_error_display_is_human_readable() {
        assert_eq!(
            LlmError::InvalidApiKey.to_string(),
            "API key is invalid or missing"
        );
        assert_eq!(LlmError::RateLimit.to_string(), "Rate limit exceeded");
        assert_eq!(
            LlmError::NetworkError("timeout".into()).to_string(),
            "Network error: timeout"
        );
        assert_eq!(
            LlmError::ModelNotFound("llama99".into()).to_string(),
            "Model not found: llama99"
        );
        assert_eq!(
            LlmError::Other("unexpected".into()).to_string(),
            "unexpected"
        );
    }

    // ── MockLlm ──

    struct MockLlm {
        available: bool,
        response: Result<String, LlmError>,
    }

    impl MockLlm {
        fn available(response: &str) -> Self {
            MockLlm {
                available: true,
                response: Ok(response.to_string()),
            }
        }

        fn unavailable() -> Self {
            MockLlm {
                available: false,
                response: Err(LlmError::InvalidApiKey),
            }
        }
    }

    impl LlmClient for MockLlm {
        fn complete(&self, _prompt: &str) -> Result<String, LlmError> {
            match &self.response {
                Ok(s) => Ok(s.clone()),
                Err(LlmError::InvalidApiKey) => Err(LlmError::InvalidApiKey),
                Err(LlmError::RateLimit) => Err(LlmError::RateLimit),
                Err(LlmError::NetworkError(m)) => Err(LlmError::NetworkError(m.clone())),
                Err(LlmError::ModelNotFound(m)) => Err(LlmError::ModelNotFound(m.clone())),
                Err(LlmError::Other(m)) => Err(LlmError::Other(m.clone())),
            }
        }

        fn is_available(&self) -> bool {
            self.available
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn model_id(&self) -> &str {
            "mock-model"
        }
    }

    // ── estimate_cost ──

    #[test]
    fn haiku_model_cost_is_estimated() {
        let cost = estimate_cost("claude-haiku-3-5-20251001", 4000, 400).unwrap();
        // 4000 chars / 4 = 1000 input tokens @ $0.80/M → $0.0008
        // 400 chars / 4 = 100 output tokens @ $4.00/M → $0.0004
        assert!((cost - 0.0012).abs() < 1e-10);
    }

    #[test]
    fn sonnet_model_cost_is_estimated() {
        let cost = estimate_cost("claude-sonnet-4-5", 4000, 400).unwrap();
        // 1000 input tokens @ $3.00/M → $0.003
        // 100 output tokens @ $15.00/M → $0.0015
        assert!((cost - 0.0045).abs() < 1e-10);
    }

    #[test]
    fn opus_model_cost_is_estimated() {
        let cost = estimate_cost("claude-opus-4-5", 4000, 400).unwrap();
        // 1000 input tokens @ $15.00/M → $0.015
        // 100 output tokens @ $75.00/M → $0.0075
        assert!((cost - 0.0225).abs() < 1e-10);
    }

    #[test]
    fn unknown_model_returns_none() {
        assert!(estimate_cost("llama3.1:8b", 4000, 400).is_none());
        assert!(estimate_cost("gpt-4", 4000, 400).is_none());
    }

    #[test]
    fn zero_chars_returns_zero_cost() {
        let cost = estimate_cost("claude-haiku-3-5-20251001", 0, 0).unwrap();
        assert_eq!(cost, 0.0);
    }

    // ── MockLlm ──

    #[test]
    fn available_client_returns_response() {
        let llm = MockLlm::available("the answer is 42");
        assert!(llm.is_available());
        assert_eq!(llm.complete("any prompt").unwrap(), "the answer is 42");
    }

    #[test]
    fn unavailable_client_reports_not_available() {
        let llm = MockLlm::unavailable();
        assert!(!llm.is_available());
    }

    // ── detect_backend ──

    #[test]
    #[serial]
    fn no_config_no_env_returns_none() {
        unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
        let cfg = WhyConfig::default();
        let _ = detect_backend(&cfg); // must not panic
    }

    #[test]
    fn explicit_claude_with_key_returns_claude_client() {
        let cfg = WhyConfig {
            llm: Some("claude".to_string()),
            api_key: Some("sk-test-key".to_string()),
            ..Default::default()
        };
        let backend = detect_backend(&cfg);
        assert!(backend.is_some());
        assert_eq!(backend.unwrap().name(), "Claude");
    }

    #[test]
    #[serial]
    fn explicit_claude_without_key_returns_none() {
        unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
        let cfg = WhyConfig {
            llm: Some("claude".to_string()),
            api_key: None,
            ..Default::default()
        };
        assert!(detect_backend(&cfg).is_none());
    }

    #[test]
    #[serial]
    fn env_var_api_key_enables_claude() {
        unsafe { std::env::set_var("ANTHROPIC_API_KEY", "sk-env-key") };
        let cfg = WhyConfig::default();
        let backend = detect_backend(&cfg);
        assert!(backend.is_some());
        assert_eq!(backend.unwrap().name(), "Claude");
        unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
    }

    // ── ClaudeClient ──

    #[test]
    fn claude_client_is_available_when_key_non_empty() {
        let c = ClaudeClient::new("sk-test".to_string(), "haiku".to_string());
        assert!(c.is_available());
    }

    #[test]
    fn claude_client_not_available_when_key_empty() {
        let c = ClaudeClient::new(String::new(), "haiku".to_string());
        assert!(!c.is_available());
    }

    #[test]
    fn claude_from_config_uses_api_key_field() {
        let cfg = WhyConfig {
            api_key: Some("sk-cfg".to_string()),
            ..Default::default()
        };
        let client = ClaudeClient::from_config(&cfg).unwrap();
        assert!(client.is_available());
    }

    #[test]
    #[serial]
    fn claude_from_config_falls_back_to_env_var() {
        unsafe { std::env::set_var("ANTHROPIC_API_KEY", "sk-env") };
        let cfg = WhyConfig::default();
        let client = ClaudeClient::from_config(&cfg).unwrap();
        assert!(client.is_available());
        unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
    }

    #[test]
    #[serial]
    fn claude_from_config_returns_none_without_key() {
        unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
        let cfg = WhyConfig::default();
        assert!(ClaudeClient::from_config(&cfg).is_none());
    }

    #[test]
    fn claude_model_alias_resolved_in_from_config() {
        let cfg = WhyConfig {
            api_key: Some("key".to_string()),
            model: Some("haiku".to_string()),
            ..Default::default()
        };
        let client = ClaudeClient::from_config(&cfg).unwrap();
        assert_eq!(client.model, "claude-haiku-3-5-20251001");
    }
}
