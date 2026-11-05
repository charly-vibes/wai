use owo_colors::OwoColorize;

use super::{LlmClient, LlmError, in_agent_session};

pub const AGENT_SENTINEL: &str = "wai::agent-mode";

/// LLM backend that delegates to the enclosing Claude Code agent session.
///
/// When wai runs inside Claude Code, this backend writes context to stdout
/// (where the agent reads it) instead of making a separate API call.
/// The full output format is finalised in Phase 5; this implementation emits
/// a minimal placeholder wrapper so the sentinel round-trip can be tested.
pub struct AgentBackend;

/// Build the agent-mode output string: header, status line, and context block.
///
/// Separated from `complete()` so the format can be verified in unit tests
/// without capturing stdout.
pub fn format_agent_output(prompt: &str) -> String {
    format!(
        "\n  {} wai why — agent mode\n  {} Context prepared\n\n[AGENT CONTEXT]\n{}\n[/AGENT CONTEXT]\n",
        "◆".cyan(),
        "○".dimmed(),
        prompt
    )
}

impl LlmClient for AgentBackend {
    fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        print!("{}", format_agent_output(prompt));
        Ok(AGENT_SENTINEL.to_string())
    }

    fn is_available(&self) -> bool {
        in_agent_session()
    }

    fn name(&self) -> &str {
        "Agent"
    }

    fn model_id(&self) -> &str {
        "agent"
    }
}
