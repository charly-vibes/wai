use super::{LlmClient, LlmError};
use crate::config::LlmConfig;

pub struct ClaudeCliClient;

impl ClaudeCliClient {
    pub fn from_config(_cfg: &LlmConfig) -> Self {
        ClaudeCliClient
    }
}

/// Return `true` when running inside an agent session.
///
/// Checks env vars in priority order — first non-empty value wins:
/// 1. `WAI_AGENT`    — universal override for any agent (Windsurf, Goose, …)
/// 2. `CLAUDECODE`   — set by Claude Code when an agent is active
/// 3. `CURSOR_AGENT` — set by Cursor IDE
///
/// An empty string is treated as unset (false). This allows
/// [`ClaudeCliClient`] to bypass the nested-session guard by clearing
/// `CLAUDECODE` without triggering detection via the other vars.
pub fn in_agent_session() -> bool {
    ["WAI_AGENT", "CLAUDECODE", "CURSOR_AGENT"]
        .iter()
        .any(|var| std::env::var(var).map(|v| !v.is_empty()).unwrap_or(false))
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
