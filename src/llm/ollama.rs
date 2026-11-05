use super::{LlmClient, LlmError};
use crate::config::LlmConfig;

const OLLAMA_DEFAULT_MODEL: &str = "llama3.1:8b";

pub struct OllamaClient {
    model: String,
}

impl OllamaClient {
    pub fn new(model: String) -> Self {
        OllamaClient { model }
    }

    /// Build from `LlmConfig`.
    pub fn from_config(cfg: &LlmConfig) -> Self {
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
