mod hooks;
mod linting;
mod release;

use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::path::Path;

use genesis::doctor::CheckStatus;

use crate::config::{SKILLS_DIR, agent_config_dir};
use crate::context::current_context;
use crate::output::print_envelope_check;

const SKILL_RULE_OF_5: (&str, &str) = ("rule-of-5-universal", include_str!("skill-rule-of-5.md"));

const SKILL_COMMIT: (&str, &str) = ("commit", include_str!("skill-commit.md"));

#[derive(Serialize)]
pub(crate) struct WayCheckEntry {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_criteria: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Serialize)]
struct WayPayload {
    checks: Vec<WayCheckEntry>,
    summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Summary {
    pub pass: usize,
    pub recommendations: usize,
}

mod checks;
mod guide;

use checks::*;
use guide::{
    guide_ai, guide_ci, guide_code_quality, guide_coverage, guide_devxp, guide_docs, guide_gh,
    guide_hooks, guide_issues, guide_specs,
};

fn collect_checks(repo_root: &Path) -> Vec<WayCheckEntry> {
    vec![
        check_task_runner(repo_root),
        hooks::check_git_hooks(repo_root),
        check_editorconfig(repo_root),
        linting::check_typos(repo_root),
        linting::check_vale(repo_root),
        linting::check_shell_linting(repo_root),
        check_documentation(repo_root),
        check_docs_status_page(repo_root),
        check_docs_openspec_inclusion(repo_root),
        check_ai_instructions(repo_root),
        check_artifact_stubs(repo_root),
        check_llm_txt(repo_root),
        check_ubiquitous_language(repo_root),
        check_agent_skills(repo_root),
        check_agent_config_sync(repo_root),
        check_gh_cli(),
        check_pretender(repo_root),
        check_ci_cd(repo_root),
        check_devcontainer(repo_root),
        release::check_release_pipeline(repo_root),
        check_test_coverage(repo_root),
        check_beads(repo_root),
        check_openspec(repo_root),
    ]
}

fn summarize(checks: &[WayCheckEntry]) -> Summary {
    Summary {
        pass: checks
            .iter()
            .filter(|c| c.status == CheckStatus::Pass)
            .count(),
        recommendations: checks
            .iter()
            .filter(|c| c.status == CheckStatus::Warn)
            .count(),
    }
}

pub fn run(topic: Option<String>, fix: Option<String>) -> Result<()> {
    // way works in any directory - doesn't require .wai/ initialization
    let repo_root = std::env::current_dir()
        .map_err(|e| miette::miette!("Cannot determine current directory: {}", e))?;

    if let Some(ref topic) = topic {
        return print_topic_guide(topic, &repo_root);
    }

    if let Some(target) = fix {
        return match target.as_str() {
            "skills" => fix_skills(&repo_root),
            other => miette::bail!(
                "Unknown fix target '{other}'. Available: 'skills'. Use 'wai way <topic>' (e.g. 'wai way ci') for interactive guidance on: {}",
                AVAILABLE_TOPICS.join(", ")
            ),
        };
    }

    let context = current_context();

    let checks = collect_checks(&repo_root);
    let summary = summarize(&checks);

    if context.json {
        let payload = WayPayload { checks, summary };
        print_envelope_check(payload)?;
    } else {
        render_human(&checks, &summary, context.verbose)?;
    }

    // Always exit 0 - these are recommendations, not requirements
    Ok(())
}

const AVAILABLE_TOPICS: &[&str] = &[
    "ai",
    "ci",
    "code-quality",
    "coverage",
    "devxp",
    "docs",
    "gh",
    "hooks",
    "issues",
    "specs",
];

fn print_topic_guide(topic: &str, repo_root: &Path) -> Result<()> {
    let guide = match topic {
        "ai" => guide_ai(repo_root),
        "ci" => guide_ci(repo_root),
        "coverage" => guide_coverage(repo_root),
        "devxp" => guide_devxp(repo_root),
        "docs" => guide_docs(repo_root),
        "code-quality" => guide_code_quality(repo_root),
        "gh" => guide_gh(),
        "hooks" => guide_hooks(repo_root),
        "issues" => guide_issues(repo_root),
        "specs" => guide_specs(repo_root),
        other => {
            miette::bail!(
                "Unknown topic '{}'. Available: {}",
                other,
                AVAILABLE_TOPICS.join(", ")
            );
        }
    };

    println!("{}", guide);
    Ok(())
}

fn fix_skills(repo_root: &Path) -> Result<()> {
    use cliclack::log;

    let skills_dir = agent_config_dir(repo_root).join(SKILLS_DIR);
    std::fs::create_dir_all(&skills_dir).into_diagnostic()?;

    println!();
    println!(
        "  Scaffolding recommended skills into {}:",
        skills_dir
            .strip_prefix(repo_root)
            .unwrap_or(&skills_dir)
            .display()
    );
    println!("    • rule-of-5-universal — iterative quality review workflow");
    println!("    • commit — structured, deliberate commit workflow");
    println!();

    let mut created = 0usize;

    for (skill_name, content) in [SKILL_RULE_OF_5, SKILL_COMMIT] {
        let skill_dir = skills_dir.join(skill_name);
        let skill_file = skill_dir.join("SKILL.md");
        if skill_file.exists() {
            println!("  {} {} — already present", "○".dimmed(), skill_name);
            continue;
        }
        std::fs::create_dir_all(&skill_dir).into_diagnostic()?;
        std::fs::write(&skill_file, content).into_diagnostic()?;
        log::success(format!("Created skill '{}'", skill_name)).into_diagnostic()?;
        created += 1;
    }

    if created == 0 {
        println!("\n  Recommended skills already present — nothing to do.");
    } else {
        println!(
            "\n  {} skill(s) added to .wai/resources/agent-config/skills/",
            created
        );
    }

    Ok(())
}

fn render_human(checks: &[WayCheckEntry], summary: &Summary, verbose: u8) -> Result<()> {
    use cliclack::outro;
    use miette::IntoDiagnostic;

    println!();
    println!("  {} Repo Hygiene & Agent Workflow Conventions", "◆".cyan());
    println!(
        "  {} For wai workspace health, run 'wai doctor'",
        "·".dimmed()
    );
    println!();

    for check in checks {
        let icon = match check.status {
            CheckStatus::Pass => "✓".green().to_string(),
            CheckStatus::Warn => "ℹ".cyan().to_string(),
            _ => "?".to_string(),
        };
        println!("  {} {}: {}", icon, check.name.bold(), check.message);
        if verbose > 0 {
            if let Some(ref intent) = check.intent {
                println!("    {} Intent: {}", "·".dimmed(), intent.dimmed());
            }
            if let Some(ref criteria) = check.success_criteria {
                println!("    {} Success: {}", "·".dimmed(), criteria.dimmed());
            }
        }
        if let Some(ref suggestion) = check.suggestion {
            println!("    {} {}", "→".dimmed(), suggestion.dimmed());
        }
    }

    println!();
    let total_checks = summary.pass + summary.recommendations;
    let summary_line = if summary.recommendations == 0 {
        "excellent! All best practices adopted".to_string()
    } else if summary.pass == 0 {
        format!(
            "{}/{} best practices adopted — quick-start: add README.md, justfile, .gitignore",
            summary.pass, total_checks
        )
    } else {
        format!("{}/{} best practices adopted", summary.pass, total_checks)
    };

    if summary.recommendations > 0 {
        outro(summary_line.cyan().to_string()).into_diagnostic()?;
    } else {
        outro(summary_line.green().to_string()).into_diagnostic()?;
    }

    println!(
        "  {} Deep-dive into any area: {}",
        "·".dimmed(),
        format!("wai way <{}>", AVAILABLE_TOPICS.join("|")).dimmed()
    );
    println!();

    Ok(())
}
