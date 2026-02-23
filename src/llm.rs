/// LLM backend abstraction for the `wai why` reasoning oracle.
///
/// Implementations are optional at compile-time. The command degrades gracefully
/// to `wai search` when no backend is available.
pub trait LlmClient: Send + Sync {
    /// Send a prompt and return the LLM's response text.
    fn complete(&self, prompt: &str) -> Result<String, LlmError>;

    /// Check whether this backend is currently usable (key present, binary found, etc.).
    fn is_available(&self) -> bool;

    /// Human-readable name for display and diagnostics (e.g. "Claude (haiku)").
    fn name(&self) -> &str;
}

/// Errors specific to LLM backend operations.
///
/// These are mapped to `WaiError::Llm*` variants for miette diagnostics at the
/// call site in `src/commands/why.rs`.
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

/// Resolve a short model alias to the canonical model ID.
///
/// Aliases let users write `model = "haiku"` in config instead of
/// `model = "claude-haiku-3-5-20251001"`.
pub fn resolve_model_alias(alias: &str) -> &str {
    match alias {
        "haiku" => "claude-haiku-3-5-20251001",
        "sonnet" => "claude-sonnet-4-5",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- resolve_model_alias ---

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
        assert_eq!(
            resolve_model_alias("claude-opus-4-5"),
            "claude-opus-4-5"
        );
    }

    // --- LlmError display ---

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

    // --- LlmClient trait: mock implementation ---

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
    }

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
}
