use cliclack::input;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::config::{CONFIG_DIR, LlmConfig, ProjectConfig, ProjectSettings};
use crate::context::{CliContext, current_context};
use crate::workspace::{ensure_workspace_current, sync_tool_commit};

/// Register wai in the shared genesis tool manifest (.genesis/tools.toml).
fn register_in_genesis_discovery(project_root: &Path) {
    // Best-effort: silently skip if genesis dep is absent or write fails.
    let _ = genesis::discovery::register(
        project_root,
        "wai",
        "Workflow manager for AI-assisted development",
        "directory",
        ".wai",
    );
}

pub fn run(name: Option<String>) -> Result<()> {
    let current_dir = std::env::current_dir().into_diagnostic()?;
    let config_dir = current_dir.join(CONFIG_DIR);
    let context = current_context();

    if context.safe {
        return Err(crate::error::WaiError::SafeModeViolation {
            action: "initialize project".to_string(),
        }
        .into());
    }

    let quiet = context.quiet;
    let already_initialized = config_dir.exists();
    if !quiet && !context.json {
        println!("┌  Initialize wai project");
        if already_initialized {
            println!("▲  Project already initialized in this directory");
        }
    }

    let detected = detect_plugins_present(&current_dir);

    if already_initialized {
        return reinit_existing(&current_dir, &context, &name, detected);
    }

    init_fresh(&current_dir, &context, name)
}

/// Detect presence markers of known plugins in the workspace.
fn detect_plugins_present(dir: &Path) -> Vec<&'static str> {
    let mut detected = Vec::new();
    if dir.join(".beads").exists() {
        detected.push("beads");
    }
    if dir.join("openspec").exists() {
        detected.push("openspec");
    }
    if dir.join(".git").exists() {
        detected.push("git");
    }
    detected
}

/// Re-init path: repair/update an already-initialized workspace.
fn reinit_existing(
    current_dir: &Path,
    context: &CliContext,
    name: &Option<String>,
    detected: Vec<&'static str>,
) -> Result<()> {
    let quiet = context.quiet;
    // Refuse to bless a workspace whose config cannot be parsed: ok:true
    // here would tell a hint-blind agent the project is fixed when
    // `.wai/config.toml` is corrupt (evallerina-00n). Repair is doctor's
    // channel — the emitted error envelope points at `wai doctor`.
    // A missing config file is different: `.wai/` present but config
    // missing is a half-initialized workspace where re-creating the
    // config is init's legitimate job (genesis_fixture recovery path).
    let existing_config = match ProjectConfig::load(current_dir) {
        Ok(config) => Some(config),
        Err(e @ crate::error::WaiError::ConfigError { .. }) => return Err(e.into()),
        Err(_) => None,
    };

    // For re-init, repair/update workspace using shared function.
    // sync_tool_commit is called explicitly here (and only here) so that
    // config.toml is only stamped during intentional init commands, not on
    // every wai invocation.
    let mut actions = ensure_workspace_current(current_dir)?;
    if let Some(action) = sync_tool_commit(current_dir)? {
        actions.push(action);
    }

    // Re-register in genesis tool manifest (ensures .genesis/tools.toml is up to date)
    register_in_genesis_discovery(current_dir);

    if !quiet {
        if context.json {
            print_reinit_envelope(existing_config, name, detected)?;
        } else {
            // Report actions taken
            for action in &actions {
                println!("✓ {}", action.description);
            }

            if actions.is_empty() {
                println!("✓ Workspace is up to date");
            }

            println!("└  Use 'wai status' to see project info");
        }
    }
    Ok(())
}

/// Emit the JSON envelope for the re-init path.
fn print_reinit_envelope(
    existing_config: Option<ProjectConfig>,
    name: &Option<String>,
    detected: Vec<&'static str>,
) -> Result<()> {
    let existing_name = existing_config
        .map(|c| c.project.name)
        .unwrap_or_else(|| name.clone().unwrap_or_default());
    let payload = crate::json::InitPayload {
        project_name: existing_name,
        already_initialized: true,
        detected_plugins: detected.iter().map(|s| s.to_string()).collect(),
        suggestions: vec![
            crate::json::Suggestion {
                label: "Sync agent configs".to_string(),
                command: "wai sync".to_string(),
            },
            crate::json::Suggestion {
                label: "Check project status".to_string(),
                command: "wai status".to_string(),
            },
            crate::json::Suggestion {
                label: "Check workspace health".to_string(),
                command: "wai doctor".to_string(),
            },
            crate::json::Suggestion {
                label: "See workflow conventions and available skills".to_string(),
                command: "wai way".to_string(),
            },
        ],
    };
    crate::output::print_envelope_ok(&payload)
}

/// Fresh-init path: create config and workspace, register, auto-commit.
fn init_fresh(current_dir: &Path, context: &CliContext, name: Option<String>) -> Result<()> {
    let quiet = context.quiet;
    let project_name = resolve_project_name(current_dir, context, name)?;

    create_project_config(current_dir, &project_name)?;
    let actions = ensure_workspace_current(current_dir)?;
    register_in_genesis_discovery(current_dir);
    let git_committed = try_git_commit(current_dir);

    // Auto-detect plugins for final message
    let plugins = crate::plugin::detect_plugins(current_dir);
    let detected: Vec<&str> = plugins
        .iter()
        .filter(|p| p.detected)
        .map(|p| p.def.name.as_str())
        .collect();

    if !quiet {
        if context.json {
            print_fresh_envelope(&project_name, &detected)?;
        } else {
            print_fresh_summary(&project_name, &actions, git_committed, &detected);
        }
    }
    Ok(())
}

/// Prompt (or derive) the project name for a fresh init.
fn resolve_project_name(
    current_dir: &Path,
    context: &CliContext,
    name: Option<String>,
) -> Result<String> {
    if let Some(n) = name {
        return Ok(n);
    }
    if context.no_input && !context.yes {
        return Err(crate::error::WaiError::NonInteractive {
            message: "Project name is required for wai init".to_string(),
        }
        .into());
    }
    let default_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();
    if context.yes {
        return Ok(default_name);
    }
    // Try to use cliclack, fall back to println if terminal not available
    match input("Project name?")
        .default_input(&default_name)
        .interact()
        .into_diagnostic()
    {
        Ok(name) => Ok(name),
        Err(_) => {
            // Terminal not available, use default
            println!("Using default project name: {}", default_name);
            Ok(default_name)
        }
    }
}

/// Create and save the initial project config (creates .wai directory).
fn create_project_config(current_dir: &Path, project_name: &str) -> Result<()> {
    let tool_commit = env!("WAI_GIT_COMMIT");
    let config = ProjectConfig {
        project: ProjectSettings {
            name: project_name.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: String::new(),
            tool_commit: if tool_commit.starts_with("unknown") {
                String::new()
            } else {
                tool_commit.to_string()
            },
        },
        plugins: vec![],
        llm: Some(LlmConfig::default()),
        why: None,
        pipeline: None,
    };
    config.save(current_dir)?;
    Ok(())
}

/// Best-effort `git add .wai/ && git commit` when inside a repo.
fn try_git_commit(current_dir: &Path) -> bool {
    if !current_dir.join(".git").exists() {
        return false;
    }
    let add_ok = std::process::Command::new("git")
        .args(["add", ".wai/"])
        .current_dir(current_dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !add_ok {
        return false;
    }
    std::process::Command::new("git")
        .args(["commit", "-m", "chore: init wai workspace"])
        .current_dir(current_dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Emit the JSON envelope for a fresh init.
fn print_fresh_envelope(project_name: &str, detected: &[&str]) -> Result<()> {
    let mut suggestions = vec![
        crate::json::Suggestion {
            label: "Sync agent configs".to_string(),
            command: "wai sync".to_string(),
        },
        crate::json::Suggestion {
            label: "Create your first project".to_string(),
            command: "wai new project \"my-app\"".to_string(),
        },
        crate::json::Suggestion {
            label: "Check project status".to_string(),
            command: "wai status".to_string(),
        },
        crate::json::Suggestion {
            label: "Check workspace health".to_string(),
            command: "wai doctor".to_string(),
        },
        crate::json::Suggestion {
            label: "See workflow conventions and available skills".to_string(),
            command: "wai way".to_string(),
        },
    ];
    if !detected.is_empty() {
        suggestions.push(crate::json::Suggestion {
            label: "View detected plugins".to_string(),
            command: "wai plugin list".to_string(),
        });
    }
    let payload = crate::json::InitPayload {
        project_name: project_name.to_string(),
        already_initialized: false,
        detected_plugins: detected.iter().map(|s| s.to_string()).collect(),
        suggestions,
    };
    crate::output::print_envelope_ok(&payload)
}

/// Print the human-readable summary for a fresh init.
fn print_fresh_summary(
    project_name: &str,
    actions: &[crate::workspace::WorkspaceAction],
    git_committed: bool,
    detected: &[&str],
) {
    println!("◆  Created .wai/ directory with PARA structure");

    for action in actions {
        if action.description.contains("Created") {
            // Only print creation actions, not updates
            println!("✓ {}", action.description);
        }
    }

    if git_committed {
        println!("✓ Committed .wai/ to git");
    }

    if !detected.is_empty() {
        println!("✓ Detected plugins: {}", detected.join(", "));
    }

    println!("●  Next steps:");
    println!("  → wai sync                     Sync agent configs to tool-specific locations");
    println!("  → wai new project \"my-app\"    Create your first project");
    println!("  → wai status                   Check project status");
    println!("  → wai doctor                   Check workspace health");
    println!("  → wai way                      See workflow conventions and available skills");
    if !detected.is_empty() {
        println!("  → wai plugin list              View detected plugins");
    }

    println!("└  Workspace '{}' initialized!", project_name);
}
