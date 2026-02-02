use cliclack::{intro, log, outro, input};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

use crate::config::{ProjectConfig, ProjectSettings, CONFIG_DIR};

pub fn run(name: Option<String>) -> Result<()> {
    let current_dir = std::env::current_dir().into_diagnostic()?;
    let config_dir = current_dir.join(CONFIG_DIR);

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
            let default_name = current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("my-project")
                .to_string();

            input("Project name?")
                .default_input(&default_name)
                .interact()
                .into_diagnostic()?
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
    create_project_structure(&current_dir, &config)?;

    log::success(format!("Created .wai/ directory")).into_diagnostic()?;
    log::info("Next steps:").into_diagnostic()?;
    println!("  → wai new bead \"First feature\"  Create your first work unit");
    println!("  → wai status                    Check project status");

    outro(format!("Project '{}' initialized!", project_name)).into_diagnostic()?;
    Ok(())
}

fn create_project_structure(root: &PathBuf, config: &ProjectConfig) -> Result<()> {
    let para_dir = root.join(CONFIG_DIR);

    // Create .wai directory and subdirectories
    std::fs::create_dir_all(para_dir.join("beads")).into_diagnostic()?;
    std::fs::create_dir_all(para_dir.join("research")).into_diagnostic()?;
    std::fs::create_dir_all(para_dir.join("plugins")).into_diagnostic()?;

    // Save config
    config.save(root)?;

    // Create .gitignore for .wai if needed
    let gitignore_path = para_dir.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(
            gitignore_path,
            "# Local-only files\n*.local\n*.cache\n",
        ).into_diagnostic()?;
    }

    Ok(())
}
