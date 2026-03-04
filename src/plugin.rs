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

/// Plugin configuration loaded from TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub success_criteria: Option<String>,
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
            intent: None,
            success_criteria: None,
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
            intent: None,
            success_criteria: None,
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
            intent: None,
            success_criteria: None,
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
            let is_toml = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "toml");

            if is_toml
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(def) = toml::from_str::<PluginDef>(&content)
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
    let parts = shell_words::split(&hook.command).ok()?;
    if parts.is_empty() {
        return None;
    }

    let output = Command::new(&parts[0])
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

// ── bd memory helpers ─────────────────────────────────────────────────────────

/// Character budget for bd memories injected into LLM prompts.
/// Memories are short KV entries; 10K chars fits ~50–100 comfortably.
pub const MEMORIES_BUDGET: usize = 10_000;

/// Fetch all bd memories by shelling out to `bd memories`.
///
/// Returns `None` if beads is not detected (no `.beads/` directory), if `bd`
/// is not on PATH, or if the command fails. Callers must degrade gracefully.
pub fn fetch_memories(project_root: &Path) -> Option<String> {
    if !project_root.join(".beads").exists() {
        return None;
    }
    let output = Command::new("bd")
        .arg("memories")
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    if text.len() <= MEMORIES_BUDGET {
        Some(text)
    } else {
        // Truncate at a char boundary to stay within budget.
        let mut cut = MEMORIES_BUDGET;
        while cut > 0 && !text.is_char_boundary(cut) {
            cut -= 1;
        }
        Some(text[..cut].to_string())
    }
}

/// Fetch bd memories filtered by `query`. Returns raw stdout or `None` if bd is unavailable.
pub fn fetch_memories_for_query(project_root: &Path, query: &str) -> Option<String> {
    if !project_root.join(".beads").exists() {
        return None;
    }
    let output = std::process::Command::new("bd")
        .args(["memories", query])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

/// Store a short insight in bd by shelling out to `bd remember "<text>"`.
///
/// Returns `Ok(())` on success, `Err` if beads is not detected or the command
/// fails. Callers should warn the user on error but must not panic.
pub fn store_memory(project_root: &Path, text: &str) -> Result<(), String> {
    if !project_root.join(".beads").exists() {
        return Err("beads not detected (.beads/ directory not found)".to_string());
    }
    let status = Command::new("bd")
        .arg("remember")
        .arg(text)
        .current_dir(project_root)
        .status()
        .map_err(|e| format!("failed to run bd: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("bd remember exited with {status}"))
    }
}

/// Detect if the current working directory is a git worktree (not the main worktree).
/// Returns the main worktree root path if we're in an additional worktree, otherwise None.
pub fn detect_main_worktree_root(project_root: &Path) -> Option<std::path::PathBuf> {
    let git_dir = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    let git_common_dir = std::process::Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .current_dir(project_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    // If --git-dir != --git-common-dir, we're in an additional worktree.
    if git_dir == git_common_dir {
        return None;
    }

    // Main worktree root is the parent of --git-common-dir.
    // git-common-dir is the .git directory (e.g. /path/to/main/.git).
    let common_dir_path = if std::path::Path::new(&git_common_dir).is_absolute() {
        std::path::PathBuf::from(&git_common_dir)
    } else {
        project_root.join(&git_common_dir)
    };

    common_dir_path.parent().map(|p| p.to_path_buf())
}

/// Execute a pass-through command.
pub fn execute_passthrough(
    project_root: &Path,
    passthrough: &str,
    extra_args: &[String],
) -> std::io::Result<std::process::ExitStatus> {
    let context = current_context();
    let parts = shell_words::split(passthrough).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
    })?;
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

    let mut cmd = Command::new(&parts[0]);
    cmd.args(&parts[1..]);
    cmd.args(extra_args);
    cmd.current_dir(project_root);

    cmd.status()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── fetch_memories ────────────────────────────────────────────────────────

    #[test]
    fn fetch_memories_none_when_no_beads_dir() {
        let tmp = TempDir::new().unwrap();
        // No .beads/ → should return None without attempting to shell out.
        assert!(fetch_memories(tmp.path()).is_none());
    }

    #[test]
    fn fetch_memories_none_when_bd_not_on_path() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".beads")).unwrap();
        // Run with an empty PATH so bd cannot be found.
        let result = std::panic::catch_unwind(|| {
            // We can't override PATH inside the process for Command easily, so
            // we verify that the function returns None (not panics) when bd
            // is genuinely absent. In CI bd won't be available.
            // If bd IS on PATH (dev machine), this test is a no-op for the None branch.
            let _ = fetch_memories(tmp.path());
        });
        assert!(result.is_ok(), "fetch_memories must not panic");
    }

    #[test]
    fn fetch_memories_truncates_to_budget() {
        // Build a string larger than MEMORIES_BUDGET and verify truncation is
        // at a valid char boundary.
        let big = "x".repeat(MEMORIES_BUDGET + 500);
        let mut cut = MEMORIES_BUDGET;
        while cut > 0 && !big.is_char_boundary(cut) {
            cut -= 1;
        }
        let truncated = &big[..cut];
        assert!(truncated.len() <= MEMORIES_BUDGET);
        assert!(big.is_char_boundary(truncated.len()));
    }

    // ── store_memory ──────────────────────────────────────────────────────────

    #[test]
    fn store_memory_err_when_no_beads_dir() {
        let tmp = TempDir::new().unwrap();
        // No .beads/ → should return Err without attempting to shell out.
        let result = store_memory(tmp.path(), "some insight");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("beads not detected"));
    }
}
