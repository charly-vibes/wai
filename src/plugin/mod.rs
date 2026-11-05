use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

const PLUGIN_TIMEOUT: Duration = Duration::from_secs(30);

use crate::config::plugins_dir;
use crate::context::current_context;
use crate::error::WaiError;

// ── Trust store ────────────────────────────────────────────────────────────────

/// The name of the trust store file inside the wai data directory.
const TRUST_STORE_FILE: &str = "plugin-trust.toml";

/// A single entry in the plugin trust store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEntry {
    pub plugin_name: String,
    pub hook_name: String,
    pub digest: String,
    pub command: String,
    pub approved_at: String,
}

/// The plugin trust store — a list of approved hook digests stored in
/// user-owned XDG state outside the repository.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrustStore {
    entries: Vec<TrustEntry>,
}

impl TrustStore {
    /// Load the trust store from disk.
    pub fn load() -> Self {
        let path = trust_store_path();
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(store) = toml::from_str::<TrustStore>(&content)
        {
            return store;
        }
        TrustStore::default()
    }

    /// Save the trust store to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = trust_store_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create trust store dir: {e}"))?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("cannot serialize trust store: {e}"))?;
        std::fs::write(&path, content).map_err(|e| format!("cannot write trust store: {e}"))?;
        Ok(())
    }

    /// Check whether a digest is approved.
    pub fn is_approved(&self, digest: &str) -> bool {
        self.entries.iter().any(|e| e.digest == digest)
    }

    /// Approve a digest. Replaces any existing entry with the same digest.
    pub fn approve(&mut self, entry: TrustEntry) {
        self.entries.retain(|e| e.digest != entry.digest);
        self.entries.push(entry);
    }

    /// Revoke a digest.
    pub fn revoke(&mut self, digest: &str) -> bool {
        let len = self.entries.len();
        self.entries.retain(|e| e.digest != digest);
        self.entries.len() < len
    }

    /// List all entries.
    pub fn list(&self) -> &[TrustEntry] {
        &self.entries
    }
}

/// Get the path to the trust store file.
/// Uses `WAI_DATA_DIR` env var override, then `XDG_DATA_HOME`, then `~/.local/share/wai/`.
pub fn trust_store_path() -> PathBuf {
    if let Ok(dir) = std::env::var("WAI_DATA_DIR") {
        return PathBuf::from(dir).join(TRUST_STORE_FILE);
    }

    let data_dir = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data).join("wai")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".local").join("share").join("wai")
    } else {
        PathBuf::from(".wai-data")
    };

    data_dir.join(TRUST_STORE_FILE)
}

/// Compute a SHA-256 digest for a plugin hook.
/// The digest covers the plugin name, all commands, the hook name, and the hook command.
/// Any change to these inputs produces a different digest.
pub fn compute_hook_digest(plugin: &PluginDef, hook_name: &str, hook: &HookDef) -> String {
    use sha2::Digest;

    let mut hasher = sha2::Sha256::new();
    hasher.update(plugin.name.as_bytes());
    for cmd in &plugin.commands {
        hasher.update(cmd.name.as_bytes());
        hasher.update(cmd.passthrough.as_bytes());
    }
    hasher.update(hook_name.as_bytes());
    hasher.update(hook.command.as_bytes());
    hasher.update(hook.inject_as.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Whether a plugin is built-in or user-defined.
#[derive(Debug, Clone, PartialEq)]
pub enum PluginSource {
    BuiltIn,
    Custom,
}

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
    pub source: PluginSource,
}

/// Built-in plugin definitions.
/// Built-in plugin definitions.
pub fn builtin_plugins() -> Vec<PluginDef> {
    vec![
        git_plugin(),
        beads_plugin(),
        openspec_plugin(),
        testaruda_plugin(),
        espectacular_plugin(),
        dont_plugin(),
        pretender_plugin(),
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
        plugins.push(ActivePlugin {
            def,
            detected,
            source: PluginSource::BuiltIn,
        });
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
                plugins.push(ActivePlugin {
                    def,
                    detected,
                    source: PluginSource::Custom,
                });
            }
        }
    }

    plugins
}

/// Execute a plugin hook and return its output.
///
/// Enforces a 30-second timeout: if the child process does not complete within
/// that window it is killed and `None` is returned.
pub fn execute_hook(project_root: &Path, hook: &HookDef) -> Option<HookOutput> {
    use std::sync::{Arc, Mutex};

    let parts = shell_words::split(&hook.command).ok()?;
    if parts.is_empty() {
        return None;
    }

    let child = Command::new(&parts[0])
        .args(&parts[1..])
        .current_dir(project_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;

    let _pid = child.id();
    let child = Arc::new(Mutex::new(child));
    let rx = passthrough::wait_with_poller(&child);

    match rx.recv_timeout(PLUGIN_TIMEOUT) {
        Ok(Ok(status)) => hook_output_from_child(&child, hook, status),
        Ok(Err(_)) => None,
        Err(_) => {
            // Timed out — terminate the child process.
            // The waiter thread releases the mutex between try_wait() calls,
            // so we can acquire it here (unlike the old wait() which held it).
            let _ = child.lock().unwrap().kill();
            // Wait briefly for the waiter thread to reap the child.
            let _ = rx.recv_timeout(Duration::from_secs(2));
            None
        }
    }
}

/// Collect the child's piped stdout into a HookOutput; empty/failed runs are None.
fn hook_output_from_child(
    child: &std::sync::Arc<std::sync::Mutex<std::process::Child>>,
    hook: &HookDef,
    status: std::process::ExitStatus,
) -> Option<HookOutput> {
    let mut guard = child.lock().unwrap();
    let stdout = guard.stdout.take();
    drop(guard);
    let content = if let Some(mut out) = stdout {
        use std::io::Read;
        let mut buf = String::new();
        out.read_to_string(&mut buf).ok();
        buf
    } else {
        String::new()
    };
    if !status.success() || content.is_empty() {
        return None;
    }
    Some(HookOutput {
        label: hook.inject_as.clone(),
        content,
    })
}

/// Run all hooks for a given event across all detected plugins.
///
/// Custom (repository-owned) plugins are only executed when their hook digest
/// is present in the user's trust store. Unapproved hooks are skipped with a
/// warning; they are never executed and never prompt. Built-in plugins are
/// always trusted.
pub fn run_hooks(project_root: &Path, event: &str) -> Vec<HookOutput> {
    let plugins = detect_plugins(project_root);
    let mut outputs = Vec::new();
    let trust_store = TrustStore::load();

    for plugin in &plugins {
        if !plugin.detected {
            continue;
        }
        if let Some(hook) = plugin.def.hooks.get(event) {
            match plugin.source {
                PluginSource::BuiltIn => {
                    if let Some(output) = execute_hook(project_root, hook) {
                        outputs.push(output);
                    }
                }
                PluginSource::Custom => {
                    let digest = compute_hook_digest(&plugin.def, event, hook);
                    if trust_store.is_approved(&digest) {
                        if let Some(output) = execute_hook(project_root, hook) {
                            outputs.push(output);
                        }
                    } else {
                        warn_skipped_hook(&plugin.def.name, event, &hook.inject_as);
                    }
                }
            }
        }
    }

    outputs
}

/// Emit a warning about a skipped (untrusted) hook.
/// In machine mode this is a structured JSON warning; otherwise a human line.
fn warn_skipped_hook(plugin_name: &str, event: &str, inject_as: &str) {
    let context = current_context();
    if context.json {
        let warning = serde_json::json!({
            "level": "warning",
            "code": "plugin_hook_untrusted",
            "plugin": plugin_name,
            "event": event,
            "inject_as": inject_as,
            "message": format!("Plugin hook '{}' is not trusted and was skipped. Approve it with: wai plugin trust {}", inject_as, plugin_name),
        });
        println!("{}", warning);
    } else {
        use owo_colors::OwoColorize;
        eprintln!(
            "  {} Plugin '{}' hook '{}' skipped: not trusted (approve with `wai plugin trust {}`)",
            "!".yellow(),
            plugin_name,
            inject_as,
            plugin_name
        );
    }
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

// Character budget for bd memories injected into LLM prompts lives in
// memories.rs (MEMORIES_BUDGET).
mod memories;
mod passthrough;

#[cfg(test)]
mod tests;

pub use memories::{
    MEMORIES_BUDGET, detect_main_worktree_root, fetch_memories, fetch_memories_for_query,
    store_memory,
};
pub use passthrough::execute_passthrough;

fn git_plugin() -> PluginDef {
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
    }
}

fn beads_plugin() -> PluginDef {
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
    }
}

fn openspec_plugin() -> PluginDef {
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
    }
}

fn testaruda_plugin() -> PluginDef {
    PluginDef {
        name: "testaruda".to_string(),
        description: "Integration with testaruda test harness".to_string(),
        intent: None,
        success_criteria: None,
        detector: Some(DetectorDef {
            detector_type: "file".to_string(),
            path: "testaruda.toml".to_string(),
        }),
        commands: vec![PluginCommand {
            name: "select".to_string(),
            description: "Select affected tests from a code change".to_string(),
            passthrough: "testaruda select".to_string(),
            read_only: true,
        }],
        hooks: HashMap::from([(
            "on_status".to_string(),
            HookDef {
                command: "testaruda metrics".to_string(),
                inject_as: "testaruda_metrics".to_string(),
            },
        )]),
    }
}

fn espectacular_plugin() -> PluginDef {
    PluginDef {
        name: "espectacular".to_string(),
        description: "Integration with espectacular spec-test correspondence".to_string(),
        intent: None,
        success_criteria: None,
        detector: Some(DetectorDef {
            detector_type: "directory".to_string(),
            path: ".espectacular".to_string(),
        }),
        commands: vec![PluginCommand {
            name: "check".to_string(),
            description: "Verify spec-test correspondence".to_string(),
            passthrough: "ah check".to_string(),
            read_only: true,
        }],
        hooks: HashMap::from([(
            "on_status".to_string(),
            HookDef {
                command: "sh -c 'ah doctor || true'".to_string(),
                inject_as: "espectacular_doctor".to_string(),
            },
        )]),
    }
}

fn dont_plugin() -> PluginDef {
    PluginDef {
        name: "dont".to_string(),
        description: "Integration with dont decision-logged conventions".to_string(),
        intent: None,
        success_criteria: None,
        detector: Some(DetectorDef {
            detector_type: "directory".to_string(),
            path: ".dont".to_string(),
        }),
        commands: vec![PluginCommand {
            name: "check".to_string(),
            description: "Verify claims and terms are grounded".to_string(),
            passthrough: "dont check".to_string(),
            read_only: true,
        }],
        hooks: HashMap::from([(
            "on_status".to_string(),
            HookDef {
                command: "dont check".to_string(),
                inject_as: "dont_check".to_string(),
            },
        )]),
    }
}

fn pretender_plugin() -> PluginDef {
    PluginDef {
        name: "pretender".to_string(),
        description: "Integration with pretender structural code quality".to_string(),
        intent: None,
        success_criteria: None,
        detector: Some(DetectorDef {
            detector_type: "file".to_string(),
            path: "pretender.toml".to_string(),
        }),
        commands: vec![PluginCommand {
            name: "check".to_string(),
            description: "Fast pass/fail scan against thresholds".to_string(),
            passthrough: "pretender check".to_string(),
            read_only: true,
        }],
        hooks: HashMap::from([(
            "on_status".to_string(),
            HookDef {
                command: "sh -c 'pretender doctor || true'".to_string(),
                inject_as: "pretender_doctor".to_string(),
            },
        )]),
    }
}
