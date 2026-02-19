use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::config::{
    agent_config_dir, plugins_dir, projects_dir, wai_dir, ProjectConfig, ARCHIVES_DIR, AREAS_DIR,
    CONFIG_FILE, PLUGINS_DIR, PROJECTS_DIR, RESOURCES_DIR, STATE_FILE,
};
use crate::context::current_context;
use crate::output::print_json;
use crate::plugin;
use crate::state::ProjectState;

use super::require_project;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Serialize)]
struct CheckResult {
    name: String,
    status: Status,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<String>,
}

#[derive(Debug, Serialize)]
struct DoctorPayload {
    checks: Vec<CheckResult>,
    summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
struct Summary {
    pass: usize,
    warn: usize,
    fail: usize,
}

#[derive(Deserialize)]
struct ProjectionsConfig {
    #[serde(default)]
    projections: Vec<ProjectionEntry>,
}

#[derive(Deserialize)]
struct ProjectionEntry {
    target: String,
    strategy: String,
    #[serde(default)]
    sources: Vec<String>,
}

pub fn run() -> Result<()> {
    let project_root = require_project()?;
    let context = current_context();

    let mut checks = Vec::new();
    checks.extend(check_directories(&project_root));
    checks.push(check_config(&project_root));
    checks.extend(check_plugin_tools(&project_root));
    checks.extend(check_agent_config_sync(&project_root));
    checks.extend(check_project_state(&project_root));
    checks.extend(check_custom_plugins(&project_root));
    checks.push(check_agent_instructions(&project_root));

    let summary = Summary {
        pass: checks.iter().filter(|c| c.status == Status::Pass).count(),
        warn: checks.iter().filter(|c| c.status == Status::Warn).count(),
        fail: checks.iter().filter(|c| c.status == Status::Fail).count(),
    };

    if context.json {
        let payload = DoctorPayload {
            checks,
            summary: summary.clone(),
        };
        print_json(&payload)?;
    } else {
        render_human(&checks, &summary)?;
    }

    if summary.fail > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn render_human(checks: &[CheckResult], summary: &Summary) -> Result<()> {
    use cliclack::outro;

    println!();
    println!("  {} Workspace Health", "◆".cyan());
    println!();

    for check in checks {
        let icon = match check.status {
            Status::Pass => "✓".green().to_string(),
            Status::Warn => "!".yellow().to_string(),
            Status::Fail => "✗".red().to_string(),
        };
        println!("  {} {}: {}", icon, check.name.bold(), check.message);
        if let Some(ref fix) = check.fix {
            println!("    {} {}", "→".dimmed(), fix.dimmed());
        }
    }

    println!();
    let summary_line = format!(
        "{} passed, {} warnings, {} failed",
        summary.pass, summary.warn, summary.fail
    );
    if summary.fail > 0 {
        outro(summary_line.red().to_string()).into_diagnostic()?;
    } else if summary.warn > 0 {
        outro(summary_line.yellow().to_string()).into_diagnostic()?;
    } else {
        outro(summary_line.green().to_string()).into_diagnostic()?;
    }

    Ok(())
}

fn check_directories(project_root: &Path) -> Vec<CheckResult> {
    let wai = wai_dir(project_root);
    let expected = [
        PROJECTS_DIR,
        AREAS_DIR,
        RESOURCES_DIR,
        ARCHIVES_DIR,
        PLUGINS_DIR,
    ];

    let mut results = Vec::new();
    let mut missing = Vec::new();

    for dir in &expected {
        if !wai.join(dir).is_dir() {
            missing.push(*dir);
        }
    }

    if missing.is_empty() {
        results.push(CheckResult {
            name: "Directory structure".to_string(),
            status: Status::Pass,
            message: "All PARA directories present".to_string(),
            fix: None,
        });
    } else {
        results.push(CheckResult {
            name: "Directory structure".to_string(),
            status: Status::Fail,
            message: format!("Missing directories: {}", missing.join(", ")),
            fix: Some(format!(
                "Run: mkdir -p {}",
                missing
                    .iter()
                    .map(|d| format!(".wai/{}", d))
                    .collect::<Vec<_>>()
                    .join(" ")
            )),
        });
    }

    results
}

fn check_config(project_root: &Path) -> CheckResult {
    let config_path = wai_dir(project_root).join(CONFIG_FILE);

    if !config_path.exists() {
        return CheckResult {
            name: "Configuration".to_string(),
            status: Status::Fail,
            message: "config.toml not found".to_string(),
            fix: Some("Run: wai init".to_string()),
        };
    }

    match ProjectConfig::load(project_root) {
        Ok(_) => CheckResult {
            name: "Configuration".to_string(),
            status: Status::Pass,
            message: "config.toml is valid".to_string(),
            fix: None,
        },
        Err(e) => CheckResult {
            name: "Configuration".to_string(),
            status: Status::Fail,
            message: format!("config.toml parse error: {}", e),
            fix: Some("Fix the syntax in .wai/config.toml".to_string()),
        },
    }
}

fn check_plugin_tools(project_root: &Path) -> Vec<CheckResult> {
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
            });
        } else {
            results.push(CheckResult {
                name: format!("Plugin tool: {}", plugin_name),
                status: Status::Warn,
                message: format!("`{}` not found in PATH", cli_name),
                fix: Some(format!("Install `{}` or add it to your PATH", cli_name)),
            });
        }
    }

    if results.is_empty() {
        results.push(CheckResult {
            name: "Plugin tools".to_string(),
            status: Status::Pass,
            message: "No detected plugins require CLI tools".to_string(),
            fix: None,
        });
    }

    results
}

fn check_agent_config_sync(project_root: &Path) -> Vec<CheckResult> {
    let config_dir = agent_config_dir(project_root);
    let projections_path = config_dir.join(".projections.yml");
    let mut results = Vec::new();

    if !projections_path.exists() {
        results.push(CheckResult {
            name: "Agent config sync".to_string(),
            status: Status::Warn,
            message: ".projections.yml not found".to_string(),
            fix: Some(
                "Run: wai init (or create .wai/resources/agent-config/.projections.yml)"
                    .to_string(),
            ),
        });
        return results;
    }

    let content = match std::fs::read_to_string(&projections_path) {
        Ok(c) => c,
        Err(e) => {
            results.push(CheckResult {
                name: "Agent config sync".to_string(),
                status: Status::Fail,
                message: format!("Cannot read .projections.yml: {}", e),
                fix: None,
            });
            return results;
        }
    };

    match serde_yaml::from_str::<ProjectionsConfig>(&content) {
        Ok(config) => {
            if config.projections.is_empty() {
                results.push(CheckResult {
                    name: "Agent config sync".to_string(),
                    status: Status::Pass,
                    message: "No projections configured".to_string(),
                    fix: None,
                });
            } else {
                for proj in &config.projections {
                    results.extend(check_projection(project_root, &config_dir, proj));
                }
            }
        }
        Err(e) => {
            results.push(CheckResult {
                name: "Agent config sync".to_string(),
                status: Status::Fail,
                message: format!("Invalid .projections.yml: {}", e),
                fix: Some(
                    "Fix the YAML syntax in .wai/resources/agent-config/.projections.yml"
                        .to_string(),
                ),
            });
        }
    }

    results
}

fn check_projection(
    project_root: &Path,
    config_dir: &Path,
    proj: &ProjectionEntry,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // Check if source directories exist
    for source in &proj.sources {
        let source_path = config_dir.join(source);
        if !source_path.exists() {
            results.push(CheckResult {
                name: format!("Projection source: {}", source),
                status: Status::Warn,
                message: format!("Source directory '{}' not found", source),
                fix: Some("Check .projections.yml sources".to_string()),
            });
        }
    }

    let target_path = project_root.join(&proj.target);

    // Check target exists
    if !target_path.exists() {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Target not synced".to_string(),
            fix: Some("Run: wai sync".to_string()),
        });
        return results;
    }

    // Strategy-specific checks
    match proj.strategy.as_str() {
        "symlink" => {
            results.extend(check_symlink_strategy(
                project_root,
                config_dir,
                proj,
                &target_path,
            ));
        }
        "inline" => {
            results.extend(check_inline_strategy(config_dir, proj, &target_path));
        }
        "reference" => {
            results.extend(check_reference_strategy(config_dir, proj, &target_path));
        }
        _ => {
            // Unknown strategy - just check target exists
            results.push(CheckResult {
                name: format!("Projection → {}", proj.target),
                status: Status::Pass,
                message: format!("Target exists (unknown strategy: {})", proj.strategy),
                fix: None,
            });
        }
    }

    results
}

fn check_symlink_strategy(
    _project_root: &Path,
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let mut has_issues = false;
    let mut broken_count = 0;

    // For symlink strategy, verify each entry is a symlink pointing to correct source
    for source in &proj.sources {
        let source_path = config_dir.join(source);
        if !source_path.exists() || !source_path.is_dir() {
            continue;
        }

        let entries = match std::fs::read_dir(&source_path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let entry_name = entry.file_name();
            let link_path = target_path.join(&entry_name);

            if !link_path.exists() {
                // Broken or missing symlink
                broken_count += 1;
                has_issues = true;
            } else {
                #[cfg(unix)]
                {
                    if let Ok(metadata) = std::fs::symlink_metadata(&link_path) {
                        if !metadata.file_type().is_symlink() {
                            has_issues = true;
                        } else if let Ok(target) = std::fs::read_link(&link_path) {
                            let expected = entry.path();
                            if target != expected {
                                has_issues = true;
                            }
                        } else {
                            has_issues = true;
                        }
                    } else {
                        has_issues = true;
                    }
                }
                #[cfg(not(unix))]
                {
                    // On non-Unix, just check file exists (copy strategy)
                    let _ = entry; // Silence unused variable warning
                }
            }
        }
    }

    if broken_count > 0 {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: format!("Has {} broken symlinks", broken_count),
            fix: Some("Run: wai sync".to_string()),
        });
    } else if has_issues {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Symlink issues detected".to_string(),
            fix: Some("Run: wai sync".to_string()),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
        });
    }

    results
}

fn check_inline_strategy(
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let expected_content = build_inline_content(config_dir, &proj.sources);
    let expected_hash = hash_string(&expected_content);

    let actual_content = match std::fs::read_to_string(target_path) {
        Ok(c) => c,
        Err(_) => {
            results.push(CheckResult {
                name: format!("Projection → {}", proj.target),
                status: Status::Warn,
                message: "Cannot read target file".to_string(),
                fix: Some("Run: wai sync".to_string()),
            });
            return results;
        }
    };
    let actual_hash = hash_string(&actual_content);

    if expected_hash != actual_hash {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Stale (content changed)".to_string(),
            fix: Some("Run: wai sync".to_string()),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
        });
    }

    results
}

fn check_reference_strategy(
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let expected_content = build_reference_content(config_dir, &proj.sources);
    let expected_hash = hash_string(&expected_content);

    let actual_content = match std::fs::read_to_string(target_path) {
        Ok(c) => c,
        Err(_) => {
            results.push(CheckResult {
                name: format!("Projection → {}", proj.target),
                status: Status::Warn,
                message: "Cannot read target file".to_string(),
                fix: Some("Run: wai sync".to_string()),
            });
            return results;
        }
    };
    let actual_hash = hash_string(&actual_content);

    if expected_hash != actual_hash {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Stale (content changed)".to_string(),
            fix: Some("Run: wai sync".to_string()),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
        });
    }

    results
}

fn build_inline_content(config_dir: &Path, sources: &[String]) -> String {
    let mut content = String::from("# Auto-generated by wai — do not edit directly\n\n");

    for source in sources {
        let source_path = config_dir.join(source);
        if source_path.exists() {
            if source_path.is_dir() {
                let mut entries: Vec<_> = std::fs::read_dir(&source_path)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.ok())
                    .collect();
                entries.sort_by_key(|e| e.file_name());

                for entry in entries {
                    if let Ok(file_content) = std::fs::read_to_string(entry.path()) {
                        content.push_str(&format!(
                            "# Source: {}/{}\n",
                            source,
                            entry.file_name().to_str().unwrap_or("?")
                        ));
                        content.push_str(&file_content);
                        content.push_str("\n\n");
                    }
                }
            } else if let Ok(file_content) = std::fs::read_to_string(&source_path) {
                content.push_str(&format!("# Source: {}\n", source));
                content.push_str(&file_content);
                content.push_str("\n\n");
            }
        }
    }

    content
}

fn build_reference_content(_config_dir: &Path, sources: &[String]) -> String {
    let mut content = String::from("# Auto-generated by wai — do not edit directly\n");
    content.push_str("# References to agent config sources:\n\n");

    for source in sources {
        // The config_dir is .wai/resources/agent-config, so we need to construct
        // paths relative to that
        let source_path = _config_dir.join(source);
        if source_path.exists() && source_path.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&source_path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                if let Some(name) = entry.file_name().to_str() {
                    // Format: .wai/resources/agent-config/{source}/{name}
                    content.push_str(&format!(
                        "- .wai/resources/agent-config/{}/{}\n",
                        source, name
                    ));
                }
            }
        }
    }

    content
}

fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn check_project_state(project_root: &Path) -> Vec<CheckResult> {
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
            results.push(CheckResult {
                name: format!("Project state: {}", name),
                status: Status::Warn,
                message: "No .state file found".to_string(),
                fix: Some(format!("Run: wai phase set research (in project {})", name)),
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
                });
            }
            Err(e) => {
                results.push(CheckResult {
                    name: format!("Project state: {}", name),
                    status: Status::Fail,
                    message: format!("Invalid .state: {}", e),
                    fix: Some(format!("Fix or recreate .wai/projects/{}/.state", name)),
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
        });
    }

    results
}

fn check_custom_plugins(project_root: &Path) -> Vec<CheckResult> {
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
            Ok(content) => match serde_yaml::from_str::<plugin::PluginDef>(&content) {
                Ok(_) => {
                    results.push(CheckResult {
                        name: format!("Custom plugin: {}", filename),
                        status: Status::Pass,
                        message: "Valid YAML".to_string(),
                        fix: None,
                    });
                }
                Err(e) => {
                    results.push(CheckResult {
                        name: format!("Custom plugin: {}", filename),
                        status: Status::Fail,
                        message: format!("Invalid plugin config: {}", e),
                        fix: Some(format!("Fix the YAML syntax in .wai/plugins/{}", filename)),
                    });
                }
            },
            Err(e) => {
                results.push(CheckResult {
                    name: format!("Custom plugin: {}", filename),
                    status: Status::Fail,
                    message: format!("Cannot read file: {}", e),
                    fix: None,
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
        });
    }

    results
}

fn check_agent_instructions(project_root: &Path) -> CheckResult {
    use crate::managed_block::has_managed_block;

    let agents_md = project_root.join("AGENTS.md");

    if !agents_md.exists() {
        return CheckResult {
            name: "Agent instructions".to_string(),
            status: Status::Warn,
            message: "AGENTS.md not found — LLMs won't know to use wai".to_string(),
            fix: Some("Run: wai init (to create AGENTS.md with wai instructions)".to_string()),
        };
    }

    if has_managed_block(&agents_md) {
        CheckResult {
            name: "Agent instructions".to_string(),
            status: Status::Pass,
            message: "AGENTS.md contains wai managed block".to_string(),
            fix: None,
        }
    } else {
        CheckResult {
            name: "Agent instructions".to_string(),
            status: Status::Warn,
            message: "AGENTS.md exists but missing wai managed block".to_string(),
            fix: Some("Run: wai init (to inject wai instructions into AGENTS.md)".to_string()),
        }
    }
}
