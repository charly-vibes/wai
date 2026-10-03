use std::path::Path;

use super::WayCheckEntry;
use genesis::doctor::CheckStatus;

pub(super) fn check_typos(repo_root: &Path) -> WayCheckEntry {
    let name = "Spell checking";
    let intent = Some(
        "Catch typos in source code, comments, and documentation before they reach history."
            .to_string(),
    );
    let success_criteria =
        Some("A typos configuration exists to enforce spell checking.".to_string());

    let configs = [
        repo_root.join("typos.toml"),
        repo_root.join("_typos.toml"),
        repo_root.join(".typos.toml"),
    ];

    if let Some(found) = configs.iter().find(|p| p.exists()) {
        return WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "typos configured ({})",
                found.file_name().unwrap_or_default().to_string_lossy()
            ),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    // Check pyproject.toml for [tool.typos]
    if let Ok(content) = std::fs::read_to_string(repo_root.join("pyproject.toml"))
        && content.contains("[tool.typos]")
    {
        return WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: "typos configured (pyproject.toml)".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    WayCheckEntry {
        name: name.to_string(),
        status: CheckStatus::Warn,
        message: "No typos configuration detected".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Add _typos.toml to catch spelling errors in code — https://github.com/crate-ci/typos"
                .to_string(),
        ),
    }
}

pub(super) fn check_vale(repo_root: &Path) -> WayCheckEntry {
    let name = "Prose linting";
    let intent = Some(
        "Enforce writing style and consistency across documentation and markdown files."
            .to_string(),
    );
    let success_criteria =
        Some("A vale configuration exists to enforce prose style rules.".to_string());

    let configs = [repo_root.join(".vale.ini"), repo_root.join("vale.ini")];

    if let Some(found) = configs.iter().find(|p| p.exists()) {
        return WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "vale configured ({})",
                found.file_name().unwrap_or_default().to_string_lossy()
            ),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    WayCheckEntry {
        name: name.to_string(),
        status: CheckStatus::Warn,
        message: "No vale configuration detected".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Add .vale.ini to lint prose in markdown and docs — https://vale.sh".to_string(),
        ),
    }
}

fn is_shell_path(p: &std::path::Path) -> bool {
    p.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "sh" || ext == "bash")
}

fn dir_has_shell_files(dir: &std::path::Path) -> bool {
    dir.is_dir()
        && std::fs::read_dir(dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| is_shell_path(&e.path()))
            })
            .unwrap_or(false)
}

/// Whether the repo contains shell scripts worth linting.
fn has_shell_scripts(repo_root: &Path) -> bool {
    ["scripts", "bin", "script"]
        .iter()
        .any(|d| dir_has_shell_files(&repo_root.join(d)))
        || std::fs::read_dir(repo_root)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| is_shell_path(&e.path()))
            })
            .unwrap_or(false)
}

fn tool_available(bin: &str) -> bool {
    std::process::Command::new(bin)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Outcome for the actionlint/shellcheck/workflows matrix.
fn linter_outcome(
    has_actionlint: bool,
    has_shellcheck: bool,
    has_workflows: bool,
) -> (CheckStatus, String, Option<String>) {
    match (has_actionlint, has_shellcheck, has_workflows) {
        (true, true, _) => (
            CheckStatus::Pass,
            "actionlint and shellcheck available".to_string(),
            None,
        ),
        (true, false, _) => (
            CheckStatus::Pass,
            "actionlint available (install shellcheck for deeper run: block analysis)".to_string(),
            Some(
                "Install shellcheck so actionlint can lint embedded run: blocks — https://www.shellcheck.net"
                    .to_string(),
            ),
        ),
        (false, true, false) => (
            CheckStatus::Pass,
            "shellcheck available".to_string(),
            None,
        ),
        (false, true, true) => (
            CheckStatus::Warn,
            "shellcheck available but actionlint missing for workflow linting".to_string(),
            Some(
                "Install actionlint to lint GitHub Actions workflows (it uses shellcheck for run: blocks) — https://github.com/rhysd/actionlint"
                    .to_string(),
            ),
        ),
        (false, false, true) => (
            CheckStatus::Warn,
            "No shell linter detected".to_string(),
            Some(
                "Install actionlint + shellcheck to lint workflows and shell scripts — https://github.com/rhysd/actionlint"
                    .to_string(),
            ),
        ),
        (false, false, false) => (
            CheckStatus::Warn,
            "No shell linter detected".to_string(),
            Some(
                "Install shellcheck to lint shell scripts — https://www.shellcheck.net".to_string(),
            ),
        ),
    }
}

pub(super) fn check_shell_linting(repo_root: &Path) -> WayCheckEntry {
    let name = "Shell linting";
    let intent = Some(
        "Catch bugs, portability issues, and bad practices in shell scripts and CI workflow run blocks."
            .to_string(),
    );
    let success_criteria = Some(
        "Shell scripts and CI run blocks are validated by a linter (actionlint or shellcheck)."
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

    // Detect shell-containing files
    let has_workflows = repo_root.join(".github/workflows").is_dir();
    let has_scripts = has_shell_scripts(repo_root);

    if !has_workflows && !has_scripts {
        return mk(
            CheckStatus::Pass,
            "No shell scripts or workflows to lint".to_string(),
            None,
        );
    }

    // Check for actionlint (covers both workflow YAML and embedded shell via shellcheck)
    let has_actionlint = tool_available("actionlint");
    let has_shellcheck = tool_available("shellcheck");

    let (status, message, suggestion) =
        linter_outcome(has_actionlint, has_shellcheck, has_workflows);
    mk(status, message, suggestion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_git_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        dir
    }

    // -- shell linting --

    #[test]
    fn shell_linting_passes_when_no_shell_content() {
        let dir = setup_git_repo();
        // No workflows, no .sh files → nothing to lint
        let result = check_shell_linting(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(
            result.message.contains("No shell scripts or workflows"),
            "unexpected message: {}",
            result.message
        );
    }

    #[test]
    fn shell_linting_detects_workflows_dir() {
        let dir = setup_git_repo();
        let workflows = dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(workflows.join("ci.yml"), "name: CI\n").unwrap();

        let result = check_shell_linting(dir.path());
        // Should not be "no shell content" — workflows exist
        assert!(
            !result.message.contains("No shell scripts or workflows"),
            "should detect workflows, got: {}",
            result.message
        );
    }

    #[test]
    fn shell_linting_ignores_scripts_dir_without_sh_files() {
        let dir = setup_git_repo();
        let scripts = dir.path().join("scripts");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(scripts.join("deploy.py"), "print('hi')\n").unwrap();

        let result = check_shell_linting(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(
            result.message.contains("No shell scripts or workflows"),
            "should ignore scripts dir without .sh files, got: {}",
            result.message
        );
    }

    #[test]
    fn shell_linting_detects_scripts_dir_with_sh_files() {
        let dir = setup_git_repo();
        let scripts = dir.path().join("scripts");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(scripts.join("deploy.sh"), "#!/bin/bash\necho hi\n").unwrap();

        let result = check_shell_linting(dir.path());
        assert!(
            !result.message.contains("No shell scripts or workflows"),
            "should detect scripts dir with .sh files, got: {}",
            result.message
        );
    }

    #[test]
    fn shell_linting_detects_root_sh_files() {
        let dir = setup_git_repo();
        fs::write(dir.path().join("setup.sh"), "#!/bin/bash\necho hi\n").unwrap();

        let result = check_shell_linting(dir.path());
        assert!(
            !result.message.contains("No shell scripts or workflows"),
            "should detect .sh files, got: {}",
            result.message
        );
    }
}
