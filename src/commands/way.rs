use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

use crate::commands::resource::parse_skill_frontmatter;
use crate::config::{SKILLS_DIR, agent_config_dir};
use crate::context::current_context;
use crate::output::print_json;

const SKILL_RULE_OF_5: (&str, &str) = (
    "rule-of-5-universal",
    include_str!("../../.wai/resources/agent-config/skills/rule-of-5-universal/SKILL.md"),
);

const SKILL_COMMIT: (&str, &str) = (
    "commit",
    include_str!("../../.wai/resources/agent-config/skills/commit/SKILL.md"),
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pass,
    Info,
}

#[derive(Serialize)]
struct CheckResult {
    name: String,
    status: Status,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

#[derive(Serialize)]
struct WayPayload {
    checks: Vec<CheckResult>,
    summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
struct Summary {
    pass: usize,
    recommendations: usize,
}

pub fn run(fix: Option<String>) -> Result<()> {
    // way works in any directory - doesn't require .wai/ initialization
    let repo_root = std::env::current_dir()
        .map_err(|e| miette::miette!("Cannot determine current directory: {}", e))?;

    if let Some(target) = fix {
        return match target.as_str() {
            "skills" => fix_skills(&repo_root),
            other => miette::bail!("Unknown fix target '{}'. Available: skills", other),
        };
    }

    let context = current_context();

    let checks = vec![
        check_task_runner(&repo_root),
        check_git_hooks(&repo_root),
        check_editorconfig(&repo_root),
        check_documentation(&repo_root),
        check_ai_instructions(&repo_root),
        check_llm_txt(&repo_root),
        check_agent_skills(&repo_root),
        check_gh_cli(),
        check_ci_cd(&repo_root),
        check_devcontainer(&repo_root),
    ];

    let summary = Summary {
        pass: checks.iter().filter(|c| c.status == Status::Pass).count(),
        recommendations: checks.iter().filter(|c| c.status == Status::Info).count(),
    };

    if context.json {
        let payload = WayPayload { checks, summary };
        print_json(&payload)?;
    } else {
        render_human(&checks, &summary)?;
    }

    // Always exit 0 - these are recommendations, not requirements
    Ok(())
}

fn fix_skills(repo_root: &Path) -> Result<()> {
    use cliclack::log;

    let skills_dir = agent_config_dir(repo_root).join(SKILLS_DIR);
    std::fs::create_dir_all(&skills_dir).into_diagnostic()?;

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

fn render_human(checks: &[CheckResult], summary: &Summary) -> Result<()> {
    use cliclack::outro;
    use miette::IntoDiagnostic;

    println!();
    println!("  {} The Wai Way — Repository Best Practices", "◆".cyan());
    println!();

    for check in checks {
        let icon = match check.status {
            Status::Pass => "✓".green().to_string(),
            Status::Info => "ℹ".cyan().to_string(),
        };
        println!("  {} {}: {}", icon, check.name.bold(), check.message);
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

    Ok(())
}

fn check_task_runner(repo_root: &Path) -> CheckResult {
    let justfile = repo_root.join("justfile");
    let makefile = repo_root.join("Makefile");

    if justfile.exists() {
        let recipes = parse_justfile_recipes(&justfile);
        let message = if recipes.is_empty() {
            "justfile detected".to_string()
        } else {
            format!("justfile detected (recipes: {})", recipes.join(", "))
        };

        CheckResult {
            name: "Task runner".to_string(),
            status: Status::Pass,
            message,
            suggestion: None,
        }
    } else if makefile.exists() {
        CheckResult {
            name: "Task runner".to_string(),
            status: Status::Pass,
            message: "Makefile detected".to_string(),
            suggestion: Some(
                "Consider migrating to justfile for better ergonomics — https://just.systems"
                    .to_string(),
            ),
        }
    } else {
        CheckResult {
            name: "Task runner".to_string(),
            status: Status::Info,
            message: "No task runner detected".to_string(),
            suggestion: Some(
                "Add a justfile to standardize common tasks — https://just.systems".to_string(),
            ),
        }
    }
}

fn parse_justfile_recipes(justfile_path: &Path) -> Vec<String> {
    let known_recipes = [
        "install", "serve", "dev", "test", "lint", "fmt", "format", "release", "build", "run",
        "clean", "check", "watch",
    ];

    let content = match std::fs::read_to_string(justfile_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut found_recipes = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // Recipe definitions start at the beginning of a line (no leading whitespace)
        // and end with a colon
        if !line.starts_with(' ')
            && !line.starts_with('\t')
            && trimmed.contains(':')
            && let Some(recipe_name) = trimmed.split(':').next()
        {
            let recipe_name = recipe_name.split_whitespace().next().unwrap_or("");
            if known_recipes.contains(&recipe_name)
                && !found_recipes.contains(&recipe_name.to_string())
            {
                found_recipes.push(recipe_name.to_string());
            }
        }
    }

    found_recipes
}

fn check_git_hooks(repo_root: &Path) -> CheckResult {
    let prek_config = repo_root.join(".prek.toml");
    let precommit_config = repo_root.join(".pre-commit-config.yaml");

    if prek_config.exists() {
        CheckResult {
            name: "Git hooks".to_string(),
            status: Status::Pass,
            message: "prek detected (recommended)".to_string(),
            suggestion: None,
        }
    } else if precommit_config.exists() {
        CheckResult {
            name: "Git hooks".to_string(),
            status: Status::Pass,
            message: "pre-commit detected".to_string(),
            suggestion: Some(
                "Consider prek for simpler hook management — https://github.com/chshersh/prek"
                    .to_string(),
            ),
        }
    } else {
        CheckResult {
            name: "Git hooks".to_string(),
            status: Status::Info,
            message: "No git hook manager detected".to_string(),
            suggestion: Some(
                "Add prek to manage git hooks — https://github.com/chshersh/prek".to_string(),
            ),
        }
    }
}

fn check_editorconfig(repo_root: &Path) -> CheckResult {
    let editorconfig = repo_root.join(".editorconfig");

    if editorconfig.exists() {
        CheckResult {
            name: "Editor config".to_string(),
            status: Status::Pass,
            message: ".editorconfig detected".to_string(),
            suggestion: None,
        }
    } else {
        CheckResult {
            name: "Editor config".to_string(),
            status: Status::Info,
            message: "No .editorconfig detected".to_string(),
            suggestion: Some(
                "Add .editorconfig to standardize formatting — https://editorconfig.org"
                    .to_string(),
            ),
        }
    }
}

fn check_documentation(repo_root: &Path) -> CheckResult {
    let readme = repo_root.join("README.md").exists();
    let license = repo_root.join("LICENSE").exists() || repo_root.join("LICENSE.md").exists();
    let contributing = repo_root.join("CONTRIBUTING.md").exists();
    let gitignore = repo_root.join(".gitignore").exists();

    let count = [readme, license, contributing, gitignore]
        .iter()
        .filter(|&&x| x)
        .count();

    match count {
        4 => CheckResult {
            name: "Documentation".to_string(),
            status: Status::Pass,
            message: "Complete".to_string(),
            suggestion: None,
        },
        0 => CheckResult {
            name: "Documentation".to_string(),
            status: Status::Info,
            message: "Not configured".to_string(),
            suggestion: Some(
                "Add README.md and .gitignore at minimum, plus LICENSE and CONTRIBUTING.md"
                    .to_string(),
            ),
        },
        _ => {
            // Check if critical files are missing
            if !readme || !gitignore {
                let mut missing = Vec::new();
                if !readme {
                    missing.push("README.md");
                }
                if !gitignore {
                    missing.push(".gitignore");
                }
                CheckResult {
                    name: "Documentation".to_string(),
                    status: Status::Info,
                    message: format!("⚠️  Missing critical files: {}", missing.join(", ")),
                    suggestion: Some("Add missing critical documentation files".to_string()),
                }
            } else {
                CheckResult {
                    name: "Documentation".to_string(),
                    status: Status::Pass,
                    message: format!("Partial documentation ({}/4 files)", count),
                    suggestion: Some("Consider adding LICENSE and CONTRIBUTING.md".to_string()),
                }
            }
        }
    }
}

fn check_ai_instructions(repo_root: &Path) -> CheckResult {
    use crate::managed_block::has_reflect_block;

    let claude_md = repo_root.join("CLAUDE.md");
    let agents_md = repo_root.join("AGENTS.md");

    if claude_md.exists() {
        let has_reflect = has_reflect_block(&claude_md);
        let suggestion = if !has_reflect {
            Some(
                "No WAI:REFLECT block found — run `wai reflect` to synthesize project-specific AI guidance into CLAUDE.md".to_string(),
            )
        } else {
            None
        };
        CheckResult {
            name: "AI instructions".to_string(),
            status: Status::Pass,
            message: "CLAUDE.md detected (recommended for Claude Code)".to_string(),
            suggestion,
        }
    } else if agents_md.exists() {
        CheckResult {
            name: "AI instructions".to_string(),
            status: Status::Pass,
            message: "AGENTS.md detected".to_string(),
            suggestion: Some("Consider adding CLAUDE.md for Claude Code compatibility".to_string()),
        }
    } else {
        CheckResult {
            name: "AI instructions".to_string(),
            status: Status::Info,
            message: "No AI instruction files detected".to_string(),
            suggestion: Some("Create CLAUDE.md to provide context to AI assistants".to_string()),
        }
    }
}

fn check_ci_cd(repo_root: &Path) -> CheckResult {
    let github_workflows = repo_root.join(".github/workflows");
    let gitlab_ci = repo_root.join(".gitlab-ci.yml");
    let circleci = repo_root.join(".circleci/config.yml");

    if github_workflows.exists() && github_workflows.is_dir() {
        let workflow_count = std::fs::read_dir(&github_workflows)
            .ok()
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        if workflow_count > 0 {
            CheckResult {
                name: "CI/CD".to_string(),
                status: Status::Pass,
                message: format!("GitHub Actions configured ({} workflow(s))", workflow_count),
                suggestion: None,
            }
        } else {
            CheckResult {
                name: "CI/CD".to_string(),
                status: Status::Info,
                message: "GitHub Actions directory present but empty".to_string(),
                suggestion: Some("Add workflow files to .github/workflows/".to_string()),
            }
        }
    } else if gitlab_ci.exists() {
        CheckResult {
            name: "CI/CD".to_string(),
            status: Status::Pass,
            message: "GitLab CI configured".to_string(),
            suggestion: None,
        }
    } else if circleci.exists() {
        CheckResult {
            name: "CI/CD".to_string(),
            status: Status::Pass,
            message: "CircleCI configured".to_string(),
            suggestion: None,
        }
    } else {
        CheckResult {
            name: "CI/CD".to_string(),
            status: Status::Info,
            message: "No CI/CD configuration detected".to_string(),
            suggestion: Some("Set up continuous integration to automate testing".to_string()),
        }
    }
}

fn check_devcontainer(repo_root: &Path) -> CheckResult {
    let devcontainer_dir = repo_root.join(".devcontainer");
    let devcontainer_json = repo_root.join(".devcontainer.json");

    if devcontainer_dir.exists() && devcontainer_dir.is_dir() {
        CheckResult {
            name: "Dev container".to_string(),
            status: Status::Pass,
            message: ".devcontainer/ directory detected".to_string(),
            suggestion: None,
        }
    } else if devcontainer_json.exists() {
        CheckResult {
            name: "Dev container".to_string(),
            status: Status::Pass,
            message: ".devcontainer.json detected".to_string(),
            suggestion: None,
        }
    } else {
        CheckResult {
            name: "Dev container".to_string(),
            status: Status::Info,
            message: "No dev container configuration detected".to_string(),
            suggestion: Some(
                "Consider adding .devcontainer/ for reproducible development environments"
                    .to_string(),
            ),
        }
    }
}

fn check_llm_txt(repo_root: &Path) -> CheckResult {
    let llm_txt = repo_root.join("llm.txt");

    if llm_txt.exists() {
        CheckResult {
            name: "LLM documentation".to_string(),
            status: Status::Pass,
            message: "llm.txt detected".to_string(),
            suggestion: None,
        }
    } else {
        CheckResult {
            name: "LLM documentation".to_string(),
            status: Status::Info,
            message: "No llm.txt detected".to_string(),
            suggestion: Some(
                "Add llm.txt for AI-friendly project documentation — https://llmstxt.org"
                    .to_string(),
            ),
        }
    }
}

fn check_agent_skills(repo_root: &Path) -> CheckResult {
    let skills_dir = agent_config_dir(repo_root).join(SKILLS_DIR);

    if !skills_dir.exists() {
        return CheckResult {
            name: "Agent skills".to_string(),
            status: Status::Info,
            message: "No skills configured".to_string(),
            suggestion: Some(
                "Add rule-of-5-universal (ro5) and commit to .wai/resources/agent-config/skills/"
                    .to_string(),
            ),
        };
    }

    // Collect skill dir names and aliases from frontmatter
    let mut skill_ids: HashSet<String> = HashSet::new();
    let mut skill_count = 0usize;

    if let Ok(entries) = std::fs::read_dir(&skills_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let skill_file = entry.path().join("SKILL.md");
            if skill_file.exists() {
                skill_count += 1;
                if let Some(dir_name) = entry.file_name().to_str() {
                    skill_ids.insert(dir_name.to_string());
                }
                if let Some(meta) = parse_skill_frontmatter(&skill_file) {
                    for alias in meta.aliases {
                        skill_ids.insert(alias);
                    }
                }
            }
        }
    }

    if skill_count == 0 {
        return CheckResult {
            name: "Agent skills".to_string(),
            status: Status::Info,
            message: "Skills directory present but empty".to_string(),
            suggestion: Some(
                "Add rule-of-5-universal (ro5) and commit to .wai/resources/agent-config/skills/"
                    .to_string(),
            ),
        };
    }

    let has_ro5 =
        skill_ids.contains("rule-of-5-universal") || skill_ids.contains("ro5");
    let has_commit = skill_ids.contains("commit");

    let missing: Vec<&str> = [
        (!has_ro5).then_some("rule-of-5-universal (ro5)"),
        (!has_commit).then_some("commit"),
    ]
    .into_iter()
    .flatten()
    .collect();

    if missing.is_empty() {
        CheckResult {
            name: "Agent skills".to_string(),
            status: Status::Pass,
            message: format!(
                "{} skill(s) configured — includes rule-of-5-universal (ro5) and commit",
                skill_count
            ),
            suggestion: None,
        }
    } else {
        CheckResult {
            name: "Agent skills".to_string(),
            status: Status::Info,
            message: format!(
                "{} skill(s) configured — missing recommended: {}",
                skill_count,
                missing.join(", ")
            ),
            suggestion: Some(format!(
                "Add to .wai/resources/agent-config/skills/: {}",
                missing.join(", ")
            )),
        }
    }
}

fn check_gh_cli() -> CheckResult {
    let gh_installed = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .is_ok();

    if !gh_installed {
        return CheckResult {
            name: "GitHub CLI".to_string(),
            status: Status::Info,
            message: "gh not installed".to_string(),
            suggestion: Some(
                "Install gh CLI for better GitHub integration — https://cli.github.com".to_string(),
            ),
        };
    }

    let auth_status = std::process::Command::new("gh")
        .args(["auth", "status"])
        .output();

    match auth_status {
        Ok(output) if output.status.success() => CheckResult {
            name: "GitHub CLI".to_string(),
            status: Status::Pass,
            message: "gh installed and authenticated".to_string(),
            suggestion: None,
        },
        _ => CheckResult {
            name: "GitHub CLI".to_string(),
            status: Status::Info,
            message: "gh installed but not authenticated".to_string(),
            suggestion: Some("Run 'gh auth login' to authenticate".to_string()),
        },
    }
}
