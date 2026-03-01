use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::WaiError;

pub const CONFIG_DIR: &str = ".wai";
pub const CONFIG_FILE: &str = "config.toml";

// User-level config (stored in ~/.config/wai/)
pub const USER_CONFIG_DIR: &str = "wai";
pub const USER_CONFIG_FILE: &str = "config.toml";

/// PARA directory names within .wai/
pub const PROJECTS_DIR: &str = "projects";
pub const AREAS_DIR: &str = "areas";
pub const RESOURCES_DIR: &str = "resources";
pub const ARCHIVES_DIR: &str = "archives";
pub const PLUGINS_DIR: &str = "plugins";

/// Agent config subdirectories within resources/
pub const AGENT_CONFIG_DIR: &str = "agent-config";
pub const SKILLS_DIR: &str = "skills";
pub const RULES_DIR: &str = "rules";
pub const CONTEXT_DIR: &str = "context";

/// Pipeline definitions and run state within resources/
pub const PIPELINES_DIR: &str = "pipelines";

/// Active pipeline run state file (`.wai/.pipeline-run`).
///
/// Written by `wai pipeline run` and removed by `wai pipeline advance` when
/// the last stage completes. Similar to `.git/HEAD` — a single-line file
/// containing just the run ID. Not committed (listed in `.wai/.gitignore`).
pub const PIPELINE_RUN_FILE: &str = ".pipeline-run";

/// Per-project subdirectories
pub const RESEARCH_DIR: &str = "research";
pub const PLANS_DIR: &str = "plans";
pub const DESIGNS_DIR: &str = "designs";
pub const HANDOFFS_DIR: &str = "handoffs";
pub const STATE_FILE: &str = ".state";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub project: ProjectSettings,
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
    /// New canonical LLM configuration section (`[llm]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm: Option<LlmConfig>,
    /// Legacy `[why]` section kept for backwards-compatible deserialisation.
    ///
    /// New code should always write to `llm`. This field is only populated
    /// when loading a config that was written by an older version of wai.
    /// Use [`ProjectConfig::llm_config`] to access the effective settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<LlmConfig>,
}

impl ProjectConfig {
    /// Return the effective LLM configuration.
    ///
    /// Prefers `[llm]` when present. Falls back to `[why]` (legacy) with a
    /// one-time deprecation warning printed to stderr.
    pub fn llm_config(&self) -> std::borrow::Cow<'_, LlmConfig> {
        if let Some(llm) = &self.llm {
            return std::borrow::Cow::Borrowed(llm);
        }
        if self.why.is_some() {
            eprintln!(
                "  [wai] Deprecation: rename [why] to [llm] in .wai/config.toml"
            );
        }
        match &self.why {
            Some(why) => std::borrow::Cow::Borrowed(why),
            None => std::borrow::Cow::Owned(LlmConfig::default()),
        }
    }
}

/// Configuration for the LLM backend used by `wai why` and `wai reflect`.
///
/// Stored under `[llm]` in `.wai/config.toml`. The legacy `[why]` section is
/// still accepted for backwards compatibility.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LlmConfig {
    /// LLM backend to use: "claude" or "ollama". Omit for auto-detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<String>,

    /// Model alias. For Claude: "haiku" or "sonnet". For Ollama: e.g. "llama3.1:8b".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// API key for Claude (also reads ANTHROPIC_API_KEY env var).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Fallback when no LLM is available: "search" (default) or "error".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,

    /// Whether the one-time privacy notice has been shown to the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_notice_shown: Option<bool>,
}

/// `WhyConfig` is a deprecated alias for [`LlmConfig`].
///
/// Kept so that any external code compiled against this crate continues to
/// build. New code should use `LlmConfig` directly.
#[deprecated(since = "0.0.0", note = "Use LlmConfig instead")]
#[allow(dead_code)]
pub type WhyConfig = LlmConfig;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    /// Git commit hash of the wai binary when workspace was last synced.
    /// Empty on old workspaces (pre-commit tracking).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tool_commit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub settings: toml::Table,
}

impl ProjectConfig {
    pub fn load(project_root: &Path) -> Result<Self, WaiError> {
        let config_path = project_root.join(CONFIG_DIR).join(CONFIG_FILE);

        if !config_path.exists() {
            // Check if .wai directory exists to provide better error message
            if project_root.join(CONFIG_DIR).exists() {
                return Err(WaiError::ConfigMissing);
            }
            return Err(WaiError::NotInitialized);
        }

        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(|e| WaiError::ConfigError {
            message: e.to_string(),
        })
    }

    pub fn save(&self, project_root: &Path) -> Result<(), WaiError> {
        let config_dir = project_root.join(CONFIG_DIR);
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join(CONFIG_FILE);
        let content = toml::to_string_pretty(self).map_err(|e| WaiError::ConfigError {
            message: e.to_string(),
        })?;

        std::fs::write(config_path, content)?;
        Ok(())
    }
}

pub fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    loop {
        if current.join(CONFIG_DIR).exists() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

/// Get the .wai directory path from a project root.
pub fn wai_dir(project_root: &Path) -> PathBuf {
    project_root.join(CONFIG_DIR)
}

/// Get the projects directory path.
pub fn projects_dir(project_root: &Path) -> PathBuf {
    wai_dir(project_root).join(PROJECTS_DIR)
}

/// Get the areas directory path.
pub fn areas_dir(project_root: &Path) -> PathBuf {
    wai_dir(project_root).join(AREAS_DIR)
}

/// Get the resources directory path.
pub fn resources_dir(project_root: &Path) -> PathBuf {
    wai_dir(project_root).join(RESOURCES_DIR)
}

/// Get the archives directory path.
pub fn archives_dir(project_root: &Path) -> PathBuf {
    wai_dir(project_root).join(ARCHIVES_DIR)
}

/// Get the plugins directory path.
pub fn plugins_dir(project_root: &Path) -> PathBuf {
    wai_dir(project_root).join(PLUGINS_DIR)
}

/// Get the agent-config directory path.
pub fn agent_config_dir(project_root: &Path) -> PathBuf {
    resources_dir(project_root).join(AGENT_CONFIG_DIR)
}

/// Get the pipelines directory path (.wai/resources/pipelines/).
pub fn pipelines_dir(project_root: &Path) -> PathBuf {
    resources_dir(project_root).join(PIPELINES_DIR)
}

/// Get a specific project's directory path.
pub fn project_path(project_root: &Path, name: &str) -> PathBuf {
    projects_dir(project_root).join(name)
}

/// Get a specific area's directory path.
pub fn area_path(project_root: &Path, name: &str) -> PathBuf {
    areas_dir(project_root).join(name)
}

/// Get a specific resource's directory path.
pub fn resource_path(project_root: &Path, name: &str) -> PathBuf {
    resources_dir(project_root).join(name)
}

/// Get the global skills directory path (~/.wai/resources/skills/).
///
/// This is the user-level skill library shared across all projects.
/// Skills here are available in every project, with local skills taking priority.
pub fn global_skills_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".wai").join(RESOURCES_DIR).join(SKILLS_DIR)
    } else {
        PathBuf::from(".")
            .join(".wai")
            .join(RESOURCES_DIR)
            .join(SKILLS_DIR)
    }
}

/// User-level configuration (stored in ~/.config/wai/config.toml)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserConfig {
    #[serde(default)]
    pub seen_tutorial: bool,
    #[serde(default)]
    pub version: String,
}

impl UserConfig {
    /// Load user config from ~/.config/wai/config.toml.
    ///
    /// If the file does not exist, writes a default config and returns it.
    /// This ensures the config file is always initialized on first use,
    /// so callers never need to save a freshly-loaded default themselves.
    pub fn load() -> Result<Self, WaiError> {
        let config_path = user_config_path();

        if !config_path.exists() {
            let default = Self::default();
            // Best-effort: ignore save errors so read-only callers still work.
            let _ = default.save();
            return Ok(default);
        }

        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(|e| WaiError::ConfigError {
            message: e.to_string(),
        })
    }

    /// Save user config to ~/.config/wai/config.toml
    pub fn save(&self) -> Result<(), WaiError> {
        let config_dir = user_config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join(USER_CONFIG_FILE);
        let content = toml::to_string_pretty(self).map_err(|e| WaiError::ConfigError {
            message: e.to_string(),
        })?;

        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Mark the tutorial as seen
    pub fn mark_tutorial_seen(&mut self) {
        self.seen_tutorial = true;
    }
}

/// Get the user config directory path (~/.config/wai/)
pub fn user_config_dir() -> PathBuf {
    // Check XDG_CONFIG_HOME first, but ensure it's non-empty
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME")
        && !xdg_config.is_empty()
    {
        return PathBuf::from(xdg_config).join(USER_CONFIG_DIR);
    }

    if let Some(home) = dirs::home_dir() {
        home.join(".config").join(USER_CONFIG_DIR)
    } else {
        // Fallback to current directory (shouldn't happen in practice)
        PathBuf::from(".").join(".config").join(USER_CONFIG_DIR)
    }
}

/// Get the user config file path (~/.config/wai/config.toml)
pub fn user_config_path() -> PathBuf {
    user_config_dir().join(USER_CONFIG_FILE)
}

/// Mark the tutorial as seen for the current user
pub fn mark_tutorial_seen() -> Result<(), WaiError> {
    let mut config = UserConfig::load()?;
    config.mark_tutorial_seen();
    config.save()
}

/// Get the path to the active pipeline run state file (`.wai/.pipeline-run`).
pub fn pipeline_run_file(project_root: &Path) -> PathBuf {
    wai_dir(project_root).join(PIPELINE_RUN_FILE)
}

/// Get the path to the `.last-run` pointer file used by TOML-based pipelines.
///
/// This file stores the most recently started run ID so that `wai pipeline next`
/// and `wai pipeline current` can resolve the active run without requiring the
/// user to set `WAI_PIPELINE_RUN` in their shell.
///
/// Stored at `.wai/resources/pipelines/.last-run`.
pub fn last_run_path(workspace_root: &Path) -> PathBuf {
    pipelines_dir(workspace_root).join(".last-run")
}

/// Write a run ID to the active pipeline run state file.
///
/// Creates or overwrites `.wai/.pipeline-run` with just the run ID (no trailing newline).
/// This mirrors how `.git/HEAD` stores a simple reference string.
pub fn write_pipeline_run_state(project_root: &Path, run_id: &str) -> Result<(), WaiError> {
    let path = pipeline_run_file(project_root);
    std::fs::write(&path, run_id)?;
    Ok(())
}

/// Read the active pipeline run ID from the state file, if present.
///
/// Returns `None` when the file does not exist or its contents are empty.
pub fn read_pipeline_run_state(project_root: &Path) -> Option<String> {
    let path = pipeline_run_file(project_root);
    let content = std::fs::read_to_string(&path).ok()?;
    let trimmed = content.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}
