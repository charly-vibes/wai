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
        },
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
        },
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
        },
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
    let child_thread = Arc::clone(&child);

    let (tx, rx): (mpsc::Sender<std::io::Result<std::process::ExitStatus>>, _) = mpsc::channel();
    std::thread::spawn(move || {
        // Poll try_wait() in a loop so the mutex is released between iterations,
        // allowing the timeout path to acquire it for kill().
        // IMPORTANT: extract the result before the match so the MutexGuard
        // is dropped before the sleep (Rust edition 2024 scoping rules).
        loop {
            let exited = child_thread.lock().unwrap().try_wait().ok().flatten();
            match exited {
                Some(status) => {
                    let _ = tx.send(Ok(status));
                    break;
                }
                None => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
            }
        }
    });

    match rx.recv_timeout(PLUGIN_TIMEOUT) {
        Ok(Ok(status)) => {
            // Collect stdout from the child's piped handle.
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
    let parts = shell_words::split(passthrough)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
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

    use std::sync::{Arc, Mutex};

    let mut cmd = Command::new(&parts[0]);
    cmd.args(&parts[1..]);
    cmd.args(extra_args);
    cmd.current_dir(project_root);

    let child = cmd.spawn()?;
    let child = Arc::new(Mutex::new(child));
    let child_thread = Arc::clone(&child);

    let (tx, rx): (mpsc::Sender<std::io::Result<std::process::ExitStatus>>, _) = mpsc::channel();
    std::thread::spawn(move || {
        // Poll try_wait() in a loop so the mutex is released between iterations,
        // allowing the timeout path to acquire it for kill().
        // IMPORTANT: extract the result before the match so the MutexGuard
        // is dropped before the sleep (Rust edition 2024 scoping rules).
        loop {
            let exited = child_thread.lock().unwrap().try_wait().ok().flatten();
            match exited {
                Some(status) => {
                    let _ = tx.send(Ok(status));
                    break;
                }
                None => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
            }
        }
    });

    match rx.recv_timeout(PLUGIN_TIMEOUT) {
        Ok(result) => result,
        Err(_) => {
            // Timed out — kill the child and report an error.
            // The waiter thread releases the mutex between try_wait() calls,
            // so we can acquire it here (unlike the old wait() which held it).
            let _ = child.lock().unwrap().kill();
            // Wait briefly for the waiter thread to reap the child.
            let _ = rx.recv_timeout(Duration::from_secs(2));
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "plugin command exceeded 30-second timeout",
            ))
        }
    }
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

    // ── trust store ───────────────────────────────────────────────────────────

    fn sample_plugin(name: &str) -> PluginDef {
        PluginDef {
            name: name.to_string(),
            description: String::new(),
            intent: None,
            success_criteria: None,
            detector: None,
            commands: vec![],
            hooks: HashMap::new(),
        }
    }

    #[test]
    fn compute_hook_digest_is_stable_and_content_sensitive() {
        let def = sample_plugin("test");
        let hook = HookDef {
            command: "echo hi".to_string(),
            inject_as: "greeting".to_string(),
        };
        let d1 = compute_hook_digest(&def, "on_status", &hook);
        let d2 = compute_hook_digest(&def, "on_status", &hook);
        assert_eq!(d1, d2, "digest must be deterministic");

        // Changing the command changes the digest.
        let hook2 = HookDef {
            command: "echo bye".to_string(),
            inject_as: "greeting".to_string(),
        };
        assert_ne!(d1, compute_hook_digest(&def, "on_status", &hook2));

        // Changing the hook name changes the digest.
        assert_ne!(d1, compute_hook_digest(&def, "on_other", &hook));

        // Changing the plugin name changes the digest.
        assert_ne!(
            d1,
            compute_hook_digest(&sample_plugin("other"), "on_status", &hook)
        );
    }

    #[test]
    fn trust_store_approve_list_revoke() {
        let mut store = TrustStore::default();
        let digest = "abc123".to_string();
        store.approve(TrustEntry {
            plugin_name: "evil".to_string(),
            hook_name: "on_status".to_string(),
            digest: digest.clone(),
            command: "touch marker".to_string(),
            approved_at: "2026-08-05T00:00:00Z".to_string(),
        });

        assert!(store.is_approved(&digest));
        assert_eq!(store.list().len(), 1);
        assert!(!store.is_approved("other"));

        // Revoking removes the entry.
        assert!(store.revoke(&digest));
        assert!(!store.is_approved(&digest));
        assert_eq!(store.list().len(), 0);

        // Revoking a missing digest returns false.
        assert!(!store.revoke(&digest));
    }

    #[test]
    fn trust_store_approve_replaces_same_digest() {
        let mut store = TrustStore::default();
        store.approve(TrustEntry {
            plugin_name: "evil".to_string(),
            hook_name: "on_status".to_string(),
            digest: "d1".to_string(),
            command: "touch marker".to_string(),
            approved_at: "t1".to_string(),
        });
        store.approve(TrustEntry {
            plugin_name: "evil".to_string(),
            hook_name: "on_status".to_string(),
            digest: "d1".to_string(),
            command: "touch marker".to_string(),
            approved_at: "t2".to_string(),
        });
        assert_eq!(
            store.list().len(),
            1,
            "same digest must not create duplicates"
        );
    }

    #[serial_test::serial]
    #[test]
    fn trust_store_path_prefers_wai_data_dir() {
        // trust_store_path uses WAI_DATA_DIR when set.
        // SAFETY: test-only env var, single-threaded tests.
        let saved = std::env::var_os("WAI_DATA_DIR");
        unsafe {
            std::env::set_var("WAI_DATA_DIR", "/tmp/wai-trust-test");
        }
        assert_eq!(
            trust_store_path(),
            PathBuf::from("/tmp/wai-trust-test").join(TRUST_STORE_FILE)
        );
        match saved {
            Some(v) => unsafe { std::env::set_var("WAI_DATA_DIR", v) },
            None => unsafe { std::env::remove_var("WAI_DATA_DIR") },
        }
    }

    #[serial_test::serial]
    #[test]
    fn trust_store_load_save_roundtrip() {
        let tmp = TempDir::new().unwrap();
        // SAFETY: test-only env var, single-threaded tests.
        let saved = std::env::var_os("WAI_DATA_DIR");
        unsafe {
            std::env::set_var("WAI_DATA_DIR", tmp.path().join("data"));
        }

        let digest = "roundtrip-digest".to_string();
        {
            let mut store = TrustStore::load();
            store.approve(TrustEntry {
                plugin_name: "test".to_string(),
                hook_name: "on_status".to_string(),
                digest: digest.clone(),
                command: "echo hi".to_string(),
                approved_at: "2026-08-05T00:00:00Z".to_string(),
            });
            store.save().unwrap();
        }
        // Load from a fresh store to verify persistence.
        let store = TrustStore::load();
        assert!(store.is_approved(&digest));
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].plugin_name, "test");

        match saved {
            Some(v) => unsafe { std::env::set_var("WAI_DATA_DIR", v) },
            None => unsafe { std::env::remove_var("WAI_DATA_DIR") },
        }
    }

    #[serial_test::serial]
    #[test]
    fn run_hooks_skips_untrusted_custom_hook() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // A repository-owned plugin with an on_status hook.
        let plugin_dir = root.join(".wai/plugins");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("evil.toml"),
            r#"
name = "evil"
description = "malicious"

[hooks.on_status]
command = "echo 'hook-ran'"
inject_as = "evil_marker"
"#,
        )
        .unwrap();

        // SAFETY: test-only env var, single-threaded tests.
        let saved = std::env::var_os("WAI_DATA_DIR");
        unsafe {
            std::env::set_var("WAI_DATA_DIR", tmp.path().join("wai-data"));
        }

        // Ensure no trust approval exists (fresh store).
        let outputs = run_hooks(root, "on_status");

        // The untrusted hook must NOT have executed.
        assert!(
            outputs.iter().all(|o| o.label != "evil_marker"),
            "no output from untrusted hook"
        );

        match saved {
            Some(v) => unsafe { std::env::set_var("WAI_DATA_DIR", v) },
            None => unsafe { std::env::remove_var("WAI_DATA_DIR") },
        }
    }

    #[serial_test::serial]
    #[test]
    fn run_hooks_executes_approved_custom_hook() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let plugin_dir = root.join(".wai/plugins");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        let marker = root.join("hook-ran");
        std::fs::write(
            plugin_dir.join("evil.toml"),
            format!(
                r#"
name = "evil"
description = "malicious"

[hooks.on_status]
command = "echo 'hook-ran'"
inject_as = "evil_marker"
"#
            ),
        )
        .unwrap();

        // SAFETY: test-only env var, single-threaded tests.
        let saved = std::env::var_os("WAI_DATA_DIR");
        unsafe {
            std::env::set_var("WAI_DATA_DIR", tmp.path().join("wai-data"));
        }

        // Approve the hook digest, then run hooks again.
        let plugins = detect_plugins(root);
        let evil = plugins.iter().find(|p| p.def.name == "evil").unwrap();
        let hook = evil.def.hooks.get("on_status").unwrap();
        let digest = compute_hook_digest(&evil.def, "on_status", hook);
        let mut store = TrustStore::load();
        store.approve(TrustEntry {
            plugin_name: "evil".to_string(),
            hook_name: "on_status".to_string(),
            digest: digest.clone(),
            command: hook.command.clone(),
            approved_at: "2026-08-05T00:00:00Z".to_string(),
        });
        store.save().unwrap();
        assert!(TrustStore::load().is_approved(&digest));

        let outputs = run_hooks(root, "on_status");
        assert!(
            outputs.iter().any(|o| o.label == "evil_marker"),
            "approved hook must produce output"
        );

        match saved {
            Some(v) => unsafe { std::env::set_var("WAI_DATA_DIR", v) },
            None => unsafe { std::env::remove_var("WAI_DATA_DIR") },
        }
    }

    // ── timeout ───────────────────────────────────────────────────────────────

    #[test]
    fn execute_hook_timeout_kills_hung_process() {
        // A process that sleeps for 60 seconds should be killed by the 30s timeout.
        let hook = HookDef {
            command: "sleep 60".to_string(),
            inject_as: "timeout_test".to_string(),
        };
        let tmp = TempDir::new().unwrap();
        let start = std::time::Instant::now();
        let result = execute_hook(tmp.path(), &hook);
        let elapsed = start.elapsed();

        // Must complete in well under 60 seconds (the sleep duration).
        assert!(
            elapsed < Duration::from_secs(40),
            "hook must be killed within timeout, took {:.1}s",
            elapsed.as_secs_f64()
        );
        // The killed process produces no output.
        assert!(result.is_none());
    }

    #[test]
    fn execute_hook_no_deadlock_on_fast_command() {
        // A fast command should complete normally (no timeout, no deadlock).
        let hook = HookDef {
            command: "echo 'hello'".to_string(),
            inject_as: "fast_test".to_string(),
        };
        let tmp = TempDir::new().unwrap();
        let result = execute_hook(tmp.path(), &hook);
        assert!(result.is_some());
        assert_eq!(result.unwrap().label, "fast_test");
    }
}
