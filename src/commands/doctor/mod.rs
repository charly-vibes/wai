use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

use crate::config::{SKILLS_DIR, agent_config_dir, projects_dir};
use crate::context::current_context;
use crate::output::print_json;
use crate::plugin;

use super::require_project;

mod checks_basic;
mod checks_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Status {
    Pass,
    Warn,
    Fail,
}

pub(super) struct CheckResult {
    name: String,
    status: Status,
    message: String,
    fix: Option<String>,
    #[allow(clippy::type_complexity)]
    fix_fn: Option<Box<dyn FnOnce(&Path) -> Result<()>>>,
}

impl Serialize for CheckResult {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let field_count = 3 + self.fix.is_some() as usize;
        let mut s = serializer.serialize_struct("CheckResult", field_count)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("status", &self.status)?;
        s.serialize_field("message", &self.message)?;
        if let Some(ref fix) = self.fix {
            s.serialize_field("fix", fix)?;
        }
        s.end()
    }
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

pub fn run(fix: bool) -> Result<()> {
    let project_root = require_project()?;
    let context = current_context();

    let mut checks = Vec::new();
    checks.extend(checks_basic::check_directories(&project_root));
    checks.push(checks_basic::check_config(&project_root));
    checks.push(checks_basic::check_version(&project_root));
    checks.extend(checks_basic::check_plugin_tools(&project_root));
    checks.extend(checks_sync::check_agent_config_sync(&project_root));
    checks.extend(check_skills_in_repo(&project_root));
    checks.extend(check_agent_tool_coverage(&project_root));
    checks.extend(checks_basic::check_project_state(&project_root));
    checks.extend(checks_basic::check_custom_plugins(&project_root));
    checks.extend(check_agent_instructions(&project_root));
    checks.extend(check_managed_block_staleness(&project_root));
    checks.extend(check_pipeline_definitions(&project_root));
    checks.extend(checks_basic::check_readme_badge(&project_root));
    checks.extend(check_claude_session_hook());
    checks.extend(checks_basic::check_wai_project_env(&project_root));
    checks.extend(check_artifact_locks(&project_root));

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
    println!(
        "  {} For repo hygiene and agent workflow conventions, run 'wai way'",
        "·".dimmed()
    );
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
    let projections_opt: Option<Vec<checks_sync::ProjectionEntry>> = if projections_path.exists() {
        std::fs::read_to_string(&projections_path)
            .ok()
            .and_then(|c| serde_yml::from_str::<checks_sync::ProjectionsConfig>(&c).ok())
            .map(|cfg| cfg.projections)
    } else {
        None
    };

    // If the projections file exists and is explicitly empty, the user has intentionally
    // opted out of projections — skip the per-directory coverage warnings entirely.
    if let Some(ref p) = projections_opt
        && p.is_empty()
    {
        return vec![];
    }

    let projections = projections_opt.unwrap_or_default();

    let mut results = Vec::new();

    for (tool_dir, tool_name) in &detected {
        // Projections that target this tool dir or a sub-path of it
        let covering: Vec<&checks_sync::ProjectionEntry> = projections
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
    let has_ro5_skill = skill_names
        .iter()
        .any(|s| s == "ro5" || s == "rule-of-5" || s == "rule-of-5-universal");

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
                let skill_name_refs: Vec<&str> = skill_names.iter().map(|s| s.as_str()).collect();
                inject_managed_block(
                    &agents_md,
                    &plugin_names,
                    &skill_name_refs,
                    &crate::workspace::detect_installed_pipelines(project_root),
                )
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
                    inject_managed_block(
                        &agents_md,
                        &plugin_names,
                        &skill_name_refs,
                        &crate::workspace::detect_installed_pipelines(project_root),
                    )
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
                let skill_name_refs: Vec<&str> = skill_names.iter().map(|s| s.as_str()).collect();
                inject_managed_block(
                    &agents_md,
                    &plugin_names,
                    &skill_name_refs,
                    &crate::workspace::detect_installed_pipelines(project_root),
                )
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
                let skill_name_refs: Vec<&str> = skill_names.iter().map(|s| s.as_str()).collect();
                inject_managed_block(
                    &claude_md,
                    &plugin_names,
                    &skill_name_refs,
                    &crate::workspace::detect_installed_pipelines(project_root),
                )
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
                    inject_managed_block(
                        &claude_md,
                        &plugin_names,
                        &skill_name_refs,
                        &crate::workspace::detect_installed_pipelines(project_root),
                    )
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
                let skill_name_refs: Vec<&str> = skill_names.iter().map(|s| s.as_str()).collect();
                inject_managed_block(
                    &claude_md,
                    &plugin_names,
                    &skill_name_refs,
                    &crate::workspace::detect_installed_pipelines(project_root),
                )
                .into_diagnostic()?;
                Ok(())
            })),
        });
    }

    results
}

/// Check managed block staleness by comparing generated vs actual content.
fn check_managed_block_staleness(project_root: &Path) -> Vec<CheckResult> {
    use crate::managed_block::{read_managed_block, wai_block_content, wai_detailed_content};
    use crate::workspace::{detect_installed_pipelines, detect_installed_skill_names};

    let mut results = Vec::new();

    let plugins = plugin::detect_plugins(project_root);
    let plugin_names: Vec<&str> = plugins
        .iter()
        .filter(|p| p.detected)
        .map(|p| p.def.name.as_str())
        .collect();
    let skill_names = detect_installed_skill_names(project_root);
    let skill_name_refs: Vec<&str> = skill_names.iter().map(|s| s.as_str()).collect();
    let installed_pipelines = detect_installed_pipelines(project_root);

    // Check root CLAUDE.md / AGENTS.md against slim block
    let expected = wai_block_content(
        project_root,
        &plugin_names,
        &skill_name_refs,
        &installed_pipelines,
    );

    for filename in &["CLAUDE.md", "AGENTS.md"] {
        let path = project_root.join(filename);
        if let Some(actual) = read_managed_block(&path)
            && actual != expected
        {
            results.push(CheckResult {
                name: format!("Managed block staleness: {}", filename),
                status: Status::Warn,
                message: format!(
                    "{} managed block outdated — run 'wai init --update' to refresh",
                    filename
                ),
                fix: Some("Run: wai init".to_string()),
                fix_fn: None,
            });
        }
    }

    // Check .wai/AGENTS.md against detailed content
    let detailed_path = project_root.join(".wai").join("AGENTS.md");
    if detailed_path.exists() {
        let expected_detailed = wai_detailed_content(
            project_root,
            &plugin_names,
            &skill_name_refs,
            &installed_pipelines,
        );
        if let Ok(actual_detailed) = std::fs::read_to_string(&detailed_path)
            && actual_detailed != expected_detailed
        {
            results.push(CheckResult {
                name: "Managed block staleness: .wai/AGENTS.md".to_string(),
                status: Status::Warn,
                message: ".wai/AGENTS.md outdated — run 'wai init --update' to refresh".to_string(),
                fix: Some("Run: wai init".to_string()),
                fix_fn: None,
            });
        }
    } else {
        results.push(CheckResult {
            name: "Managed block staleness: .wai/AGENTS.md".to_string(),
            status: Status::Warn,
            message: ".wai/AGENTS.md missing — run 'wai init' to create it".to_string(),
            fix: Some("Run: wai init".to_string()),
            fix_fn: None,
        });
    }

    results
}

/// Validate pipeline TOML definitions for correctness.
fn check_pipeline_definitions(project_root: &Path) -> Vec<CheckResult> {
    use crate::config::pipelines_dir;

    let mut results = Vec::new();
    let pipelines = pipelines_dir(project_root);
    if !pipelines.exists() {
        return results;
    }
    let Ok(entries) = std::fs::read_dir(&pipelines) else {
        return results;
    };

    let mut names_seen: Vec<(String, String)> = Vec::new(); // (name, file)

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                results.push(CheckResult {
                    name: format!("Pipeline: {}", file_stem),
                    status: Status::Fail,
                    message: format!("Cannot read: {}", e),
                    fix: None,
                    fix_fn: None,
                });
                continue;
            }
        };

        // Try to parse the TOML
        let parsed: Result<toml::Value, _> = toml::from_str(&content);
        match parsed {
            Err(e) => {
                results.push(CheckResult {
                    name: format!("Pipeline: {}", file_stem),
                    status: Status::Fail,
                    message: format!("Invalid TOML: {}", e),
                    fix: None,
                    fix_fn: None,
                });
                continue;
            }
            Ok(val) => {
                // Check for pipeline name
                let pipeline_name = val
                    .get("pipeline")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");

                if pipeline_name.is_empty() {
                    results.push(CheckResult {
                        name: format!("Pipeline: {}", file_stem),
                        status: Status::Fail,
                        message: "Missing [pipeline].name".to_string(),
                        fix: None,
                        fix_fn: None,
                    });
                    continue;
                }

                // Check for duplicate names
                if let Some((_, prev_file)) = names_seen.iter().find(|(n, _)| n == pipeline_name) {
                    results.push(CheckResult {
                        name: format!("Pipeline: {}", file_stem),
                        status: Status::Fail,
                        message: format!(
                            "Duplicate pipeline name '{}' (also in {})",
                            pipeline_name, prev_file
                        ),
                        fix: None,
                        fix_fn: None,
                    });
                } else {
                    names_seen.push((pipeline_name.to_string(), file_stem.clone()));
                }

                // Check for metadata
                let has_metadata = val
                    .get("pipeline")
                    .and_then(|p| p.get("metadata"))
                    .and_then(|m| m.get("when"))
                    .and_then(|w| w.as_str())
                    .is_some();

                if !has_metadata {
                    results.push(CheckResult {
                        name: format!("Pipeline: {}", file_stem),
                        status: Status::Warn,
                        message: format!(
                            "Missing [pipeline.metadata] — pipeline '{}' won't appear in managed block",
                            pipeline_name
                        ),
                        fix: None,
                        fix_fn: None,
                    });
                }

                // Check oracle references
                let oracles_dir = crate::config::wai_dir(project_root)
                    .join("resources")
                    .join("oracles");
                if let Some(steps) = val.get("steps").and_then(|s| s.as_array()) {
                    for step in steps {
                        let Some(gate) = step.get("gate") else {
                            continue;
                        };
                        let Some(oracles) = gate.get("oracles").and_then(|o| o.as_array()) else {
                            continue;
                        };
                        for oracle in oracles {
                            if oracle.get("command").is_some() {
                                continue;
                            }
                            let Some(name) = oracle.get("name").and_then(|n| n.as_str()) else {
                                continue;
                            };
                            let found = ["", ".sh", ".py"]
                                .iter()
                                .any(|ext| oracles_dir.join(format!("{}{}", name, ext)).exists());
                            if !found {
                                results.push(CheckResult {
                                    name: format!("Pipeline: {}", file_stem),
                                    status: Status::Warn,
                                    message: format!("Gate oracle '{}' — command not found", name),
                                    fix: None,
                                    fix_fn: None,
                                });
                                continue;
                            }
                            // Check executability
                            let mut executable = false;
                            for ext in &["", ".sh", ".py"] {
                                let p = oracles_dir.join(format!("{}{}", name, ext));
                                if p.exists() {
                                    #[cfg(unix)]
                                    {
                                        use std::os::unix::fs::PermissionsExt;
                                        if let Ok(meta) = p.metadata()
                                            && meta.permissions().mode() & 0o111 != 0
                                        {
                                            executable = true;
                                        }
                                    }
                                    #[cfg(not(unix))]
                                    {
                                        executable = true;
                                    }
                                    break;
                                }
                            }
                            if !executable {
                                results.push(CheckResult {
                                    name: format!("Pipeline: {}", file_stem),
                                    status: Status::Warn,
                                    message: format!("Gate oracle '{}' — not executable", name),
                                    fix: None,
                                    fix_fn: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

fn check_artifact_locks(project_root: &Path) -> Vec<CheckResult> {
    use super::pipeline::{artifact_hash, read_artifact_lock};
    use walkdir::WalkDir;

    let projects = projects_dir(project_root);
    if !projects.exists() {
        return vec![];
    }

    let lock_files: Vec<std::path::PathBuf> = WalkDir::new(&projects)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().and_then(|ext| ext.to_str()) == Some("lock")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if lock_files.is_empty() {
        return vec![];
    }

    let mut mismatches = Vec::new();
    let mut verified = 0usize;

    for lock_path in &lock_files {
        let lock = match read_artifact_lock(lock_path) {
            Ok(l) => l,
            Err(_) => {
                mismatches.push(CheckResult {
                    name: "Artifact lock".to_string(),
                    status: Status::Warn,
                    message: format!(
                        "Cannot read lock file: {}",
                        lock_path
                            .strip_prefix(project_root)
                            .unwrap_or(lock_path)
                            .display()
                    ),
                    fix: None,
                    fix_fn: None,
                });
                continue;
            }
        };

        let artifact_path = lock_path.parent().unwrap().join(&lock.artifact);
        if !artifact_path.exists() {
            mismatches.push(CheckResult {
                name: "Artifact lock".to_string(),
                status: Status::Warn,
                message: format!(
                    "Locked artifact missing: {}",
                    artifact_path
                        .strip_prefix(project_root)
                        .unwrap_or(&artifact_path)
                        .display()
                ),
                fix: None,
                fix_fn: None,
            });
            continue;
        }

        match artifact_hash(&artifact_path) {
            Ok(current_hash) => {
                if current_hash != lock.lock_hash {
                    mismatches.push(CheckResult {
                        name: "Artifact lock".to_string(),
                        status: Status::Warn,
                        message: format!(
                            "Hash mismatch: {} (run {})",
                            lock.artifact, lock.pipeline_run
                        ),
                        fix: None,
                        fix_fn: None,
                    });
                } else {
                    verified += 1;
                }
            }
            Err(_) => {
                mismatches.push(CheckResult {
                    name: "Artifact lock".to_string(),
                    status: Status::Warn,
                    message: format!(
                        "Cannot hash artifact: {}",
                        artifact_path
                            .strip_prefix(project_root)
                            .unwrap_or(&artifact_path)
                            .display()
                    ),
                    fix: None,
                    fix_fn: None,
                });
            }
        }
    }

    if mismatches.is_empty() {
        vec![CheckResult {
            name: "Artifact locks".to_string(),
            status: Status::Pass,
            message: format!("All {} locked artifacts verified", verified),
            fix: None,
            fix_fn: None,
        }]
    } else {
        mismatches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: create a minimal wai workspace with `.wai/projects/` directory.
    fn setup_workspace() -> TempDir {
        let tmp = TempDir::new().expect("create tempdir");
        let projects = tmp.path().join(".wai").join("projects").join("test-proj");
        std::fs::create_dir_all(&projects).expect("create projects dir");
        tmp
    }

    /// Helper: write an artifact file and a matching lock sidecar.
    fn write_artifact_and_lock(
        dir: &Path,
        artifact_name: &str,
        content: &str,
        run_id: &str,
        tamper: bool,
    ) {
        use super::super::pipeline::artifact_hash;

        let artifact_path = dir.join(artifact_name);
        std::fs::write(&artifact_path, content).expect("write artifact");

        let hash = if tamper {
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string()
        } else {
            artifact_hash(&artifact_path).expect("hash artifact")
        };

        let lock = toml::to_string_pretty(&super::super::pipeline::ArtifactLock {
            artifact: artifact_name.to_string(),
            locked_at: "2026-01-01T00:00:00Z".to_string(),
            lock_hash: hash,
            pipeline_run: run_id.to_string(),
            pipeline_step: "step-1".to_string(),
        })
        .expect("serialize lock");

        let lock_name = format!("{}.{}.lock", artifact_name, run_id);
        std::fs::write(dir.join(lock_name), lock).expect("write lock");
    }

    #[test]
    fn no_lock_files_returns_empty() {
        let tmp = setup_workspace();
        let results = check_artifact_locks(tmp.path());
        assert!(results.is_empty());
    }

    #[test]
    fn valid_lock_returns_pass() {
        let tmp = setup_workspace();
        let proj_dir = tmp.path().join(".wai/projects/test-proj");
        write_artifact_and_lock(
            &proj_dir,
            "research.md",
            "# Research\nFindings here\n",
            "run-1",
            false,
        );

        let results = check_artifact_locks(tmp.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, Status::Pass);
        assert!(results[0].message.contains("1 locked artifacts verified"));
    }

    #[test]
    fn tampered_artifact_returns_warn() {
        let tmp = setup_workspace();
        let proj_dir = tmp.path().join(".wai/projects/test-proj");
        write_artifact_and_lock(&proj_dir, "design.md", "# Design\nChoices\n", "run-2", true);

        let results = check_artifact_locks(tmp.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, Status::Warn);
        assert!(results[0].message.contains("Hash mismatch"));
    }

    #[test]
    fn missing_artifact_returns_warn() {
        let tmp = setup_workspace();
        let proj_dir = tmp.path().join(".wai/projects/test-proj");
        // Write only the lock, not the artifact
        let lock = toml::to_string_pretty(&super::super::pipeline::ArtifactLock {
            artifact: "ghost.md".to_string(),
            locked_at: "2026-01-01T00:00:00Z".to_string(),
            lock_hash: "sha256:abc".to_string(),
            pipeline_run: "run-3".to_string(),
            pipeline_step: "step-1".to_string(),
        })
        .expect("serialize lock");
        std::fs::write(proj_dir.join("ghost.md.run-3.lock"), lock).expect("write lock");

        let results = check_artifact_locks(tmp.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, Status::Warn);
        assert!(results[0].message.contains("Locked artifact missing"));
    }

    #[test]
    fn mixed_valid_and_tampered() {
        let tmp = setup_workspace();
        let proj_dir = tmp.path().join(".wai/projects/test-proj");
        write_artifact_and_lock(&proj_dir, "good.md", "valid content", "run-4", false);
        write_artifact_and_lock(&proj_dir, "bad.md", "tampered content", "run-4", true);

        let results = check_artifact_locks(tmp.path());
        // Should only contain the mismatch warning (pass is suppressed when mismatches exist)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, Status::Warn);
        assert!(results[0].message.contains("Hash mismatch"));
        assert!(results[0].message.contains("bad.md"));
    }
}
