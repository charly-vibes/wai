use std::path::Path;

use miette::IntoDiagnostic;

use crate::config::{
    AGENT_CONFIG_DIR, ARCHIVES_DIR, AREAS_DIR, CONFIG_FILE, CONTEXT_DIR, PLUGINS_DIR, PROJECTS_DIR,
    ProjectConfig, RESOURCES_DIR, RULES_DIR, SKILLS_DIR, STATE_FILE, plugins_dir, projects_dir,
    wai_dir,
};
use crate::plugin;
use crate::state::ProjectState;
use crate::workspace::ensure_workspace_current;

use super::{CheckResult, Status};

pub(super) fn check_directories(project_root: &Path) -> Vec<CheckResult> {
    let wai = wai_dir(project_root);
    let mut results = Vec::new();
    let mut missing = Vec::new();

    // Check top-level PARA directories
    let para_dirs = [
        PROJECTS_DIR,
        AREAS_DIR,
        RESOURCES_DIR,
        ARCHIVES_DIR,
        PLUGINS_DIR,
    ];

    for dir in &para_dirs {
        if !wai.join(dir).is_dir() {
            missing.push(format!(".wai/{}", dir));
        }
    }

    // Check agent-config subdirectories
    let agent_config = wai.join(RESOURCES_DIR).join(AGENT_CONFIG_DIR);
    let agent_subdirs = [SKILLS_DIR, RULES_DIR, CONTEXT_DIR];

    for subdir in &agent_subdirs {
        if !agent_config.join(subdir).is_dir() {
            missing.push(format!(".wai/resources/agent-config/{}", subdir));
        }
    }

    // Check resource subdirectories
    let resources = wai.join(RESOURCES_DIR);
    let resource_subdirs = ["templates", "patterns"];

    for subdir in &resource_subdirs {
        if !resources.join(subdir).is_dir() {
            missing.push(format!(".wai/resources/{}", subdir));
        }
    }

    // Check default files
    if !wai.join(".gitignore").exists() {
        missing.push(".wai/.gitignore".to_string());
    }

    if !agent_config.join(".projections.yml").exists() {
        missing.push(".wai/resources/agent-config/.projections.yml".to_string());
    }

    if missing.is_empty() {
        results.push(CheckResult {
            name: "Directory structure".to_string(),
            status: Status::Pass,
            message: "All directories and default files present".to_string(),
            fix: None,
            fix_fn: None,
        });
    } else {
        results.push(CheckResult {
            name: "Directory structure".to_string(),
            status: Status::Fail,
            message: format!("Missing: {}", missing.join(", ")),
            fix: Some("Run: wai doctor --fix (or wai init to repair)".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                ensure_workspace_current(project_root)?;
                Ok(())
            })),
        });
    }

    results
}

pub(super) fn check_config(project_root: &Path) -> CheckResult {
    let config_path = wai_dir(project_root).join(CONFIG_FILE);

    if !config_path.exists() {
        return CheckResult {
            name: "Configuration".to_string(),
            status: Status::Fail,
            message: "config.toml not found".to_string(),
            fix: Some("Run: wai init".to_string()),
            fix_fn: None,
        };
    }

    match ProjectConfig::load(project_root) {
        Ok(_) => CheckResult {
            name: "Configuration".to_string(),
            status: Status::Pass,
            message: "config.toml is valid".to_string(),
            fix: None,
            fix_fn: None,
        },
        Err(e) => CheckResult {
            name: "Configuration".to_string(),
            status: Status::Fail,
            message: format!("config.toml parse error: {}", e),
            fix: Some("Fix the syntax in .wai/config.toml".to_string()),
            fix_fn: None,
        },
    }
}

pub(super) fn check_version(project_root: &Path) -> CheckResult {
    let config_path = wai_dir(project_root).join(CONFIG_FILE);

    if !config_path.exists() {
        return CheckResult {
            name: "Version".to_string(),
            status: Status::Pass,
            message: "Skipped (no config.toml)".to_string(),
            fix: None,
            fix_fn: None,
        };
    }

    match ProjectConfig::load(project_root) {
        Ok(config) => {
            let current_version = env!("CARGO_PKG_VERSION");
            let current_commit = env!("WAI_GIT_COMMIT");
            let has_commit_tracking = !current_commit.starts_with("unknown");

            // Commit-based check: fires on every rebuild after a new commit.
            // Takes priority over the version check when commit info is available.
            if has_commit_tracking {
                if config.project.tool_commit.is_empty() {
                    // Old workspace: was initialized before commit tracking was added.
                    return CheckResult {
                        name: "Version".to_string(),
                        status: Status::Warn,
                        message: format!(
                            "Workspace not yet synced to commit tracking (binary: {} {})",
                            current_version, current_commit
                        ),
                        fix: Some("Run: wai init (to sync workspace)".to_string()),
                        fix_fn: Some(Box::new(move |project_root| {
                            ensure_workspace_current(project_root)?;
                            crate::workspace::sync_tool_commit(project_root)?;
                            Ok(())
                        })),
                    };
                }
                if config.project.tool_commit != current_commit {
                    return CheckResult {
                        name: "Version".to_string(),
                        status: Status::Warn,
                        message: format!(
                            "Workspace synced at commit {}, binary is at {} — workspace may be stale",
                            config.project.tool_commit, current_commit
                        ),
                        fix: Some("Run: wai init (or wai doctor --fix)".to_string()),
                        fix_fn: Some(Box::new(move |project_root| {
                            ensure_workspace_current(project_root)?;
                            crate::workspace::sync_tool_commit(project_root)?;
                            Ok(())
                        })),
                    };
                }
                // Commit matches — workspace is current.
                return CheckResult {
                    name: "Version".to_string(),
                    status: Status::Pass,
                    message: format!("Up to date ({} {})", current_version, current_commit),
                    fix: None,
                    fix_fn: None,
                };
            }

            // Fallback: no commit info in binary, compare version strings.
            if config.project.version == current_version {
                CheckResult {
                    name: "Version".to_string(),
                    status: Status::Pass,
                    message: format!("Up to date ({})", current_version),
                    fix: None,
                    fix_fn: None,
                }
            } else {
                CheckResult {
                    name: "Version".to_string(),
                    status: Status::Warn,
                    message: format!(
                        "Config version ({}) differs from binary ({})",
                        config.project.version, current_version
                    ),
                    fix: Some("Run: wai doctor --fix (to update config.toml version)".to_string()),
                    fix_fn: Some(Box::new(move |project_root| {
                        ensure_workspace_current(project_root)?;
                        Ok(())
                    })),
                }
            }
        }
        Err(_) => CheckResult {
            name: "Version".to_string(),
            status: Status::Pass,
            message: "Skipped (invalid config.toml)".to_string(),
            fix: None,
            fix_fn: None,
        },
    }
}

pub(super) fn check_plugin_tools(project_root: &Path) -> Vec<CheckResult> {
    let plugins = plugin::detect_plugins(project_root);
    let mut results = Vec::new();

    let tool_checks: Vec<(&str, &str)> = plugins
        .iter()
        .filter(|p| p.detected)
        .filter_map(|p| match p.def.name.as_str() {
            "git" => Some(("git", "git")),
            "beads" => Some(("beads", "bd")),
            "openspec" => Some(("openspec", "openspec")),
            _ => None,
        })
        .collect();

    for (plugin_name, cli_name) in &tool_checks {
        let available = std::process::Command::new("which")
            .arg(cli_name)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if available {
            results.push(CheckResult {
                name: format!("Plugin tool: {}", plugin_name),
                status: Status::Pass,
                message: format!("`{}` is available", cli_name),
                fix: None,
                fix_fn: None,
            });
        } else {
            results.push(CheckResult {
                name: format!("Plugin tool: {}", plugin_name),
                status: Status::Warn,
                message: format!("`{}` not found in PATH", cli_name),
                fix: Some(format!("Install `{}` or add it to your PATH", cli_name)),
                fix_fn: None,
            });
        }
    }

    if results.is_empty() {
        results.push(CheckResult {
            name: "Plugin tools".to_string(),
            status: Status::Pass,
            message: "No detected plugins require CLI tools".to_string(),
            fix: None,
            fix_fn: None,
        });
    }

    results
}

pub(super) fn check_project_state(project_root: &Path) -> Vec<CheckResult> {
    let proj_dir = projects_dir(project_root);
    let mut results = Vec::new();

    if !proj_dir.exists() {
        return results;
    }

    let entries = match std::fs::read_dir(&proj_dir) {
        Ok(e) => e,
        Err(_) => return results,
    };

    let mut any_project = false;
    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        any_project = true;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let state_path = entry.path().join(STATE_FILE);

        if !state_path.exists() {
            let state_path_clone = state_path.clone();
            results.push(CheckResult {
                name: format!("Project state: {}", name),
                status: Status::Warn,
                message: "No .state file found".to_string(),
                fix: Some(format!("Run: wai phase set research (in project {})", name)),
                fix_fn: Some(Box::new(move |_project_root| {
                    let state = ProjectState::default();
                    state.save(&state_path_clone).into_diagnostic()
                })),
            });
            continue;
        }

        match ProjectState::load(&state_path) {
            Ok(_) => {
                results.push(CheckResult {
                    name: format!("Project state: {}", name),
                    status: Status::Pass,
                    message: "Valid".to_string(),
                    fix: None,
                    fix_fn: None,
                });
            }
            Err(e) => {
                results.push(CheckResult {
                    name: format!("Project state: {}", name),
                    status: Status::Fail,
                    message: format!("Invalid .state: {}", e),
                    fix: Some(format!("Fix or recreate .wai/projects/{}/.state", name)),
                    fix_fn: None,
                });
            }
        }
    }

    if !any_project {
        results.push(CheckResult {
            name: "Project state".to_string(),
            status: Status::Pass,
            message: "No projects to check".to_string(),
            fix: None,
            fix_fn: None,
        });
    }

    results
}

pub(super) fn check_custom_plugins(project_root: &Path) -> Vec<CheckResult> {
    let plugins_path = plugins_dir(project_root);
    let mut results = Vec::new();

    if !plugins_path.exists() {
        return results;
    }

    let entries = match std::fs::read_dir(&plugins_path) {
        Ok(e) => e,
        Err(_) => return results,
    };

    let mut any_yaml = false;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let is_yaml = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e == "yml" || e == "yaml");

        if !is_yaml {
            continue;
        }
        any_yaml = true;

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_yml::from_str::<plugin::PluginDef>(&content) {
                Ok(_) => {
                    results.push(CheckResult {
                        name: format!("Custom plugin: {}", filename),
                        status: Status::Pass,
                        message: "Valid YAML".to_string(),
                        fix: None,
                        fix_fn: None,
                    });
                }
                Err(e) => {
                    results.push(CheckResult {
                        name: format!("Custom plugin: {}", filename),
                        status: Status::Fail,
                        message: format!("Invalid plugin config: {}", e),
                        fix: Some(format!("Fix the YAML syntax in .wai/plugins/{}", filename)),
                        fix_fn: None,
                    });
                }
            },
            Err(e) => {
                results.push(CheckResult {
                    name: format!("Custom plugin: {}", filename),
                    status: Status::Fail,
                    message: format!("Cannot read file: {}", e),
                    fix: None,
                    fix_fn: None,
                });
            }
        }
    }

    if !any_yaml {
        results.push(CheckResult {
            name: "Custom plugins".to_string(),
            status: Status::Pass,
            message: "No custom plugin configs found".to_string(),
            fix: None,
            fix_fn: None,
        });
    }

    results
}

pub(super) fn check_wai_project_env(project_root: &Path) -> Vec<CheckResult> {
    let mut results = Vec::new();

    match std::env::var("WAI_PROJECT") {
        Ok(val) if val.is_empty() => {
            results.push(CheckResult {
                name: "WAI_PROJECT env var".to_string(),
                status: Status::Warn,
                message: "WAI_PROJECT is set to an empty string (treated as unset)".to_string(),
                fix: Some("unset WAI_PROJECT".to_string()),
                fix_fn: None,
            });
        }
        Ok(val) => {
            let proj_dir = projects_dir(project_root).join(&val);
            if proj_dir.exists() {
                results.push(CheckResult {
                    name: "WAI_PROJECT env var".to_string(),
                    status: Status::Pass,
                    message: format!("WAI_PROJECT='{}' points to a valid project", val),
                    fix: None,
                    fix_fn: None,
                });
            } else {
                let available = crate::commands::list_projects(project_root);
                results.push(CheckResult {
                    name: "WAI_PROJECT env var".to_string(),
                    status: Status::Warn,
                    message: format!(
                        "WAI_PROJECT='{}' but project not found. Available: {}",
                        val,
                        if available.is_empty() {
                            "none".to_string()
                        } else {
                            available.join(", ")
                        }
                    ),
                    fix: Some(format!(
                        "unset WAI_PROJECT  # or: export WAI_PROJECT={}",
                        available.first().unwrap_or(&"<name>".to_string())
                    )),
                    fix_fn: None,
                });
            }
        }
        Err(_) => {
            // WAI_PROJECT not set — nothing to check
        }
    }

    results
}

pub(super) fn check_readme_badge(project_root: &Path) -> Vec<CheckResult> {
    use crate::commands::why::{WAI_BADGE_MARKDOWN, content_has_wai_badge};

    let candidates = ["README.md", "README.rst", "README.txt", "README"];
    let readme_path = candidates
        .iter()
        .map(|n| project_root.join(n))
        .find(|p| p.exists());

    let Some(path) = readme_path else {
        // No README — skip this check entirely
        return vec![];
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    if content_has_wai_badge(&content) {
        vec![CheckResult {
            name: "README badge".to_string(),
            status: Status::Pass,
            message: "wai badge present in README".to_string(),
            fix: None,
            fix_fn: None,
        }]
    } else {
        vec![CheckResult {
            name: "README badge".to_string(),
            status: Status::Warn,
            message: "No wai badge in README — add one to show the project uses wai".to_string(),
            fix: Some(format!("Add to README: {}", WAI_BADGE_MARKDOWN)),
            fix_fn: None,
        }]
    }
}
