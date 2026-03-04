use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::config::{
    AGENT_CONFIG_DIR, ARCHIVES_DIR, AREAS_DIR, CONFIG_FILE, CONTEXT_DIR, PLUGINS_DIR, PROJECTS_DIR,
    ProjectConfig, RESOURCES_DIR, RULES_DIR, SKILLS_DIR, STATE_FILE, agent_config_dir, plugins_dir,
    projects_dir, wai_dir,
};
use crate::context::current_context;
use crate::output::print_json;
use crate::plugin;
use crate::state::ProjectState;
use crate::workspace::ensure_workspace_current;

use super::require_project;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pass,
    Warn,
    Fail,
}

#[derive(Serialize)]
struct CheckResult {
    name: String,
    status: Status,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<String>,
    #[serde(skip)]
    #[allow(clippy::type_complexity)]
    fix_fn: Option<Box<dyn FnOnce(&Path) -> Result<()>>>,
}

#[derive(Serialize)]
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

pub fn run(fix: bool) -> Result<()> {
    let project_root = require_project()?;
    let context = current_context();

    let mut checks = Vec::new();
    checks.extend(check_directories(&project_root));
    checks.push(check_config(&project_root));
    checks.push(check_version(&project_root));
    checks.extend(check_plugin_tools(&project_root));
    checks.extend(check_agent_config_sync(&project_root));
    checks.extend(check_skills_in_repo(&project_root));
    checks.extend(check_agent_tool_coverage(&project_root));
    checks.extend(check_project_state(&project_root));
    checks.extend(check_custom_plugins(&project_root));
    checks.extend(check_agent_instructions(&project_root));
    checks.extend(check_readme_badge(&project_root));
    checks.extend(check_claude_session_hook());

    let summary = Summary {
        pass: checks.iter().filter(|c| c.status == Status::Pass).count(),
        warn: checks.iter().filter(|c| c.status == Status::Warn).count(),
        fail: checks.iter().filter(|c| c.status == Status::Fail).count(),
    };

    // Handle fix mode vs diagnostic mode
    if fix {
        // In fix mode, show diagnostics first (if human mode), then apply fixes
        if !context.json {
            render_human(&checks, &summary)?;
        }
        apply_fixes(&project_root, checks, &context)?;
    } else {
        // In diagnostic mode, just show results
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
    }

    Ok(())
}

fn apply_fixes(
    project_root: &Path,
    mut checks: Vec<CheckResult>,
    context: &crate::context::CliContext,
) -> Result<()> {
    use crate::error::WaiError;

    // Refuse in safe mode
    if context.safe {
        return Err(WaiError::SafeModeViolation {
            action: "apply doctor fixes".to_string(),
        }
        .into());
    }

    // Filter to fixable checks
    let fixable_checks: Vec<CheckResult> =
        checks.drain(..).filter(|c| c.fix_fn.is_some()).collect();

    if fixable_checks.is_empty() {
        if !context.json {
            use cliclack::log;
            log::info("No fixable issues found").into_diagnostic()?;
        }
        return Ok(());
    }

    // Confirm (unless --yes, --no-input, or --json)
    let should_apply = if context.json || context.no_input || context.yes {
        true
    } else {
        use cliclack::confirm;
        confirm(format!("Apply {} fix(es)?", fixable_checks.len()))
            .interact()
            .into_diagnostic()?
    };

    if !should_apply {
        if !context.json {
            use cliclack::log;
            log::warning("Fixes cancelled").into_diagnostic()?;
        }
        return Ok(());
    }

    // Apply fixes
    let mut fixes_applied = Vec::new();
    let mut fixes_failed = Vec::new();

    for mut check in fixable_checks {
        if let Some(fix_fn) = check.fix_fn.take() {
            match fix_fn(project_root) {
                Ok(()) => {
                    fixes_applied.push(FixResult {
                        name: check.name.clone(),
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    fixes_failed.push(FixResult {
                        name: check.name.clone(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    // Check if there were failures (before moving the vectors)
    let had_failures = !fixes_failed.is_empty();

    // Output results
    if context.json {
        #[derive(Serialize)]
        struct FixPayload {
            fixes_applied: Vec<FixResult>,
            fixes_failed: Vec<FixResult>,
        }

        let payload = FixPayload {
            fixes_applied,
            fixes_failed,
        };
        print_json(&payload)?;
    } else {
        use cliclack::log;
        println!();
        for fix in &fixes_applied {
            log::success(format!("Fixed: {}", fix.name)).into_diagnostic()?;
        }
        for fix in &fixes_failed {
            log::error(format!(
                "Failed to fix {}: {}",
                fix.name,
                fix.error.as_ref().unwrap_or(&"unknown error".to_string())
            ))
            .into_diagnostic()?;
        }
        println!();

        if had_failures {
            use cliclack::outro;
            outro("Some fixes failed. Re-run 'wai doctor' to check status.").into_diagnostic()?;
        } else {
            use cliclack::outro;
            outro("All fixes applied. Re-run 'wai doctor' to verify.").into_diagnostic()?;
        }
    }

    // Exit with appropriate code
    if had_failures {
        std::process::exit(1);
    }

    Ok(())
}

#[derive(Serialize)]
struct FixResult {
    name: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn render_human(checks: &[CheckResult], summary: &Summary) -> Result<()> {
    use cliclack::outro;

    println!();
    println!("  {} Workspace Health", "◆".cyan());
    println!("  {} For repo hygiene and agent workflow conventions, run 'wai way'", "·".dimmed());
    println!();

    for check in checks {
        let icon = match check.status {
            Status::Pass => "✓".green().to_string(),
            Status::Warn => "⚠".yellow().to_string(),
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

fn check_config(project_root: &Path) -> CheckResult {
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

fn check_version(project_root: &Path) -> CheckResult {
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
            fix_fn: None,
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
                fix_fn: None,
            });
            return results;
        }
    };

    match serde_yml::from_str::<ProjectionsConfig>(&content) {
        Ok(config) => {
            if config.projections.is_empty() {
                results.push(CheckResult {
                    name: "Agent config sync".to_string(),
                    status: Status::Pass,
                    message: "No projections configured".to_string(),
                    fix: None,
                    fix_fn: None,
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
                fix_fn: None,
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
                fix_fn: None,
            });
        }
    }

    let target_path = project_root.join(&proj.target);

    // Check target exists
    if !target_path.exists() {
        let config_dir_clone = config_dir.to_path_buf();
        let sync_proj = crate::sync_core::Projection {
            target: proj.target.clone(),
            strategy: proj.strategy.clone(),
            sources: proj.sources.clone(),
        };
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Target not synced".to_string(),
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                match sync_proj.strategy.as_str() {
                    "symlink" => crate::sync_core::execute_symlink(
                        project_root,
                        &config_dir_clone,
                        &sync_proj,
                    ),
                    "inline" => crate::sync_core::execute_inline(
                        project_root,
                        &config_dir_clone,
                        &sync_proj,
                    ),
                    "reference" => crate::sync_core::execute_reference(
                        project_root,
                        &config_dir_clone,
                        &sync_proj,
                    ),
                    _ => Ok(()),
                }
            })),
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
                fix_fn: None,
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

    // Also scan the target directory for any broken symlinks (e.g. source file deleted
    // after sync, leaving a dangling symlink with no corresponding source entry).
    #[cfg(unix)]
    if target_path.exists()
        && let Ok(entries) = std::fs::read_dir(target_path)
    {
        for entry in entries.filter_map(|e| e.ok()) {
            let link_path = entry.path();
            if let Ok(meta) = std::fs::symlink_metadata(&link_path)
                && meta.file_type().is_symlink()
                && !link_path.exists()
            {
                broken_count += 1;
                has_issues = true;
            }
        }
    }

    if broken_count > 0 || has_issues {
        let config_dir_clone = config_dir.to_path_buf();
        let sync_proj = crate::sync_core::Projection {
            target: proj.target.clone(),
            strategy: proj.strategy.clone(),
            sources: proj.sources.clone(),
        };
        let message = if broken_count > 0 {
            format!("Has {} broken symlinks", broken_count)
        } else {
            "Symlink issues detected".to_string()
        };
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message,
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                crate::sync_core::execute_symlink(project_root, &config_dir_clone, &sync_proj)
            })),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
            fix_fn: None,
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
            let config_dir_clone = config_dir.to_path_buf();
            let sync_proj = crate::sync_core::Projection {
                target: proj.target.clone(),
                strategy: proj.strategy.clone(),
                sources: proj.sources.clone(),
            };
            results.push(CheckResult {
                name: format!("Projection → {}", proj.target),
                status: Status::Warn,
                message: "Cannot read target file".to_string(),
                fix: Some("Run: wai sync".to_string()),
                fix_fn: Some(Box::new(move |project_root| {
                    crate::sync_core::execute_inline(project_root, &config_dir_clone, &sync_proj)
                })),
            });
            return results;
        }
    };
    let actual_hash = hash_string(&actual_content);

    if expected_hash != actual_hash {
        let config_dir_clone = config_dir.to_path_buf();
        let sync_proj = crate::sync_core::Projection {
            target: proj.target.clone(),
            strategy: proj.strategy.clone(),
            sources: proj.sources.clone(),
        };
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Stale (content changed)".to_string(),
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                crate::sync_core::execute_inline(project_root, &config_dir_clone, &sync_proj)
            })),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
            fix_fn: None,
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
            let config_dir_clone = config_dir.to_path_buf();
            let sync_proj = crate::sync_core::Projection {
                target: proj.target.clone(),
                strategy: proj.strategy.clone(),
                sources: proj.sources.clone(),
            };
            results.push(CheckResult {
                name: format!("Projection → {}", proj.target),
                status: Status::Warn,
                message: "Cannot read target file".to_string(),
                fix: Some("Run: wai sync".to_string()),
                fix_fn: Some(Box::new(move |project_root| {
                    crate::sync_core::execute_reference(project_root, &config_dir_clone, &sync_proj)
                })),
            });
            return results;
        }
    };
    let actual_hash = hash_string(&actual_content);

    if expected_hash != actual_hash {
        let config_dir_clone = config_dir.to_path_buf();
        let sync_proj = crate::sync_core::Projection {
            target: proj.target.clone(),
            strategy: proj.strategy.clone(),
            sources: proj.sources.clone(),
        };
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Stale (content changed)".to_string(),
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                crate::sync_core::execute_reference(project_root, &config_dir_clone, &sync_proj)
            })),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
            fix_fn: None,
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

fn check_readme_badge(project_root: &Path) -> Vec<CheckResult> {
    use super::why::{WAI_BADGE_MARKDOWN, content_has_wai_badge};

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

fn check_claude_session_hook() -> Vec<CheckResult> {
    let settings_path = match dirs::home_dir() {
        Some(home) => home.join(".claude").join("settings.json"),
        None => {
            return vec![CheckResult {
                name: "Claude Code session hook".to_string(),
                status: Status::Warn,
                message: "Could not determine home directory".to_string(),
                fix: None,
                fix_fn: None,
            }];
        }
    };

    if !settings_path.exists() {
        return vec![CheckResult {
            name: "Claude Code session hook".to_string(),
            status: Status::Warn,
            message: "~/.claude/settings.json not found — Claude Code may not be installed"
                .to_string(),
            fix: None,
            fix_fn: None,
        }];
    }

    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(e) => {
            return vec![CheckResult {
                name: "Claude Code session hook".to_string(),
                status: Status::Warn,
                message: format!("Cannot read ~/.claude/settings.json: {}", e),
                fix: None,
                fix_fn: None,
            }];
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return vec![CheckResult {
                name: "Claude Code session hook".to_string(),
                status: Status::Warn,
                message: format!("~/.claude/settings.json is not valid JSON: {}", e),
                fix: None,
                fix_fn: None,
            }];
        }
    };

    // Check whether any SessionStart hook command contains "wai status"
    let has_hook = json
        .get("hooks")
        .and_then(|h| h.get("SessionStart"))
        .and_then(|s| s.as_array())
        .map(|entries| {
            entries.iter().any(|entry| {
                entry
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|hook| {
                            hook.get("command")
                                .and_then(|c| c.as_str())
                                .map(|cmd| cmd.contains("wai status"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if has_hook {
        vec![CheckResult {
            name: "Claude Code session hook".to_string(),
            status: Status::Pass,
            message: "`wai status` is in the SessionStart hook".to_string(),
            fix: None,
            fix_fn: None,
        }]
    } else {
        vec![CheckResult {
            name: "Claude Code session hook".to_string(),
            status: Status::Warn,
            message: "`wai status` not found in ~/.claude/settings.json SessionStart hooks"
                .to_string(),
            fix: Some(
                r#"Add to ~/.claude/settings.json hooks.SessionStart: {"matcher":"","hooks":[{"type":"command","command":"wai status 2>/dev/null || true"}]}"#
                    .to_string(),
            ),
            fix_fn: Some(Box::new(move |_project_root| {
                use miette::IntoDiagnostic;

                let content = std::fs::read_to_string(&settings_path).into_diagnostic()?;
                let mut json: serde_json::Value =
                    serde_json::from_str(&content).into_diagnostic()?;

                let new_hook = serde_json::json!({
                    "matcher": "",
                    "hooks": [{"type": "command", "command": "wai status 2>/dev/null || true"}]
                });

                // Ensure hooks.SessionStart exists as an array, then push
                let session_start = json
                    .get_mut("hooks")
                    .and_then(|h| h.get_mut("SessionStart"))
                    .and_then(|s| s.as_array_mut());

                if let Some(arr) = session_start {
                    arr.push(new_hook);
                } else {
                    // Build hooks.SessionStart from scratch, preserving other hooks
                    let hooks = json
                        .get_mut("hooks")
                        .and_then(|h| h.as_object_mut());

                    if let Some(hooks_obj) = hooks {
                        hooks_obj.insert(
                            "SessionStart".to_string(),
                            serde_json::json!([new_hook]),
                        );
                    } else {
                        json["hooks"] = serde_json::json!({
                            "SessionStart": [new_hook]
                        });
                    }
                }

                let updated = serde_json::to_string_pretty(&json).into_diagnostic()?;
                std::fs::write(&settings_path, updated).into_diagnostic()?;
                Ok(())
            })),
        }]
    }
}

/// Known agent tool directories: (dir name, display name)
const AGENT_TOOL_DIRS: &[(&str, &str)] = &[
    (".agents", "Agents"),
    (".amp", "Amp"),
    (".claude", "Claude Code"),
    (".cursor", "Cursor"),
    (".gemini", "Gemini CLI"),
];

/// Find SKILL.md files outside `.wai/` and agent tool directories, and report any not yet
/// imported into wai. Agent tool directories (.claude, .amp, .gemini, .cursor) are excluded
/// because they hold synced copies of skills, not source definitions.
fn check_skills_in_repo(project_root: &Path) -> Vec<CheckResult> {
    use walkdir::WalkDir;

    let wai_path = project_root.join(".wai");
    let target_path = project_root.join("target");
    let git_path = project_root.join(".git");
    // Exclude agent tool dirs — those contain synced copies, not source definitions
    let agent_tool_paths: Vec<std::path::PathBuf> = AGENT_TOOL_DIRS
        .iter()
        .map(|(dir, _)| project_root.join(dir))
        .collect();
    let skills_dir = agent_config_dir(project_root).join(SKILLS_DIR);

    // Walk repo, skip managed/build dirs and agent tool dirs
    let external_skills: Vec<std::path::PathBuf> = WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            !p.starts_with(&wai_path)
                && !p.starts_with(&target_path)
                && !p.starts_with(&git_path)
                && !agent_tool_paths.iter().any(|ap| p.starts_with(ap))
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "SKILL.md" && e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    if external_skills.is_empty() {
        return vec![];
    }

    // Collect skill directory names already managed by wai
    let imported: HashSet<String> = if skills_dir.exists() {
        std::fs::read_dir(&skills_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().join("SKILL.md").exists())
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .collect()
    } else {
        HashSet::new()
    };

    let mut unimported: Vec<String> = Vec::new();
    for skill_path in &external_skills {
        if let Some(parent) = skill_path.parent()
            && let Some(dir_name) = parent.file_name().and_then(|n| n.to_str())
            && !imported.contains(dir_name)
        {
            let rel = skill_path
                .strip_prefix(project_root)
                .unwrap_or(skill_path)
                .display()
                .to_string();
            unimported.push(rel);
        }
    }

    if unimported.is_empty() {
        vec![CheckResult {
            name: "Skills import".to_string(),
            status: Status::Pass,
            message: format!(
                "{} SKILL.md file(s) found outside wai — all imported",
                external_skills.len()
            ),
            fix: None,
            fix_fn: None,
        }]
    } else {
        vec![CheckResult {
            name: "Skills import".to_string(),
            status: Status::Warn,
            message: format!(
                "{} SKILL.md file(s) found outside wai but not imported: {}",
                unimported.len(),
                unimported.join(", ")
            ),
            fix: Some(
                "Copy each skill to .wai/resources/agent-config/skills/<name>/SKILL.md".to_string(),
            ),
            fix_fn: None,
        }]
    }
}

/// Check that detected agent tool directories (.claude, .amp, .gemini, .cursor) are covered by
/// projections, and that wai skills are synced to them.
fn check_agent_tool_coverage(project_root: &Path) -> Vec<CheckResult> {
    let config_dir = agent_config_dir(project_root);
    let projections_path = config_dir.join(".projections.yml");
    let skills_dir = config_dir.join(SKILLS_DIR);

    // Does wai manage any skills?
    let has_skills = skills_dir.exists()
        && std::fs::read_dir(&skills_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.path().join("SKILL.md").exists())
            })
            .unwrap_or(false);

    // Which known agent tool directories exist at the project root?
    let detected: Vec<(&str, &str)> = AGENT_TOOL_DIRS
        .iter()
        .filter(|(dir, _)| project_root.join(dir).is_dir())
        .copied()
        .collect();

    if detected.is_empty() {
        return vec![];
    }

    // Load projections, distinguishing three cases:
    //   None  → file missing or unreadable/unparseable (coverage check should warn)
    //   Some(vec) with items → projections configured (coverage check applies)
    //   Some(empty vec) → user explicitly set projections: [] (suppress coverage warnings)
    let projections_opt: Option<Vec<ProjectionEntry>> = if projections_path.exists() {
        std::fs::read_to_string(&projections_path)
            .ok()
            .and_then(|c| serde_yml::from_str::<ProjectionsConfig>(&c).ok())
            .map(|cfg| cfg.projections)
    } else {
        None
    };

    // If the projections file exists and is explicitly empty, the user has intentionally
    // opted out of projections — skip the per-directory coverage warnings entirely.
    if let Some(ref p) = projections_opt {
        if p.is_empty() {
            return vec![];
        }
    }

    let projections = projections_opt.unwrap_or_default();

    let mut results = Vec::new();

    for (tool_dir, tool_name) in &detected {
        // Projections that target this tool dir or a sub-path of it
        let covering: Vec<&ProjectionEntry> = projections
            .iter()
            .filter(|p| p.target == *tool_dir || p.target.starts_with(&format!("{}/", tool_dir)))
            .collect();

        if covering.is_empty() {
            results.push(CheckResult {
                name: format!("Agent tool projection: {}", tool_name),
                status: Status::Warn,
                message: format!(
                    "{} directory detected but not in .projections.yml",
                    tool_dir
                ),
                fix: Some(format!(
                    "Add a projection for {} in .wai/resources/agent-config/.projections.yml",
                    tool_dir
                )),
                fix_fn: None,
            });
        } else if has_skills {
            let skills_synced = covering.iter().any(|p| {
                p.sources
                    .iter()
                    .any(|s| s == SKILLS_DIR || s.ends_with(&format!("/{}", SKILLS_DIR)))
            });
            if skills_synced {
                results.push(CheckResult {
                    name: format!("Agent tool projection: {}", tool_name),
                    status: Status::Pass,
                    message: format!("{} projected with skills synced", tool_dir),
                    fix: None,
                    fix_fn: None,
                });
            } else {
                results.push(CheckResult {
                    name: format!("Agent tool projection: {}", tool_name),
                    status: Status::Warn,
                    message: format!(
                        "{} projected but skills source not included — wai skills won't sync to {}",
                        tool_dir, tool_name
                    ),
                    fix: Some(format!(
                        "Add 'skills' to sources for the {} projection in .projections.yml",
                        tool_dir
                    )),
                    fix_fn: None,
                });
            }
        } else {
            results.push(CheckResult {
                name: format!("Agent tool projection: {}", tool_name),
                status: Status::Pass,
                message: format!("{} has a projection defined", tool_dir),
                fix: None,
                fix_fn: None,
            });
        }
    }

    results
}

/// Returns true if the WAI managed block in `path` already mentions the ro5 skill.
/// Used to detect a stale block when the ro5 skill was installed after the last `wai init`.
fn managed_block_mentions_ro5(path: &std::path::Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let wai_start = "<!-- WAI:START -->";
    let wai_end = "<!-- WAI:END -->";
    if let (Some(start), Some(end)) = (content.find(wai_start), content.find(wai_end)) {
        content[start..end].contains("/ro5")
    } else {
        false
    }
}

fn check_agent_instructions(project_root: &Path) -> Vec<CheckResult> {
    use crate::managed_block::has_managed_block;
    use crate::workspace::detect_installed_skill_names;

    let mut results = Vec::new();

    let skill_names = detect_installed_skill_names(project_root);
    let has_ro5_skill = skill_names.iter().any(|s| {
        s == "ro5" || s == "rule-of-5" || s == "rule-of-5-universal"
    });

    // Check AGENTS.md
    let agents_md = project_root.join("AGENTS.md");
    if !agents_md.exists() {
        results.push(CheckResult {
            name: "Agent instructions: AGENTS.md".to_string(),
            status: Status::Warn,
            message: "AGENTS.md not found — LLMs won't know to use wai".to_string(),
            fix: Some("Run: wai init (to create AGENTS.md with wai instructions)".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                use crate::managed_block::inject_managed_block;
                let agents_md = project_root.join("AGENTS.md");
                let plugins = plugin::detect_plugins(project_root);
                let plugin_names: Vec<&str> = plugins
                    .iter()
                    .filter(|p| p.detected)
                    .map(|p| p.def.name.as_str())
                    .collect();
                let skill_names = detect_installed_skill_names(project_root);
                let skill_name_refs: Vec<&str> =
                    skill_names.iter().map(|s| s.as_str()).collect();
                inject_managed_block(&agents_md, &plugin_names, &skill_name_refs)
                    .into_diagnostic()?;
                Ok(())
            })),
        });
    } else if has_managed_block(&agents_md) {
        if has_ro5_skill && !managed_block_mentions_ro5(&agents_md) {
            results.push(CheckResult {
                name: "Agent instructions: AGENTS.md".to_string(),
                status: Status::Warn,
                message: "Managed block is stale: ro5 skill installed but not reflected"
                    .to_string(),
                fix: Some(
                    "Run: wai init (to regenerate managed block with ro5 reminders)".to_string(),
                ),
                fix_fn: Some(Box::new(move |project_root| {
                    use crate::managed_block::inject_managed_block;
                    let agents_md = project_root.join("AGENTS.md");
                    let plugins = plugin::detect_plugins(project_root);
                    let plugin_names: Vec<&str> = plugins
                        .iter()
                        .filter(|p| p.detected)
                        .map(|p| p.def.name.as_str())
                        .collect();
                    let skill_names = detect_installed_skill_names(project_root);
                    let skill_name_refs: Vec<&str> =
                        skill_names.iter().map(|s| s.as_str()).collect();
                    inject_managed_block(&agents_md, &plugin_names, &skill_name_refs)
                        .into_diagnostic()?;
                    Ok(())
                })),
            });
        } else {
            results.push(CheckResult {
                name: "Agent instructions: AGENTS.md".to_string(),
                status: Status::Pass,
                message: "Contains wai managed block".to_string(),
                fix: None,
                fix_fn: None,
            });
        }
    } else {
        results.push(CheckResult {
            name: "Agent instructions: AGENTS.md".to_string(),
            status: Status::Warn,
            message: "Exists but missing wai managed block".to_string(),
            fix: Some("Run: wai init (to inject wai instructions into AGENTS.md)".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                use crate::managed_block::inject_managed_block;
                let agents_md = project_root.join("AGENTS.md");
                let plugins = plugin::detect_plugins(project_root);
                let plugin_names: Vec<&str> = plugins
                    .iter()
                    .filter(|p| p.detected)
                    .map(|p| p.def.name.as_str())
                    .collect();
                let skill_names = detect_installed_skill_names(project_root);
                let skill_name_refs: Vec<&str> =
                    skill_names.iter().map(|s| s.as_str()).collect();
                inject_managed_block(&agents_md, &plugin_names, &skill_name_refs)
                    .into_diagnostic()?;
                Ok(())
            })),
        });
    }

    // Check CLAUDE.md
    let claude_md = project_root.join("CLAUDE.md");
    if !claude_md.exists() {
        results.push(CheckResult {
            name: "Agent instructions: CLAUDE.md".to_string(),
            status: Status::Warn,
            message: "CLAUDE.md not found — Claude Code won't know to use wai".to_string(),
            fix: Some("Run: wai init (to create CLAUDE.md with wai instructions)".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                use crate::managed_block::inject_managed_block;
                let claude_md = project_root.join("CLAUDE.md");
                let plugins = plugin::detect_plugins(project_root);
                let plugin_names: Vec<&str> = plugins
                    .iter()
                    .filter(|p| p.detected)
                    .map(|p| p.def.name.as_str())
                    .collect();
                let skill_names = detect_installed_skill_names(project_root);
                let skill_name_refs: Vec<&str> =
                    skill_names.iter().map(|s| s.as_str()).collect();
                inject_managed_block(&claude_md, &plugin_names, &skill_name_refs)
                    .into_diagnostic()?;
                Ok(())
            })),
        });
    } else if has_managed_block(&claude_md) {
        if has_ro5_skill && !managed_block_mentions_ro5(&claude_md) {
            results.push(CheckResult {
                name: "Agent instructions: CLAUDE.md".to_string(),
                status: Status::Warn,
                message: "Managed block is stale: ro5 skill installed but not reflected"
                    .to_string(),
                fix: Some(
                    "Run: wai init (to regenerate managed block with ro5 reminders)".to_string(),
                ),
                fix_fn: Some(Box::new(move |project_root| {
                    use crate::managed_block::inject_managed_block;
                    let claude_md = project_root.join("CLAUDE.md");
                    let plugins = plugin::detect_plugins(project_root);
                    let plugin_names: Vec<&str> = plugins
                        .iter()
                        .filter(|p| p.detected)
                        .map(|p| p.def.name.as_str())
                        .collect();
                    let skill_names = detect_installed_skill_names(project_root);
                    let skill_name_refs: Vec<&str> =
                        skill_names.iter().map(|s| s.as_str()).collect();
                    inject_managed_block(&claude_md, &plugin_names, &skill_name_refs)
                        .into_diagnostic()?;
                    Ok(())
                })),
            });
        } else {
            results.push(CheckResult {
                name: "Agent instructions: CLAUDE.md".to_string(),
                status: Status::Pass,
                message: "Contains wai managed block".to_string(),
                fix: None,
                fix_fn: None,
            });
        }
    } else {
        results.push(CheckResult {
            name: "Agent instructions: CLAUDE.md".to_string(),
            status: Status::Warn,
            message: "Exists but missing wai managed block".to_string(),
            fix: Some("Run: wai init (to inject wai instructions into CLAUDE.md)".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                use crate::managed_block::inject_managed_block;
                let claude_md = project_root.join("CLAUDE.md");
                let plugins = plugin::detect_plugins(project_root);
                let plugin_names: Vec<&str> = plugins
                    .iter()
                    .filter(|p| p.detected)
                    .map(|p| p.def.name.as_str())
                    .collect();
                let skill_names = detect_installed_skill_names(project_root);
                let skill_name_refs: Vec<&str> =
                    skill_names.iter().map(|s| s.as_str()).collect();
                inject_managed_block(&claude_md, &plugin_names, &skill_name_refs)
                    .into_diagnostic()?;
                Ok(())
            })),
        });
    }

    results
}
