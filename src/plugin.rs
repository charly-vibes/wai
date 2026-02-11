use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::config::plugins_dir;
use crate::context::current_context;
use crate::error::WaiError;

/// Data returned by a plugin hook execution.
#[derive(Debug, Default)]
pub struct HookOutput {
    pub label: String,
    pub content: String,
}

/// A plugin command that passes through to an external CLI.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub passthrough: String,
    #[serde(default)]
    pub read_only: bool,
}

/// Hook definition from plugin config.
#[derive(Debug, Clone, Deserialize)]
pub struct HookDef {
    pub command: String,
    pub inject_as: String,
}

/// Plugin configuration loaded from YAML.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub detector: Option<DetectorDef>,
    #[serde(default)]
    pub commands: Vec<PluginCommand>,
    #[serde(default)]
    pub hooks: HashMap<String, HookDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetectorDef {
    #[serde(rename = "type")]
    pub detector_type: String,
    pub path: String,
}

/// Represents a detected and active plugin at runtime.
pub struct ActivePlugin {
    pub def: PluginDef,
    pub detected: bool,
}

/// Built-in plugin definitions.
pub fn builtin_plugins() -> Vec<PluginDef> {
    vec![
        PluginDef {
            name: "git".to_string(),
            description: "Git version control integration".to_string(),
            detector: Some(DetectorDef {
                detector_type: "directory".to_string(),
                path: ".git".to_string(),
            }),
            commands: vec![],
            hooks: HashMap::from([
                (
                    "on_handoff_generate".to_string(),
                    HookDef {
                        command: "git status --short".to_string(),
                        inject_as: "git_status".to_string(),
                    },
                ),
                (
                    "on_status".to_string(),
                    HookDef {
                        command: "git log --oneline -5".to_string(),
                        inject_as: "recent_commits".to_string(),
                    },
                ),
            ]),
        },
        PluginDef {
            name: "beads".to_string(),
            description: "Integration with beads issue tracker".to_string(),
            detector: Some(DetectorDef {
                detector_type: "directory".to_string(),
                path: ".beads".to_string(),
            }),
            commands: vec![
                PluginCommand {
                    name: "list".to_string(),
                    description: "List beads issues".to_string(),
                    passthrough: "bd list".to_string(),
                    read_only: true,
                },
                PluginCommand {
                    name: "show".to_string(),
                    description: "Show beads issue details".to_string(),
                    passthrough: "bd show".to_string(),
                    read_only: true,
                },
                PluginCommand {
                    name: "ready".to_string(),
                    description: "Show ready issues".to_string(),
                    passthrough: "bd ready".to_string(),
                    read_only: true,
                },
            ],
            hooks: HashMap::from([
                (
                    "on_handoff_generate".to_string(),
                    HookDef {
                        command: "bd list --status=open".to_string(),
                        inject_as: "open_issues".to_string(),
                    },
                ),
                (
                    "on_status".to_string(),
                    HookDef {
                        command: "bd stats".to_string(),
                        inject_as: "beads_stats".to_string(),
                    },
                ),
            ]),
        },
        PluginDef {
            name: "openspec".to_string(),
            description: "OpenSpec specification management".to_string(),
            detector: Some(DetectorDef {
                detector_type: "directory".to_string(),
                path: "openspec".to_string(),
            }),
            commands: vec![],
            hooks: HashMap::new(),
        },
    ]
}

/// Detect which plugins are available at the given project root.
pub fn detect_plugins(project_root: &Path) -> Vec<ActivePlugin> {
    let mut plugins = Vec::new();

    // Load built-in plugins
    for def in builtin_plugins() {
        let detected = if let Some(ref detector) = def.detector {
            project_root.join(&detector.path).exists()
        } else {
            false
        };
        plugins.push(ActivePlugin { def, detected });
    }

    // Load custom plugins from .wai/plugins/
    let plugins_dir = plugins_dir(project_root);
    if plugins_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&plugins_dir)
    {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let is_yaml = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "yml" || e == "yaml");

            if is_yaml
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(def) = serde_yaml::from_str::<PluginDef>(&content)
            {
                let detected = if let Some(ref detector) = def.detector {
                    project_root.join(&detector.path).exists()
                } else {
                    true
                };
                plugins.push(ActivePlugin { def, detected });
            }
        }
    }

    plugins
}

/// Execute a plugin hook and return its output.
pub fn execute_hook(project_root: &Path, hook: &HookDef) -> Option<HookOutput> {
    let parts: Vec<&str> = hook.command.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(project_root)
        .output()
        .ok()?;

    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }

    Some(HookOutput {
        label: hook.inject_as.clone(),
        content: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

/// Run all hooks for a given event across all detected plugins.
pub fn run_hooks(project_root: &Path, event: &str) -> Vec<HookOutput> {
    let plugins = detect_plugins(project_root);
    let mut outputs = Vec::new();

    for plugin in &plugins {
        if !plugin.detected {
            continue;
        }
        if let Some(hook) = plugin.def.hooks.get(event)
            && let Some(output) = execute_hook(project_root, hook)
        {
            outputs.push(output);
        }
    }

    outputs
}

/// Find a plugin command for pass-through execution.
pub fn find_plugin_command<'a>(
    plugins: &'a [ActivePlugin],
    plugin_name: &str,
    command_name: &str,
) -> Option<&'a PluginCommand> {
    for plugin in plugins {
        if plugin.def.name == plugin_name && plugin.detected {
            for cmd in &plugin.def.commands {
                if cmd.name == command_name {
                    return Some(cmd);
                }
            }
        }
    }
    None
}

/// Execute a pass-through command.
pub fn execute_passthrough(
    project_root: &Path,
    passthrough: &str,
    extra_args: &[String],
) -> std::io::Result<std::process::ExitStatus> {
    let context = current_context();
    let parts: Vec<&str> = passthrough.split_whitespace().collect();
    if parts.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Empty passthrough command",
        ));
    }

    if context.no_input {
        return Err(std::io::Error::other(WaiError::NonInteractive {
            message: "Plugin passthrough requires interactive input".to_string(),
        }));
    }

    if context.safe {
        return Err(std::io::Error::other(WaiError::SafeModeViolation {
            action: "plugin passthrough".to_string(),
        }));
    }

    let mut cmd = Command::new(parts[0]);
    cmd.args(&parts[1..]);
    cmd.args(extra_args);
    cmd.current_dir(project_root);

    cmd.status()
}
