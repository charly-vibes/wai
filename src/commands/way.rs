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
    intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success_criteria: Option<String>,
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
        check_release_pipeline(&repo_root),
        check_test_coverage(&repo_root),
        check_beads(&repo_root),
        check_openspec(&repo_root),
    ];

    let summary = Summary {
        pass: checks.iter().filter(|c| c.status == Status::Pass).count(),
        recommendations: checks.iter().filter(|c| c.status == Status::Info).count(),
    };

    if context.json {
        let payload = WayPayload { checks, summary };
        print_json(&payload)?;
    } else {
        render_human(&checks, &summary, context.verbose)?;
    }

    // Always exit 0 - these are recommendations, not requirements
    Ok(())
}

fn fix_skills(repo_root: &Path) -> Result<()> {
    use cliclack::log;

    let skills_dir = agent_config_dir(repo_root).join(SKILLS_DIR);
    std::fs::create_dir_all(&skills_dir).into_diagnostic()?;

    println!();
    println!(
        "  Scaffolding recommended skills into {}:",
        skills_dir.strip_prefix(repo_root).unwrap_or(&skills_dir).display()
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

fn render_human(checks: &[CheckResult], summary: &Summary, verbose: u8) -> Result<()> {
    use cliclack::outro;
    use miette::IntoDiagnostic;

    println!();
    println!(
        "  {} Repo Hygiene & Agent Workflow Conventions",
        "◆".cyan()
    );
    println!("  {} For wai workspace health, run 'wai doctor'", "·".dimmed());
    println!();

    for check in checks {
        let icon = match check.status {
            Status::Pass => "✓".green().to_string(),
            Status::Info => "ℹ".cyan().to_string(),
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

    Ok(())
}

fn check_task_runner(repo_root: &Path) -> CheckResult {
    let name = "Command standardization";
    let intent = Some("Provide a single, tool-agnostic entry point for common repository tasks (build, test, deploy).".to_string());
    let success_criteria = Some(
        "A standard interface (justfile, Makefile, npm scripts) exists for common tasks."
            .to_string(),
    );

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
            name: name.to_string(),
            status: Status::Pass,
            message,
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if makefile.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "Makefile detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Consider migrating to justfile for better ergonomics — https://just.systems"
                    .to_string(),
            ),
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No task runner detected".to_string(),
            intent,
            success_criteria,
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

/// Read `.git/hooks/pre-commit` contents, or `None` if absent/unreadable.
fn read_precommit_hook(repo_root: &Path) -> Option<String> {
    let hook_path = repo_root.join(".git/hooks/pre-commit");
    if !hook_path.exists() || hook_path.is_dir() {
        return None;
    }
    std::fs::read_to_string(&hook_path).ok()
}

/// Return `true` if the pre-commit hook file contains `needle`.
fn hook_contains(repo_root: &Path, needle: &str) -> bool {
    read_precommit_hook(repo_root).is_some_and(|c| c.contains(needle))
}

/// Return `true` if the pre-commit hook file exists and is non-empty.
fn hook_exists_nonempty(repo_root: &Path) -> bool {
    read_precommit_hook(repo_root).is_some_and(|c| !c.trim().is_empty())
}

fn check_git_hooks(repo_root: &Path) -> CheckResult {
    let name = "Pre-commit quality gates";
    let intent = Some(
        "Prevent low-quality commits by running automated checks before code is saved to history."
            .to_string(),
    );
    let success_criteria = Some(
        "Automated checks (linters, tests) run automatically before code is committed.".to_string(),
    );

    let prek_config = repo_root.join("prek.toml");
    let precommit_config = repo_root.join(".pre-commit-config.yaml");
    let lefthook_config = repo_root.join("lefthook.yml");
    let lefthook_config_yaml = repo_root.join("lefthook.yaml");
    let husky_dir = repo_root.join(".husky");

    if prek_config.exists() {
        if hook_contains(repo_root, "prek") {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "prek detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: "prek.toml found but hooks not installed — run: prek install".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        }
    } else if lefthook_config.exists() || lefthook_config_yaml.exists() {
        if hook_contains(repo_root, "lefthook") {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "lefthook detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: "lefthook.yml found but hooks not installed — run: lefthook install"
                    .to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        }
    } else if husky_dir.exists() && husky_dir.is_dir() {
        if hook_exists_nonempty(repo_root) {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "husky detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: ".husky/ found but hooks not installed — run: npx husky install"
                    .to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        }
    } else if precommit_config.exists() {
        if hook_contains(repo_root, "pre-commit") {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "pre-commit detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Consider prek for simpler hook management — https://github.com/chshersh/prek"
                        .to_string(),
                ),
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: ".pre-commit-config.yaml found but hooks not installed — run: pre-commit install".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Consider prek for simpler hook management — https://github.com/chshersh/prek"
                        .to_string(),
                ),
            }
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No git hook manager detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add prek to manage git hooks — https://github.com/chshersh/prek".to_string(),
            ),
        }
    }
}

fn check_editorconfig(repo_root: &Path) -> CheckResult {
    let name = "Consistent formatting";
    let intent =
        Some("Ensure consistent code formatting across different editors and IDEs.".to_string());
    let success_criteria =
        Some("Project-wide style rules are enforced by a shared configuration file.".to_string());

    let editorconfig = repo_root.join(".editorconfig");

    if editorconfig.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: ".editorconfig detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No .editorconfig detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add .editorconfig to standardize formatting — https://editorconfig.org"
                    .to_string(),
            ),
        }
    }
}

fn check_documentation(repo_root: &Path) -> CheckResult {
    let name = "Project documentation";
    let intent = Some(
        "Provide essential project identity, onboarding, and legal/contribution guidance."
            .to_string(),
    );
    let success_criteria = Some(
        "Essential files (README, .gitignore, LICENSE) provide project context and rules."
            .to_string(),
    );

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
            name: name.to_string(),
            status: Status::Pass,
            message: "Complete".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        },
        0 => CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "Not configured".to_string(),
            intent,
            success_criteria,
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
                    name: name.to_string(),
                    status: Status::Info,
                    message: format!("⚠️  Missing critical files: {}", missing.join(", ")),
                    intent,
                    success_criteria,
                    suggestion: Some("Add missing critical documentation files".to_string()),
                }
            } else {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: format!("Partial documentation ({}/4 files)", count),
                    intent,
                    success_criteria,
                    suggestion: Some("Consider adding LICENSE and CONTRIBUTING.md".to_string()),
                }
            }
        }
    }
}

fn check_ai_instructions(repo_root: &Path) -> CheckResult {
    use crate::config::reflections_dir;

    let name = "AI-agent context";
    let intent = Some(
        "Provide persistent \"rules of the road\" and project context for AI collaborators."
            .to_string(),
    );
    let success_criteria = Some(
        "Persistent instructions define coding standards and context for AI assistants."
            .to_string(),
    );

    let claude_md = repo_root.join("CLAUDE.md");
    let agents_md = repo_root.join("AGENTS.md");

    if claude_md.exists() {
        // Check whether wai reflect has been run: a reflection resource file
        // must exist in .wai/resources/reflections/. The old WAI:REFLECT inline
        // block is no longer written — reflect now writes to a resource file and
        // injects a slim WAI:REFLECT:REF reference block instead.
        let refl_dir = reflections_dir(repo_root);
        let has_reflections = refl_dir
            .exists()
            .then(|| std::fs::read_dir(&refl_dir).ok())
            .flatten()
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false);
        let suggestion = if !has_reflections {
            Some(
                "No reflection resource found — run `wai reflect` to synthesize project-specific AI guidance into .wai/resources/reflections/".to_string(),
            )
        } else {
            None
        };
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "CLAUDE.md detected (recommended for Claude Code)".to_string(),
            intent,
            success_criteria,
            suggestion,
        }
    } else if agents_md.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "AGENTS.md detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Consider adding CLAUDE.md for Claude Code compatibility".to_string()),
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No AI instruction files detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Create CLAUDE.md to provide context to AI assistants".to_string()),
        }
    }
}

fn check_ci_cd(repo_root: &Path) -> CheckResult {
    let name = "Automated verification";
    let intent = Some(
        "Ensure code quality and correctness through automated builds and tests on every change."
            .to_string(),
    );
    let success_criteria = Some(
        "Every change is automatically validated by a remote build/test pipeline.".to_string(),
    );

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
                name: name.to_string(),
                status: Status::Pass,
                message: format!("GitHub Actions configured ({} workflow(s))", workflow_count),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: "GitHub Actions directory present but empty".to_string(),
                intent,
                success_criteria,
                suggestion: Some("Add workflow files to .github/workflows/".to_string()),
            }
        }
    } else if gitlab_ci.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "GitLab CI configured".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if circleci.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "CircleCI configured".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No CI/CD configuration detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Set up continuous integration to automate testing".to_string()),
        }
    }
}

fn check_devcontainer(repo_root: &Path) -> CheckResult {
    let name = "Reproducible environments";
    let intent =
        Some("Provide a standardized, containerized environment for all contributors.".to_string());
    let success_criteria = Some(
        "A configuration exists to spin up a consistent, reproducible dev environment.".to_string(),
    );

    let devcontainer_dir = repo_root.join(".devcontainer");
    let devcontainer_json = repo_root.join(".devcontainer.json");

    if devcontainer_dir.exists() && devcontainer_dir.is_dir() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: ".devcontainer/ directory detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if devcontainer_json.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: ".devcontainer.json detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No dev container configuration detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Consider adding .devcontainer/ for reproducible development environments"
                    .to_string(),
            ),
        }
    }
}

fn check_llm_txt(repo_root: &Path) -> CheckResult {
    let name = "LLM-friendly context";
    let intent =
        Some("Provide machine-readable project context and navigation for LLMs.".to_string());
    let success_criteria =
        Some("Machine-readable project documentation (llm.txt) exists for AI tools.".to_string());

    let llm_txt = repo_root.join("llm.txt");

    if llm_txt.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "llm.txt detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No llm.txt detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add llm.txt for AI-friendly project documentation — https://llmstxt.org"
                    .to_string(),
            ),
        }
    }
}

fn check_agent_skills(repo_root: &Path) -> CheckResult {
    let name = "Extended agent capabilities";
    let intent = Some(
        "Enhance agent functionality with specialized iterative review and commit workflows."
            .to_string(),
    );
    let success_criteria =
        Some("Specialized agent workflows (Rule of 5, Deliberate Commits) are active.".to_string());

    let skills_dir = agent_config_dir(repo_root).join(SKILLS_DIR);

    if !skills_dir.exists() {
        return CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No skills configured".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Run 'wai way --fix skills' to scaffold rule-of-5-universal (ro5) and commit"
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
            name: name.to_string(),
            status: Status::Info,
            message: "Skills directory present but empty".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Run 'wai way --fix skills' to scaffold rule-of-5-universal (ro5) and commit"
                    .to_string(),
            ),
        };
    }

    let has_ro5 = skill_ids.contains("rule-of-5-universal") || skill_ids.contains("ro5");
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
            name: name.to_string(),
            status: Status::Pass,
            message: format!(
                "{} skill(s) configured — includes rule-of-5-universal (ro5) and commit",
                skill_count
            ),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: format!(
                "{} skill(s) configured — missing recommended: {}",
                skill_count,
                missing.join(", ")
            ),
            intent,
            success_criteria,
            suggestion: Some(format!(
                "Run 'wai way --fix skills' to scaffold missing: {}",
                missing.join(", ")
            )),
        }
    }
}

fn check_release_pipeline(repo_root: &Path) -> CheckResult {
    let name = "Automated delivery";
    let intent = Some(
        "Automate the process of building, packaging, and publishing software releases."
            .to_string(),
    );
    let success_criteria = Some(
        "Software releases and distribution (packages, binaries) are fully automated.".to_string(),
    );

    if !has_binary_target(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "Library project — release pipeline not required".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if repo_root.join(".goreleaser.yml").exists() || repo_root.join(".goreleaser.yaml").exists() {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "goreleaser detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if repo_root.join("dist.toml").exists() || has_cargo_dist_in_toml(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "cargo-dist detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if has_release_workflow(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "GitHub Actions release workflow detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    CheckResult {
        name: name.to_string(),
        status: Status::Info,
        message: "No release pipeline found".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Consider goreleaser (Go/Rust/any) or cargo-dist (Rust) to automate GitHub releases and publish to Homebrew/Scoop".to_string(),
        ),
    }
}

fn has_binary_target(repo_root: &Path) -> bool {
    // Rust: src/main.rs or [[bin]] in Cargo.toml
    if repo_root.join("src/main.rs").exists() {
        return true;
    }
    if let Ok(content) = std::fs::read_to_string(repo_root.join("Cargo.toml"))
        && content.contains("[[bin]]")
    {
        return true;
    }
    // Go: any top-level .go file with "package main"
    if let Ok(entries) = std::fs::read_dir(repo_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("go")
                && let Ok(content) = std::fs::read_to_string(&path)
                && content.contains("package main")
            {
                return true;
            }
        }
    }
    // Python: [project.scripts] or [tool.poetry.scripts] in pyproject.toml
    if let Ok(content) = std::fs::read_to_string(repo_root.join("pyproject.toml"))
        && (content.contains("[project.scripts]") || content.contains("[tool.poetry.scripts]"))
    {
        return true;
    }
    false
}

fn has_cargo_dist_in_toml(repo_root: &Path) -> bool {
    if let Ok(content) = std::fs::read_to_string(repo_root.join("Cargo.toml")) {
        return content.contains("[workspace.metadata.dist]")
            || content.contains("[package.metadata.dist]");
    }
    false
}

fn has_release_workflow(repo_root: &Path) -> bool {
    let workflows_dir = repo_root.join(".github/workflows");
    if !workflows_dir.exists() {
        return false;
    }
    if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let is_yaml = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "yml" || e == "yaml");
            if !is_yaml {
                continue;
            }
            // Filename contains "release"
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("release"))
            {
                return true;
            }
            // Content has tag trigger matching "v*"
            if let Ok(content) = std::fs::read_to_string(&path)
                && content.contains("tags:")
                && content.contains("v*")
            {
                return true;
            }
        }
    }
    false
}

fn check_test_coverage(repo_root: &Path) -> CheckResult {
    let name = "Test coverage";
    let intent = Some(
        "Enforced coverage thresholds catch regressions automatically and keep quality high."
            .to_string(),
    );
    let success_criteria =
        Some("A coverage tool is configured with a minimum threshold.".to_string());

    // Rust — tarpaulin
    let tarpaulin_toml = repo_root.join("tarpaulin.toml");
    if tarpaulin_toml.exists() {
        let has_threshold = std::fs::read_to_string(&tarpaulin_toml)
            .ok()
            .is_some_and(|c| c.contains("fail-under"));
        return if has_threshold {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "tarpaulin configured (threshold enforced)".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "tarpaulin configured".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Add `fail-under` to tarpaulin config to enforce a minimum — https://github.com/xd009642/tarpaulin".to_string(),
                ),
            }
        };
    }

    // Rust — tarpaulin via Cargo.toml metadata
    if let Ok(cargo_content) = std::fs::read_to_string(repo_root.join("Cargo.toml")) {
        if cargo_content.contains("[package.metadata.tarpaulin]")
            || cargo_content.contains("[workspace.metadata.tarpaulin]")
        {
            let has_threshold = cargo_content.contains("fail-under");
            return if has_threshold {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "tarpaulin configured (threshold enforced)".to_string(),
                    intent,
                    success_criteria,
                    suggestion: None,
                }
            } else {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "tarpaulin configured".to_string(),
                    intent,
                    success_criteria,
                    suggestion: Some(
                        "Add `fail-under` to tarpaulin config to enforce a minimum — https://github.com/xd009642/tarpaulin".to_string(),
                    ),
                }
            };
        }
        // Rust — cargo-llvm-cov via dev-dependencies
        if cargo_content.contains("cargo-llvm-cov") {
            return CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "cargo-llvm-cov detected".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Configure a threshold via CI flags or llvm-cov config — https://github.com/taiki-e/cargo-llvm-cov".to_string(),
                ),
            };
        }
    }

    // Python — coverage.py / pytest-cov
    let coveragerc = repo_root.join(".coveragerc");
    if coveragerc.exists() {
        let has_threshold = std::fs::read_to_string(&coveragerc)
            .ok()
            .is_some_and(|c| c.contains("fail_under"));
        return if has_threshold {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "coverage.py configured (threshold enforced)".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "coverage.py configured".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Add `fail_under` to [tool.coverage.report] in pyproject.toml — https://coverage.readthedocs.io".to_string(),
                ),
            }
        };
    }
    if let Ok(pyproject) = std::fs::read_to_string(repo_root.join("pyproject.toml")) {
        if pyproject.contains("[tool.coverage.report]") {
            let has_threshold = pyproject.contains("fail_under");
            return if has_threshold {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "coverage.py configured (threshold enforced)".to_string(),
                    intent,
                    success_criteria,
                    suggestion: None,
                }
            } else {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "coverage.py configured".to_string(),
                    intent,
                    success_criteria,
                    suggestion: Some(
                        "Add `fail_under` to [tool.coverage.report] in pyproject.toml — https://coverage.readthedocs.io".to_string(),
                    ),
                }
            };
        }
    }

    // JavaScript / TypeScript — vitest
    for config_name in &["vitest.config.ts", "vitest.config.js", "vitest.config.mts"] {
        let config_path = repo_root.join(config_name);
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if content.contains("coverage") {
                let has_threshold = content.contains("thresholds");
                return if has_threshold {
                    CheckResult {
                        name: name.to_string(),
                        status: Status::Pass,
                        message: "vitest coverage configured (threshold enforced)".to_string(),
                        intent,
                        success_criteria,
                        suggestion: None,
                    }
                } else {
                    CheckResult {
                        name: name.to_string(),
                        status: Status::Pass,
                        message: "vitest coverage configured".to_string(),
                        intent,
                        success_criteria,
                        suggestion: Some(
                            "Add `thresholds` to the coverage block in vitest.config — https://vitest.dev/config/#coverage".to_string(),
                        ),
                    }
                };
            }
        }
    }

    // JavaScript / TypeScript — nyc or c8
    let nycrc = repo_root.join(".nycrc");
    let nycrc_json = repo_root.join(".nycrc.json");
    let c8_config = repo_root.join("c8.config.js");
    if nycrc.exists() || nycrc_json.exists() || c8_config.exists() {
        let content = nycrc
            .exists()
            .then(|| std::fs::read_to_string(&nycrc).ok())
            .flatten()
            .or_else(|| {
                nycrc_json
                    .exists()
                    .then(|| std::fs::read_to_string(&nycrc_json).ok())
                    .flatten()
            })
            .or_else(|| {
                c8_config
                    .exists()
                    .then(|| std::fs::read_to_string(&c8_config).ok())
                    .flatten()
            })
            .unwrap_or_default();
        let has_threshold = content.contains("branches") || content.contains("lines");
        return if has_threshold {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "nyc/c8 configured (threshold enforced)".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "nyc/c8 configured".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Add branch/line thresholds to nyc or c8 config — https://github.com/istanbuljs/nyc · https://github.com/bcoe/c8".to_string(),
                ),
            }
        };
    }
    if let Ok(pkg) = std::fs::read_to_string(repo_root.join("package.json")) {
        if pkg.contains("\"nyc\"") || pkg.contains("\"c8\"") {
            let has_threshold = pkg.contains("branches") || pkg.contains("lines");
            return if has_threshold {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "nyc/c8 configured (threshold enforced)".to_string(),
                    intent,
                    success_criteria,
                    suggestion: None,
                }
            } else {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "nyc/c8 configured".to_string(),
                    intent,
                    success_criteria,
                    suggestion: Some(
                        "Add branch/line thresholds to nyc or c8 config — https://github.com/istanbuljs/nyc · https://github.com/bcoe/c8".to_string(),
                    ),
                }
            };
        }
    }

    // Any language — codecov / coveralls
    let codecov_yml = repo_root.join(".codecov.yml");
    let codecov_yaml = repo_root.join("codecov.yml");
    let coveralls_yml = repo_root.join(".coveralls.yml");
    if codecov_yml.exists() || codecov_yaml.exists() || coveralls_yml.exists() {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "Coverage reporting service detected (codecov/coveralls)".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add a coverage threshold in the config to enforce minimums".to_string(),
            ),
        };
    }

    // Nothing detected
    CheckResult {
        name: name.to_string(),
        status: Status::Info,
        message: "No coverage tool configured".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Configure a coverage tool with an enforced threshold — Rust: https://github.com/xd009642/tarpaulin · Python: https://coverage.readthedocs.io · JS: https://github.com/istanbuljs/nyc".to_string(),
        ),
    }
}

fn check_beads(repo_root: &Path) -> CheckResult {
    let name = "Issue tracking";
    let intent = Some(
        "Structured task tracking keeps work visible and prevents context loss across sessions."
            .to_string(),
    );
    let success_criteria = Some(
        "A beads workspace (.beads/) exists for tracking issues and dependencies.".to_string(),
    );

    let beads_dir = repo_root.join(".beads");
    if beads_dir.exists() && beads_dir.is_dir() {
        let issues_jsonl = beads_dir.join("issues.jsonl");
        let message = if let Ok(content) = std::fs::read_to_string(&issues_jsonl) {
            let count = content.lines().filter(|l| !l.trim().is_empty()).count();
            format!("beads detected ({} issues tracked)", count)
        } else {
            "beads detected".to_string()
        };
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message,
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No issue tracker detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Initialize beads issue tracking — https://github.com/steveyegge/beads"
                    .to_string(),
            ),
        }
    }
}

fn check_openspec(repo_root: &Path) -> CheckResult {
    let name = "Change proposals";
    let intent = Some(
        "Formal change proposals prevent architectural drift and create a reviewable design record."
            .to_string(),
    );
    let success_criteria = Some(
        "An openspec workspace (openspec/) exists for managing change proposals.".to_string(),
    );

    let openspec_dir = repo_root.join("openspec");
    if openspec_dir.exists() && openspec_dir.is_dir() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "openspec detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No change proposal system detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Track architectural change proposals — https://github.com/Fission-AI/OpenSpec"
                    .to_string(),
            ),
        }
    }
}

fn check_gh_cli() -> CheckResult {
    let name = "Integration & automation";
    let intent = Some(
        "Streamline repository interactions (PRs, issues, releases) from the CLI.".to_string(),
    );
    let success_criteria = Some(
        "CLI tools are configured for seamless integration with the hosting provider.".to_string(),
    );

    let gh_installed = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .is_ok();

    if !gh_installed {
        return CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "gh not installed".to_string(),
            intent,
            success_criteria,
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
            name: name.to_string(),
            status: Status::Pass,
            message: "gh installed and authenticated".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        },
        _ => CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "gh installed but not authenticated".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Run 'gh auth login' to authenticate".to_string()),
        },
    }
}
