use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::WaiError;

pub const CONFIG_DIR: &str = ".wai";
pub const CONFIG_FILE: &str = "config.toml";

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
    pub fn load(project_root: &PathBuf) -> Result<Self, WaiError> {
        let config_path = project_root.join(CONFIG_DIR).join(CONFIG_FILE);

        if !config_path.exists() {
            return Err(WaiError::NotInitialized);
        }

        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(|e| WaiError::ConfigError {
            message: e.to_string(),
        })
    }

    pub fn save(&self, project_root: &PathBuf) -> Result<(), WaiError> {
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
