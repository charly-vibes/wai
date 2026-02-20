use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::config::{
    AGENT_CONFIG_DIR, ARCHIVES_DIR, AREAS_DIR, CONFIG_DIR, CONTEXT_DIR, PLUGINS_DIR, PROJECTS_DIR,
    ProjectConfig, RESOURCES_DIR, RULES_DIR, SKILLS_DIR,
};
use crate::managed_block::inject_managed_block;
use crate::plugin;

/// Actions taken during workspace repair/initialization
#[derive(Debug, Clone)]
pub struct WorkspaceAction {
    pub description: String,
}

impl WorkspaceAction {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
        }
    }
}

/// Ensures the workspace is current by creating/repairing all expected artifacts:
/// - PARA directories (projects, areas, resources, archives, plugins)
/// - agent-config subdirectories (skills, rules, context)
/// - resource subdirectories (templates, patterns)
/// - default files (.gitignore, .projections.yml, PLUGINS.md)
/// - config.toml version update
/// - managed block injection in agent instruction files
///
/// Returns a list of actions taken for reporting.
pub fn ensure_workspace_current(project_root: &Path) -> Result<Vec<WorkspaceAction>> {
    let mut actions = Vec::new();

    // Create PARA directories
    let wai_dir = project_root.join(CONFIG_DIR);
    let para_dirs = [
        PROJECTS_DIR,
        AREAS_DIR,
        RESOURCES_DIR,
        ARCHIVES_DIR,
        PLUGINS_DIR,
    ];

    for dir in &para_dirs {
        let dir_path = wai_dir.join(dir);
        if !dir_path.exists() {
            std::fs::create_dir_all(&dir_path).into_diagnostic()?;
            actions.push(WorkspaceAction::new(format!("Created .wai/{}", dir)));
        }
    }

    // Create agent-config subdirectories
    let agent_config = wai_dir.join(RESOURCES_DIR).join(AGENT_CONFIG_DIR);
    let agent_subdirs = [SKILLS_DIR, RULES_DIR, CONTEXT_DIR];

    for subdir in &agent_subdirs {
        let subdir_path = agent_config.join(subdir);
        if !subdir_path.exists() {
            std::fs::create_dir_all(&subdir_path).into_diagnostic()?;
            actions.push(WorkspaceAction::new(format!(
                "Created .wai/resources/agent-config/{}",
                subdir
            )));
        }
    }

    // Create resource subdirectories
    let resources = wai_dir.join(RESOURCES_DIR);
    let resource_subdirs = ["templates", "patterns"];

    for subdir in &resource_subdirs {
        let subdir_path = resources.join(subdir);
        if !subdir_path.exists() {
            std::fs::create_dir_all(&subdir_path).into_diagnostic()?;
            actions.push(WorkspaceAction::new(format!(
                "Created .wai/resources/{}",
                subdir
            )));
        }
    }

    // Create .projections.yml if missing
    let projections_path = agent_config.join(".projections.yml");
    if !projections_path.exists() {
        let projections_content = "# Agent config projections — defines how configs are synced to tool-specific locations\n\
            # Strategies: symlink, inline, reference\n\
            projections: []\n";
        std::fs::write(&projections_path, projections_content).into_diagnostic()?;
        actions.push(WorkspaceAction::new(
            "Created .wai/resources/agent-config/.projections.yml".to_string(),
        ));
    }

    // Create .gitignore for .wai if missing
    let gitignore_path = wai_dir.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, "# Local-only files\n*.local\n*.cache\n")
            .into_diagnostic()?;
        actions.push(WorkspaceAction::new("Created .wai/.gitignore".to_string()));
    }

    // Update config.toml version if it exists
    let config_path = wai_dir.join("config.toml");
    if config_path.exists() {
        match ProjectConfig::load(project_root) {
            Ok(mut config) => {
                let current_version = env!("CARGO_PKG_VERSION");
                if config.project.version != current_version {
                    config.project.version = current_version.to_string();
                    config.save(project_root)?;
                    actions.push(WorkspaceAction::new(format!(
                        "Updated config.toml version to {}",
                        current_version
                    )));
                }
            }
            Err(_) => {
                // Config exists but is invalid - don't modify it
            }
        }
    }

    // Detect plugins for managed block injection
    let plugins = plugin::detect_plugins(project_root);
    let detected: Vec<&str> = plugins
        .iter()
        .filter(|p| p.detected)
        .map(|p| p.def.name.as_str())
        .collect();

    // Create/update PLUGINS.md
    if !detected.is_empty() {
        let plugins_md = create_plugins_readme(&detected);
        let plugins_path = wai_dir.join("PLUGINS.md");

        // Only write if it doesn't exist or content has changed
        let should_write = if plugins_path.exists() {
            match std::fs::read_to_string(&plugins_path) {
                Ok(existing) => existing != plugins_md,
                Err(_) => true,
            }
        } else {
            true
        };

        if should_write {
            std::fs::write(&plugins_path, &plugins_md).into_diagnostic()?;
            if plugins_path.exists() {
                actions.push(WorkspaceAction::new("Updated .wai/PLUGINS.md".to_string()));
            } else {
                actions.push(WorkspaceAction::new("Created .wai/PLUGINS.md".to_string()));
            }
        }
    }

    // Inject/update managed blocks in agent instruction files
    let agent_files = ["AGENTS.md", "CLAUDE.md"];
    for filename in &agent_files {
        let path = project_root.join(filename);

        // For AGENTS.md, always ensure it exists. For CLAUDE.md, only if it already exists.
        if filename == &"AGENTS.md" || path.exists() {
            match inject_managed_block(&path, &detected) {
                Ok(result) => {
                    actions.push(WorkspaceAction::new(result.description(filename)));
                }
                Err(e) => {
                    // Log but don't fail - this is a best-effort operation
                    eprintln!("Warning: Failed to update {}: {}", filename, e);
                }
            }
        }
    }

    Ok(actions)
}

/// Creates the content for PLUGINS.md based on detected plugins
fn create_plugins_readme(detected: &[&str]) -> String {
    let mut readme = String::from(
        "# Plugins\n\n\
        Wai auto-detects and integrates with external tools:\n\n\
        ## Detected in this workspace:\n",
    );

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
        "\n\n\
        ## Custom plugins\n\n\
        Add YAML files to `.wai/plugins/` to register custom plugins:\n\n\
        ```yaml\n\
        name: my-tool\n\
        description: Integration with my-tool\n\
        detector:\n\
          type: directory\n\
          path: .my-tool\n\
        commands:\n\
          - name: list\n\
            description: List my-tool items\n\
            passthrough: my-tool list\n\
            read_only: true\n\
        hooks:\n\
          on_status:\n\
            command: my-tool status\n\
            inject_as: tool_status\n\
        ```\n\n\
        Run `wai plugin list` to see all available plugins.\n",
    );

    readme
}
