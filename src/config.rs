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
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
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

/// User-level configuration (stored in ~/.config/wai/config.toml)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserConfig {
    #[serde(default)]
    pub seen_tutorial: bool,
    #[serde(default)]
    pub version: String,
}

impl UserConfig {
    /// Load user config from ~/.config/wai/config.toml
    /// Returns default config if file doesn't exist
    pub fn load() -> Result<Self, WaiError> {
        let config_path = user_config_path();

        if !config_path.exists() {
            return Ok(Self::default());
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
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join(USER_CONFIG_DIR)
    } else if let Some(home) = dirs::home_dir() {
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

/// Check if this is the first time running wai
pub fn is_first_run() -> Result<bool, WaiError> {
    let config = UserConfig::load()?;
    Ok(!config.seen_tutorial)
}

/// Mark the tutorial as seen for the current user
pub fn mark_tutorial_seen() -> Result<(), WaiError> {
    let mut config = UserConfig::load()?;
    config.mark_tutorial_seen();
    config.save()
}
