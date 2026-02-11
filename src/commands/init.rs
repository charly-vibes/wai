use cliclack::input;
use miette::{IntoDiagnostic, Result};

use crate::config::{
    AGENT_CONFIG_DIR, ARCHIVES_DIR, AREAS_DIR, CONFIG_DIR, CONTEXT_DIR, PLUGINS_DIR, PROJECTS_DIR,
    ProjectConfig, ProjectSettings, RESOURCES_DIR, RULES_DIR, SKILLS_DIR,
};
use crate::context::current_context;

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

    // Check if already initialized
    if config_dir.exists() {
        println!("┌  Initialize wai project");
        println!("▲  Project already initialized in this directory");

        // Still inject/update managed block in agent instruction files
        inject_agent_instructions(&current_dir)?;

        println!("└  Use 'wai status' to see project info");
        return Ok(());
    }

    println!("┌  Initialize wai project");

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
    let config = ProjectConfig {
        project: ProjectSettings {
            name: project_name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: String::new(),
        },
        plugins: vec![],
    };

    // Create directory structure
    create_para_structure(&current_dir, &config)?;

    // Auto-detect plugins
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

    println!("◆  Created .wai/ directory with PARA structure");

    // Create plugin configuration files
    setup_plugins(&current_dir, &detected)?;

    if !detected.is_empty() {
        println!("✓ Detected plugins: {}", detected.join(", "));
    }

    // Inject managed block into agent instruction files
    inject_agent_instructions(&current_dir)?;

    println!("●  Next steps:");
    println!("  → wai new project \"my-app\"    Create your first project");
    println!("  → wai status                   Check project status");
    if !detected.is_empty() {
        println!("  → wai plugin list              View detected plugins");
    }

    println!("└  Workspace '{}' initialized!", project_name);
    Ok(())
}

fn setup_plugins(root: &std::path::Path, detected: &[&str]) -> Result<()> {
    use crate::config::CONFIG_DIR;
    let config_dir = root.join(CONFIG_DIR);

    // Create README for plugin system
    let plugins_info = r#"# Plugins

Wai auto-detects and integrates with external tools:

## Detected in this workspace:
"#;

    let mut readme = plugins_info.to_string();

    if detected.contains(&"git") {
        readme.push_str("- **git** — Version control (hooks: status, handoff)\n");
    }
    if detected.contains(&"beads") {
        readme.push_str("- **beads** — Issue tracking (commands: list, show, ready)\n");
    }
    if detected.contains(&"openspec") {
        readme.push_str("- **openspec** — Specification management\n");
    }

    readme.push_str(
        r#"

## Custom plugins

Add YAML files to `.wai/plugins/` to register custom plugins:

```yaml
name: my-tool
description: Integration with my-tool
detector:
  type: directory
  path: .my-tool
commands:
  - name: list
    description: List my-tool items
    passthrough: my-tool list
    read_only: true
hooks:
  on_status:
    command: my-tool status
    inject_as: tool_status
```

Run `wai plugin list` to see all available plugins.
"#,
    );

    std::fs::write(config_dir.join("PLUGINS.md"), readme).into_diagnostic()?;

    Ok(())
}

fn inject_agent_instructions(root: &std::path::Path) -> Result<()> {
    use crate::managed_block::inject_managed_block;

    let agent_files = ["AGENTS.md", "CLAUDE.md"];
    for filename in &agent_files {
        let path = root.join(filename);
        if filename == &"AGENTS.md" || path.exists() {
            match inject_managed_block(&path) {
                Ok(result) => println!("✓ {}", result.description(filename)),
                Err(e) => {
                    eprintln!("⚠ Failed to update {}: {}", filename, e);
                }
            }
        }
    }
    Ok(())
}

fn create_para_structure(root: &std::path::Path, config: &ProjectConfig) -> Result<()> {
    let wai_dir = root.join(CONFIG_DIR);

    // Create PARA directories
    std::fs::create_dir_all(wai_dir.join(PROJECTS_DIR)).into_diagnostic()?;
    std::fs::create_dir_all(wai_dir.join(AREAS_DIR)).into_diagnostic()?;
    std::fs::create_dir_all(wai_dir.join(RESOURCES_DIR)).into_diagnostic()?;
    std::fs::create_dir_all(wai_dir.join(ARCHIVES_DIR)).into_diagnostic()?;
    std::fs::create_dir_all(wai_dir.join(PLUGINS_DIR)).into_diagnostic()?;

    // Create agent-config structure
    let agent_config = wai_dir.join(RESOURCES_DIR).join(AGENT_CONFIG_DIR);
    std::fs::create_dir_all(agent_config.join(SKILLS_DIR)).into_diagnostic()?;
    std::fs::create_dir_all(agent_config.join(RULES_DIR)).into_diagnostic()?;
    std::fs::create_dir_all(agent_config.join(CONTEXT_DIR)).into_diagnostic()?;

    // Create default .projections.yml
    let projections_content = "# Agent config projections — defines how configs are synced to tool-specific locations\n\
        # Strategies: symlink, inline, reference\n\
        projections: []\n";
    std::fs::write(agent_config.join(".projections.yml"), projections_content).into_diagnostic()?;

    // Create resource subdirectories
    std::fs::create_dir_all(wai_dir.join(RESOURCES_DIR).join("templates")).into_diagnostic()?;
    std::fs::create_dir_all(wai_dir.join(RESOURCES_DIR).join("patterns")).into_diagnostic()?;

    // Save config
    config.save(root)?;

    // Create .gitignore for .wai if needed
    let gitignore_path = wai_dir.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(gitignore_path, "# Local-only files\n*.local\n*.cache\n")
            .into_diagnostic()?;
    }

    Ok(())
}
