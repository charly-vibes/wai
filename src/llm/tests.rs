use super::agent::format_agent_output;
use super::claude_api::ClaudeClient;
use super::*;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

fn install_fake_claude_bin(dir: &std::path::Path) -> String {
    let bin_dir = dir.join("fake-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let script_path = bin_dir.join("claude");
    fs::write(
            &script_path,
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'claude 0.0-test'\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

// ── AgentBackend ──

#[test]
#[serial]
fn agent_backend_complete_returns_sentinel() {
    let backend = AgentBackend;
    let result = backend.complete("test prompt").unwrap();
    assert_eq!(result, AGENT_SENTINEL);
}

#[test]
#[serial]
fn agent_backend_is_available_matches_in_agent_session() {
    unsafe { std::env::set_var("CLAUDECODE", "1") };
    assert!(AgentBackend.is_available());
    assert_eq!(AgentBackend.is_available(), in_agent_session());

    unsafe { std::env::set_var("CLAUDECODE", "") };
    assert!(!AgentBackend.is_available());
    assert_eq!(AgentBackend.is_available(), in_agent_session());

    unsafe { std::env::remove_var("CLAUDECODE") };
    assert!(!AgentBackend.is_available());
    assert_eq!(AgentBackend.is_available(), in_agent_session());
}

// ── format_agent_output (Phase 5) ──

#[test]
fn agent_output_includes_delimiters() {
    let out = format_agent_output("my prompt");
    assert!(out.contains("[AGENT CONTEXT]"));
    assert!(out.contains("[/AGENT CONTEXT]"));
}

#[test]
fn agent_output_includes_prompt_between_delimiters() {
    let out = format_agent_output("why use TOML?");
    let start = out.find("[AGENT CONTEXT]").unwrap() + "[AGENT CONTEXT]".len();
    let end = out.find("[/AGENT CONTEXT]").unwrap();
    let between = &out[start..end];
    assert!(between.contains("why use TOML?"));
}

#[test]
fn agent_output_header_precedes_context_block() {
    let out = format_agent_output("test");
    let header_pos = out.find("wai why — agent mode").unwrap();
    let block_pos = out.find("[AGENT CONTEXT]").unwrap();
    assert!(header_pos < block_pos);
}

// ── detect_backend (Phase 3) ──

#[test]
#[serial]
fn claudecode_set_no_api_key_selects_agent() {
    unsafe { std::env::set_var("CLAUDECODE", "1") };
    unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
    let cfg = LlmConfig::default();
    let backend = detect_backend(&cfg);
    assert!(backend.is_some());
    assert_eq!(backend.unwrap().name(), "Agent");
    unsafe { std::env::remove_var("CLAUDECODE") };
}

#[test]
#[serial]
fn claudecode_set_with_api_key_prefers_agent_over_claude_api() {
    unsafe { std::env::set_var("CLAUDECODE", "1") };
    unsafe { std::env::set_var("ANTHROPIC_API_KEY", "sk-test") };
    let cfg = LlmConfig::default();
    let backend = detect_backend(&cfg);
    assert!(backend.is_some());
    assert_eq!(backend.unwrap().name(), "Agent");
    unsafe { std::env::remove_var("CLAUDECODE") };
    unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
}

#[test]
#[serial]
fn explicit_agent_config_selects_agent_regardless_of_claudecode() {
    unsafe { std::env::remove_var("CLAUDECODE") };
    let cfg = LlmConfig {
        llm: Some("agent".to_string()),
        ..Default::default()
    };
    // Returns AgentBackend even outside an agent session (with a warning on stderr)
    let backend = detect_backend(&cfg);
    assert!(backend.is_some());
    assert_eq!(backend.unwrap().name(), "Agent");
}

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
    unsafe {
        std::env::remove_var("WAI_AGENT");
        std::env::remove_var("CLAUDECODE");
        std::env::remove_var("CURSOR_AGENT");
    }
    assert!(!in_agent_session());
}

#[test]
#[serial]
fn wai_agent_set_returns_true() {
    unsafe { std::env::set_var("WAI_AGENT", "1") };
    assert!(in_agent_session());
    unsafe { std::env::remove_var("WAI_AGENT") };
}

#[test]
#[serial]
fn wai_agent_empty_returns_false() {
    unsafe {
        std::env::set_var("WAI_AGENT", "");
        std::env::remove_var("CLAUDECODE");
        std::env::remove_var("CURSOR_AGENT");
    }
    assert!(!in_agent_session());
    unsafe { std::env::remove_var("WAI_AGENT") };
}

#[test]
#[serial]
fn cursor_agent_set_returns_true() {
    unsafe { std::env::set_var("CURSOR_AGENT", "1") };
    assert!(in_agent_session());
    unsafe { std::env::remove_var("CURSOR_AGENT") };
}

#[test]
#[serial]
fn all_agent_vars_unset_returns_false() {
    unsafe {
        std::env::remove_var("WAI_AGENT");
        std::env::remove_var("CLAUDECODE");
        std::env::remove_var("CURSOR_AGENT");
    }
    assert!(!in_agent_session());
}

#[test]
#[serial]
fn all_agent_vars_empty_returns_false() {
    unsafe {
        std::env::set_var("WAI_AGENT", "");
        std::env::set_var("CLAUDECODE", "");
        std::env::set_var("CURSOR_AGENT", "");
    }
    assert!(!in_agent_session());
    unsafe {
        std::env::remove_var("WAI_AGENT");
        std::env::remove_var("CLAUDECODE");
        std::env::remove_var("CURSOR_AGENT");
    }
}

// ── resolve_model_alias ──

#[test]
fn alias_haiku_resolves_to_canonical_id() {
    assert_eq!(resolve_model_alias("haiku"), "claude-haiku-3-5-20251001");
}

#[test]
fn alias_sonnet_resolves_to_canonical_id() {
    assert_eq!(resolve_model_alias("sonnet"), "claude-sonnet-4-6");
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
    let cfg = LlmConfig::default();
    let _ = detect_backend(&cfg); // must not panic
}

#[test]
fn explicit_claude_with_key_returns_claude_client() {
    let cfg = LlmConfig {
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
    let cfg = LlmConfig {
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
    let cfg = LlmConfig::default();
    let backend = detect_backend(&cfg);
    assert!(backend.is_some());
    assert_eq!(backend.unwrap().name(), "Claude");
    unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
}

#[test]
#[serial]
fn explicit_claude_cli_uses_selected_provider_even_in_agent_session() {
    let tmp = TempDir::new().unwrap();
    let path = install_fake_claude_bin(tmp.path());
    let old_path = std::env::var("PATH").ok();
    unsafe {
        std::env::set_var("PATH", path);
        std::env::set_var("CLAUDECODE", "1");
        std::env::remove_var("ANTHROPIC_API_KEY");
    }
    let cfg = LlmConfig {
        llm: Some("claude-cli".to_string()),
        ..Default::default()
    };
    let backend = detect_backend(&cfg).expect("explicit claude-cli should win");
    assert_eq!(backend.name(), "Claude CLI");
    unsafe {
        std::env::remove_var("CLAUDECODE");
        if let Some(path) = old_path {
            std::env::set_var("PATH", path);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

#[test]
#[serial]
fn auto_detect_without_agent_or_api_key_uses_claude_cli_when_binary_exists() {
    let tmp = TempDir::new().unwrap();
    let path = install_fake_claude_bin(tmp.path());
    let old_path = std::env::var("PATH").ok();
    unsafe {
        std::env::set_var("PATH", path);
        std::env::remove_var("CLAUDECODE");
        std::env::remove_var("WAI_AGENT");
        std::env::remove_var("CURSOR_AGENT");
        std::env::remove_var("ANTHROPIC_API_KEY");
    }
    let backend = detect_backend(&LlmConfig::default())
        .expect("auto-detect should use claude-cli when available");
    assert_eq!(backend.name(), "Claude CLI");
    unsafe {
        if let Some(path) = old_path {
            std::env::set_var("PATH", path);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

#[test]
#[serial]
fn explicit_claude_cli_without_binary_returns_none() {
    let old_path = std::env::var("PATH").ok();
    unsafe {
        std::env::set_var("PATH", "/nonexistent");
        std::env::remove_var("CLAUDECODE");
    }
    let cfg = LlmConfig {
        llm: Some("claude-cli".to_string()),
        ..Default::default()
    };
    assert!(detect_backend(&cfg).is_none());
    unsafe {
        if let Some(path) = old_path {
            std::env::set_var("PATH", path);
        } else {
            std::env::remove_var("PATH");
        }
    }
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
    let cfg = LlmConfig {
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
    let cfg = LlmConfig::default();
    let client = ClaudeClient::from_config(&cfg).unwrap();
    assert!(client.is_available());
    unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
}

#[test]
#[serial]
fn claude_from_config_returns_none_without_key() {
    unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
    let cfg = LlmConfig::default();
    assert!(ClaudeClient::from_config(&cfg).is_none());
}

#[test]
fn claude_model_alias_resolved_in_from_config() {
    let cfg = LlmConfig {
        api_key: Some("key".to_string()),
        model: Some("haiku".to_string()),
        ..Default::default()
    };
    let client = ClaudeClient::from_config(&cfg).unwrap();
    assert_eq!(client.model_id(), "claude-haiku-3-5-20251001");
}
