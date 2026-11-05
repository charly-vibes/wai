use serde::Deserialize;

use super::{LlmClient, LlmError, resolve_model_alias};
use crate::config::LlmConfig;

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

    /// Build from `LlmConfig`, falling back to `ANTHROPIC_API_KEY` env var.
    ///
    /// Returns `None` if no API key is available.
    pub fn from_config(cfg: &LlmConfig) -> Option<Self> {
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
