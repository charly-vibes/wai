use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::config::{
    AGENT_CONFIG_DIR, ARCHIVES_DIR, AREAS_DIR, CONFIG_DIR, CONTEXT_DIR, PLUGINS_DIR, PROJECTS_DIR,
    ProjectConfig, RESOURCES_DIR, RULES_DIR, SKILLS_DIR, agent_config_dir,
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

/// Updates `version` and `tool_commit` in `config.toml` to match the current binary.
///
/// This is intentionally separated from `ensure_workspace_current` so that the
/// version stamp is only written during explicit user commands (`wai init`).
/// Automatic per-command checks must NOT call this function, because writing
/// `config.toml` on every run silently dirtifies the git working tree after a
/// binary upgrade.
///
/// Returns an action description when the config was updated, or `None` when it
/// was already up to date or when config could not be loaded.
pub fn sync_tool_commit(project_root: &Path) -> Result<Option<WorkspaceAction>> {
    let wai_dir = project_root.join(crate::config::CONFIG_DIR);
    let config_path = wai_dir.join("config.toml");
    if !config_path.exists() {
        return Ok(None);
    }
    match ProjectConfig::load(project_root) {
        Ok(mut config) => {
            let current_version = env!("CARGO_PKG_VERSION");
            let current_commit = env!("WAI_GIT_COMMIT");
            let version_changed = config.project.version != current_version;
            let commit_changed = !current_commit.starts_with("unknown")
                && config.project.tool_commit != current_commit;
            if version_changed || commit_changed {
                config.project.version = current_version.to_string();
                if !current_commit.starts_with("unknown") {
                    config.project.tool_commit = current_commit.to_string();
                }
                config.save(project_root)?;
                Ok(Some(WorkspaceAction::new(format!(
                    "Updated workspace to wai {} ({})",
                    current_version, current_commit
                ))))
            } else {
                Ok(None)
            }
        }
        Err(_) => {
            // Config exists but is invalid — don't modify it.
            Ok(None)
        }
    }
}

/// Returns the names of skills installed in the project's agent-config skills directory.
/// Scans `.wai/resources/agent-config/skills/` for subdirectories containing a `SKILL.md`.
pub fn detect_installed_skill_names(project_root: &Path) -> Vec<String> {
    let skills_dir = agent_config_dir(project_root).join(SKILLS_DIR);
    if !skills_dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&skills_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect()
}

/// Ensures the workspace is current by creating/repairing all expected artifacts:
/// - PARA directories (projects, areas, resources, archives, plugins)
/// - agent-config subdirectories (skills, rules, context)
/// - resource subdirectories (templates, patterns)
/// - default files (.gitignore, .projections.yml, PLUGINS.md)
/// - managed block injection in agent instruction files
///
/// This function does NOT update `tool_commit` or `version` in `config.toml`.
/// Call `sync_tool_commit` explicitly from commands that are meant to stamp the
/// workspace (currently only `wai init`).
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

    // Detect plugins for managed block injection
    let plugins = plugin::detect_plugins(project_root);
    let detected: Vec<&str> = plugins
        .iter()
        .filter(|p| p.detected)
        .map(|p| p.def.name.as_str())
        .collect();

    // Detect installed skills for managed block injection
    let skill_names = detect_installed_skill_names(project_root);
    let skill_name_refs: Vec<&str> = skill_names.iter().map(|s| s.as_str()).collect();

    // Create/update PLUGINS.md
    if !detected.is_empty() {
        let plugins_md = create_plugins_readme(&detected);
        let plugins_path = wai_dir.join("PLUGINS.md");

        // Only write if it doesn't exist or content has changed
        let already_existed = plugins_path.exists();
        let should_write = if already_existed {
            match std::fs::read_to_string(&plugins_path) {
                Ok(existing) => existing != plugins_md,
                Err(_) => true,
            }
        } else {
            true
        };

        if should_write {
            std::fs::write(&plugins_path, &plugins_md).into_diagnostic()?;
            if already_existed {
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

        // For AGENTS.md and CLAUDE.md, always ensure they exist.
        if filename == &"AGENTS.md" || filename == &"CLAUDE.md" || path.exists() {
            match inject_managed_block(&path, &detected, &skill_name_refs) {
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
