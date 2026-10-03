//! Repo-convention checks for `wai way`.

use genesis::doctor::CheckStatus;
use std::path::Path;

use crate::commands::way::WayCheckEntry;

pub(crate) fn check_task_runner(repo_root: &Path) -> WayCheckEntry {
    let name = "Command standardization";
    let intent = Some("Provide a single, tool-agnostic entry point for common repository tasks (build, test, deploy).".to_string());
    let success_criteria = Some(
        "A standard interface (justfile, Makefile, npm scripts) exists for common tasks."
            .to_string(),
    );
    let mk = |status: CheckStatus, message: String, suggestion: Option<String>| -> WayCheckEntry {
        WayCheckEntry {
            name: name.to_string(),
            status,
            message,
            intent,
            success_criteria,
            suggestion,
        }
    };

    let justfile = repo_root.join("justfile");
    let makefile = repo_root.join("Makefile");

    if justfile.exists() {
        let recipes = parse_justfile_recipes(&justfile);
        let message = if recipes.is_empty() {
            "justfile detected".to_string()
        } else {
            format!("justfile detected (recipes: {})", recipes.join(", "))
        };
        mk(CheckStatus::Pass, message, None)
    } else if makefile.exists() {
        mk(
            CheckStatus::Pass,
            "Makefile detected".to_string(),
            Some(
                "Consider migrating to justfile for better ergonomics — https://just.systems"
                    .to_string(),
            ),
        )
    } else if repo_root.join("mise.toml").exists() || repo_root.join(".mise.toml").exists() {
        mk(
            CheckStatus::Pass,
            "mise.toml detected".to_string(),
            Some(
                "Consider migrating to justfile for better ergonomics — https://just.systems"
                    .to_string(),
            ),
        )
    } else {
        mk(
            CheckStatus::Warn,
            "No task runner detected".to_string(),
            Some("Add a justfile to standardize common tasks — https://just.systems".to_string()),
        )
    }
}

pub(crate) fn parse_justfile_recipes(justfile_path: &Path) -> Vec<String> {
    let known_recipes = [
        "install", "serve", "dev", "test", "lint", "fmt", "format", "release", "build", "run",
        "clean", "check", "watch", "docs",
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

pub(crate) fn check_editorconfig(repo_root: &Path) -> WayCheckEntry {
    let name = "Consistent formatting";
    let intent =
        Some("Ensure consistent code formatting across different editors and IDEs.".to_string());
    let success_criteria =
        Some("Project-wide style rules are enforced by a shared configuration file.".to_string());

    let editorconfig = repo_root.join(".editorconfig");

    if editorconfig.exists() {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: ".editorconfig detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
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

pub(crate) fn detect_doc_tool(repo_root: &Path) -> Option<String> {
    if repo_root.join("Cargo.toml").exists() {
        return Some("cargo doc".to_string());
    }
    if repo_root.join("mkdocs.yml").exists() || repo_root.join("mkdocs.yaml").exists() {
        return Some("mkdocs".to_string());
    }
    if repo_root.join("docs").join("conf.py").exists() {
        return Some("sphinx".to_string());
    }
    if repo_root.join("typedoc.json").exists() || repo_root.join(".typedoc.json").exists() {
        return Some("typedoc".to_string());
    }
    if repo_root.join("go.mod").exists() {
        return Some("godoc".to_string());
    }
    None
}

pub(crate) fn check_ci_cd(repo_root: &Path) -> WayCheckEntry {
    let name = "Automated verification";
    let intent = Some(
        "Ensure code quality and correctness through automated builds and tests on every change."
            .to_string(),
    );
    let success_criteria = Some(
        "Every change is automatically validated by a remote build/test pipeline.".to_string(),
    );
    let mk = |status: CheckStatus, message: String, suggestion: Option<String>| -> WayCheckEntry {
        WayCheckEntry {
            name: name.to_string(),
            status,
            message,
            intent,
            success_criteria,
            suggestion,
        }
    };

    let github_workflows = repo_root.join(".github/workflows");
    let gitlab_ci = repo_root.join(".gitlab-ci.yml");
    let circleci = repo_root.join(".circleci/config.yml");

    if github_workflows.exists() && github_workflows.is_dir() {
        let workflow_count = std::fs::read_dir(&github_workflows)
            .ok()
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        if workflow_count > 0 {
            mk(
                CheckStatus::Pass,
                format!("GitHub Actions configured ({} workflow(s))", workflow_count),
                None,
            )
        } else {
            mk(
                CheckStatus::Warn,
                "GitHub Actions directory present but empty".to_string(),
                Some("Add workflow files to .github/workflows/".to_string()),
            )
        }
    } else if gitlab_ci.exists() {
        mk(CheckStatus::Pass, "GitLab CI configured".to_string(), None)
    } else if circleci.exists() {
        mk(CheckStatus::Pass, "CircleCI configured".to_string(), None)
    } else {
        mk(
            CheckStatus::Warn,
            "No CI/CD configuration detected".to_string(),
            Some("Set up continuous integration to automate testing".to_string()),
        )
    }
}

pub(crate) fn check_devcontainer(repo_root: &Path) -> WayCheckEntry {
    let name = "Reproducible environments";
    let intent =
        Some("Provide a standardized, containerized environment for all contributors.".to_string());
    let success_criteria = Some(
        "A configuration exists to spin up a consistent, reproducible dev environment.".to_string(),
    );

    let devcontainer_dir = repo_root.join(".devcontainer");
    let devcontainer_json = repo_root.join(".devcontainer.json");

    if devcontainer_dir.exists() && devcontainer_dir.is_dir() {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: ".devcontainer/ directory detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if devcontainer_json.exists() {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: ".devcontainer.json detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
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

pub(crate) fn check_llm_txt(repo_root: &Path) -> WayCheckEntry {
    let name = "LLM-friendly context";
    let intent =
        Some("Provide machine-readable project context and navigation for LLMs.".to_string());
    let success_criteria =
        Some("Machine-readable project documentation (llm.txt) exists for AI tools.".to_string());

    let llm_txt = repo_root.join("llm.txt");

    if llm_txt.exists() {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: "llm.txt detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
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

pub(crate) fn check_beads(repo_root: &Path) -> WayCheckEntry {
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
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message,
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: "No issue tracker detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Initialize beads issue tracking — https://github.com/steveyegge/beads".to_string(),
            ),
        }
    }
}

pub(crate) fn check_openspec(repo_root: &Path) -> WayCheckEntry {
    let name = "Change proposals";
    let intent = Some(
        "Formal change proposals prevent architectural drift and create a reviewable design record."
            .to_string(),
    );
    let success_criteria =
        Some("An openspec workspace (openspec/) exists for managing change proposals.".to_string());

    let openspec_dir = repo_root.join("openspec");
    if openspec_dir.exists() && openspec_dir.is_dir() {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: "openspec detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
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

pub(crate) fn check_pretender(repo_root: &Path) -> WayCheckEntry {
    let name = "Code quality (pretender)";
    let intent = Some(
        "Enforce structural code quality thresholds (cyclomatic complexity, nesting, duplication) across multiple languages."
            .to_string(),
    );
    let success_criteria =
        Some("pretender.toml at repo root with threshold configuration.".to_string());

    let pretender_config = repo_root.join("pretender.toml");

    if pretender_config.exists() {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: "pretender.toml detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: "No pretender.toml detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Run 'pretender init --non-interactive' to set up code quality checks — https://github.com/charly-vibes/pretender"
                    .to_string(),
            ),
        }
    }
}

pub(crate) fn check_gh_cli() -> WayCheckEntry {
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
        return WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
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
        Ok(output) if output.status.success() => WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: "gh installed and authenticated".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        },
        _ => WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: "gh installed but not authenticated".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Run 'gh auth login' to authenticate".to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::way::check_artifact_stubs;
    use std::fs;

    #[test]
    fn task_runner_detects_mise_toml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("mise.toml"), "[tools]\n").unwrap();

        let result = check_task_runner(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(
            result.message.contains("mise.toml detected"),
            "expected mise.toml detected, got: {}",
            result.message
        );
    }

    #[test]
    fn task_runner_detects_dot_mise_toml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".mise.toml"), "[tools]\n").unwrap();

        let result = check_task_runner(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(
            result.message.contains("mise.toml detected"),
            "expected mise.toml detected, got: {}",
            result.message
        );
    }

    #[test]
    fn artifact_stubs_flags_empty_research_doc() {
        let dir = tempfile::tempdir().unwrap();
        let research = dir.path().join(".wai/projects/demo/research");
        fs::create_dir_all(&research).unwrap();
        fs::write(
            research.join("2026-07-29-investigation.md"),
            "---\ntags: [x]\n---\n\ninvestigation: findings from 2026-07-29\n",
        )
        .unwrap();

        let result = check_artifact_stubs(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(
            result.message.contains("2026-07-29-investigation.md"),
            "expected stub filename in message, got: {}",
            result.message
        );
    }

    #[test]
    fn artifact_stubs_accepts_structured_single_line_records() {
        let dir = tempfile::tempdir().unwrap();
        let research = dir.path().join(".wai/projects/demo/research");
        fs::create_dir_all(&research).unwrap();
        // Pipeline-step record convention: "KEY: issue; field=value; ..."
        for (name, line) in [
            (
                "2026-05-13-ro5u-fixes-wai-fvhv-109-fixed-none-required-beca.md",
                "RO5U-FIXES: wai-fvhv.109; fixed=none required because RO5 review reported 0 critical; verified=cargo test, 60 passed.",
            ),
            (
                "2026-05-13-analysis-start-wai-fvhv-64.md",
                "ANALYSIS START: wai-fvhv.64; artifact=bead notes; validation=compare output; non-code review only.",
            ),
        ] {
            fs::write(
                research.join(name),
                format!("---\ntags: [pipeline]\n---\n\n{}\n", line),
            )
            .unwrap();
        }

        let result = check_artifact_stubs(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn artifact_stubs_flags_title_only_doc() {
        let dir = tempfile::tempdir().unwrap();
        let research = dir.path().join(".wai/projects/demo/research");
        fs::create_dir_all(&research).unwrap();
        // Title-only line starting lowercase: a promise of findings with none captured.
        fs::write(
            research.join("2026-07-29-pipeline-utilization-investigation-findings-from.md"),
            "---\ntags: [x]\n---\n\npipeline-utilization-investigation: findings from 2026-07-29\n",
        )
        .unwrap();

        let result = check_artifact_stubs(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
    }

    #[test]
    fn artifact_stubs_passes_on_substantive_docs() {
        let dir = tempfile::tempdir().unwrap();
        let research = dir.path().join(".wai/projects/demo/handoffs");
        fs::create_dir_all(&research).unwrap();
        let body = "line one\nline two\nline three\nline four\nline five\nline six";
        fs::write(research.join("2026-09-17-session-end.md"), body).unwrap();

        let result = check_artifact_stubs(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn artifact_stubs_scans_all_artifact_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join(".wai/projects/demo");
        for subdir in ["research", "handoffs", "designs", "plans", "reviews"] {
            let d = base.join(subdir);
            fs::create_dir_all(&d).unwrap();
            // Substantive doc so only an intentionally planted stub gets flagged.
            fs::write(
                d.join("substantive.md"),
                "---\ntags: []\n---\n\nline one\nline two\nline three\nline four\nline five\nline six\n",
            )
            .unwrap();
        }
        // One true stub in designs/ — the dir a research+handoffs-only scan misses.
        fs::write(
            base.join("designs/2026-05-13-forgotten-findings.md"),
            "---\ntags: []\n---\n\nforgotten-findings: details to follow\n",
        )
        .unwrap();

        let result = check_artifact_stubs(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(
            result
                .message
                .contains("designs/2026-05-13-forgotten-findings.md"),
            "expected designs/ stub flagged, got: {}",
            result.message
        );
    }

    #[test]
    fn artifact_stubs_accepts_multiline_structured_records() {
        let dir = tempfile::tempdir().unwrap();
        let designs = dir.path().join(".wai/projects/demo/designs");
        fs::create_dir_all(&designs).unwrap();
        // Structured record header plus a verification paragraph (2 body lines).
        fs::write(
            designs.join("2026-05-13-tidy-wai-fvhv-108.md"),
            "---\ntags: []\n---\n\nTIDY: wai-fvhv.108; no refactoring needed — helper is distinct\n\nVerification: non-code work, no refactoring applied. Tests pass.\n",
        )
        .unwrap();

        let result = check_artifact_stubs(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn artifact_stubs_passes_when_no_project_artifacts() {
        let dir = tempfile::tempdir().unwrap();

        let result = check_artifact_stubs(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn task_runner_no_runner_still_reports_info() {
        let dir = tempfile::tempdir().unwrap();

        let result = check_task_runner(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(
            result.message.contains("No task runner detected"),
            "expected No task runner detected, got: {}",
            result.message
        );
    }
}
