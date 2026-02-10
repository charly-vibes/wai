use cliclack::{input, intro, log, outro};
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

    intro("Initialize wai project").into_diagnostic()?;

    // Check if already initialized
    if config_dir.exists() {
        log::warning("Project already initialized in this directory").into_diagnostic()?;
        outro("Use 'wai status' to see project info").into_diagnostic()?;
        return Ok(());
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
                input("Project name?")
                    .default_input(&default_name)
                    .interact()
                    .into_diagnostic()?
            }
        }
    };

    // Create config
    let config = ProjectConfig {
        project: ProjectSettings {
            name: project_name.clone(),
            version: "0.1.0".to_string(),
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

    log::success("Created .wai/ directory with PARA structure").into_diagnostic()?;

    if !detected.is_empty() {
        log::info(format!("Detected plugins: {}", detected.join(", "))).into_diagnostic()?;
    }

    log::info("Next steps:").into_diagnostic()?;
    println!("  → wai new project \"my-app\"    Create your first project");
    println!("  → wai status                   Check project status");

    outro(format!("Workspace '{}' initialized!", project_name)).into_diagnostic()?;
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
