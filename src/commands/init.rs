use cliclack::input;
use miette::{IntoDiagnostic, Result};

use crate::config::{CONFIG_DIR, ProjectConfig, ProjectSettings, WhyConfig};
use crate::context::current_context;
use crate::workspace::{ensure_workspace_current, sync_tool_commit};

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

    // Check if already initialized
    let already_initialized = config_dir.exists();
    if !quiet {
        if already_initialized {
            println!("┌  Initialize wai project");
            println!("▲  Project already initialized in this directory");
        } else {
            println!("┌  Initialize wai project");
        }
    }

    // Detect plugins even on re-init so managed block stays current
    let mut detected = Vec::new();
    if current_dir.join(".beads").exists() {
        detected.push("beads");
    }
    if current_dir.join("openspec").exists() {
        detected.push("openspec");
    }
    if current_dir.join(".git").exists() {
        detected.push("git");
    }

    if already_initialized {
        // For re-init, repair/update workspace using shared function.
        // sync_tool_commit is called explicitly here (and only here) so that
        // config.toml is only stamped during intentional init commands, not on
        // every wai invocation.
        let mut actions = ensure_workspace_current(&current_dir)?;
        if let Some(action) = sync_tool_commit(&current_dir)? {
            actions.push(action);
        }

        if !quiet {
            // Report actions taken
            for action in &actions {
                println!("✓ {}", action.description);
            }

            if actions.is_empty() {
                println!("✓ Workspace is up to date");
            }

            println!("└  Use 'wai status' to see project info");
        }
        return Ok(());
    }

    if !quiet {
        println!("┌  Initialize wai project");
    }

    // Get project name
    let project_name = match name {
        Some(n) => n,
        None => {
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
                default_name
            } else {
                // Try to use cliclack, fall back to println if terminal not available
                match input("Project name?")
                    .default_input(&default_name)
                    .interact()
                    .into_diagnostic()
                {
                    Ok(name) => name,
                    Err(_) => {
                        // Terminal not available, use default
                        println!("Using default project name: {}", default_name);
                        default_name
                    }
                }
            }
        }
    };

    // Create config
    let tool_commit = env!("WAI_GIT_COMMIT");
    let config = ProjectConfig {
        project: ProjectSettings {
            name: project_name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: String::new(),
            tool_commit: if tool_commit.starts_with("unknown") {
                String::new()
            } else {
                tool_commit.to_string()
            },
        },
        plugins: vec![],
        why: Some(WhyConfig::default()),
    };

    // Save config (creates .wai directory)
    config.save(&current_dir)?;

    // Create/repair all workspace artifacts using shared function
    let actions = ensure_workspace_current(&current_dir)?;

    // Auto-detect plugins for final message
    let plugins = crate::plugin::detect_plugins(&current_dir);
    let detected: Vec<&str> = plugins
        .iter()
        .filter(|p| p.detected)
        .map(|p| p.def.name.as_str())
        .collect();

    if !quiet {
        println!("◆  Created .wai/ directory with PARA structure");

        // Report actions taken
        for action in &actions {
            if action.description.contains("Created") {
                // Only print creation actions, not updates
                println!("✓ {}", action.description);
            }
        }

        if !detected.is_empty() {
            println!("✓ Detected plugins: {}", detected.join(", "));
        }

        println!("●  Next steps:");
        println!("  → wai new project \"my-app\"    Create your first project");
        println!("  → wai status                   Check project status");
        if !detected.is_empty() {
            println!("  → wai plugin list              View detected plugins");
        }

        println!("└  Workspace '{}' initialized!", project_name);
    }
    Ok(())
}
