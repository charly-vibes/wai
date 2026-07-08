use std::path::Path;

use super::{CheckResult, Status};

/// Read a hook file from an arbitrary path. Returns `None` if the file does not
/// exist, is a directory, or cannot be read.
pub(super) fn read_hook_from_path(path: &Path) -> Option<String> {
    if !path.exists() || path.is_dir() {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

pub(super) fn read_hook(repo_root: &Path, hook_name: &str) -> Option<String> {
    let hook_path = repo_root.join(".git/hooks").join(hook_name);
    if !hook_path.exists() || hook_path.is_dir() {
        return None;
    }
    std::fs::read_to_string(&hook_path).ok()
}

/// Return `true` if pre-commit **or** pre-push hook contains `needle`.
pub(super) fn hook_contains(repo_root: &Path, needle: &str) -> bool {
    read_hook(repo_root, "pre-commit").is_some_and(|c| c.contains(needle))
        || read_hook(repo_root, "pre-push").is_some_and(|c| c.contains(needle))
}

/// Return `true` if the pre-commit hook file exists and is non-empty.
pub(super) fn hook_exists_nonempty(repo_root: &Path) -> bool {
    read_hook(repo_root, "pre-commit").is_some_and(|c| !c.trim().is_empty())
}

/// Return the repo-local `core.hooksPath` git config value, else `None`.
///
/// Only checks `--local` scope. Global/system `core.hooksPath` is a machine-level
/// setting outside the repo's control and not something a repo best-practices
/// check should flag.
pub(super) fn git_core_hooks_path(repo_root: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args([
            "-C",
            &repo_root.to_string_lossy(),
            "config",
            "--local",
            "core.hooksPath",
        ])
        .output()
        .ok()?;
    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !val.is_empty() { Some(val) } else { None }
    } else {
        None
    }
}

/// Identify which known tool owns the pre-commit hook, or `None`.
///
/// Checks for the same sigils that the rest of `check_git_hooks` uses:
/// `bd`, `lefthook`, `husky`, `pre-commit` (the framework).
pub(super) fn hook_owner(repo_root: &Path) -> Option<&'static str> {
    let content = read_hook(repo_root, "pre-commit")?;
    // Check more-specific patterns first to avoid false matches.
    for (needle, name) in &[
        ("lefthook", "lefthook"),
        ("husky", "husky"),
        ("bd", "bd"),
        ("pre-commit", "pre-commit"),
    ] {
        if content.contains(needle) {
            return Some(name);
        }
    }
    None
}

pub(super) fn check_git_hooks(repo_root: &Path) -> CheckResult {
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
        // core.hooksPath being set means prek refuses to install — report this first.
        if let Some(hooks_path) = git_core_hooks_path(repo_root) {
            return CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: format!(
                    "prek.toml found but core.hooksPath is set ('{}') — prek cannot install",
                    hooks_path
                ),
                intent,
                success_criteria,
                suggestion: Some(
                    "Unset it first: git config --local --unset core.hooksPath".to_string(),
                ),
            };
        }
        // Check both pre-commit and pre-push for prek signature.
        if hook_contains(repo_root, "prek") {
            return CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "prek detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            };
        }
        // Another tool owns the hook — name it rather than suggest a re-install.
        if let Some(owner) = hook_owner(repo_root) {
            return CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: format!(
                    "prek.toml found but hook is owned by {} — chain prek or use {}'s runner",
                    owner, owner
                ),
                intent,
                success_criteria,
                suggestion: None,
            };
        }
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "prek.toml found but hooks not installed — run: prek install".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if lefthook_config.exists() || lefthook_config_yaml.exists() {
        // Check the delegated core.hooksPath first (e.g., when beads/bd sets hooksPath)
        if let Some(hooks_path) = git_core_hooks_path(repo_root) {
            let delegated_precommit = repo_root.join(&hooks_path).join("pre-commit");
            let delegated_prepush = repo_root.join(&hooks_path).join("pre-push");

            let lefthook_in_delegated = read_hook_from_path(&delegated_precommit)
                .is_some_and(|c| c.contains("lefthook"))
                || read_hook_from_path(&delegated_prepush).is_some_and(|c| c.contains("lefthook"));

            if lefthook_in_delegated {
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
                    message: format!(
                        "lefthook.yml found but core.hooksPath is set ('{}') — lefthook not found in delegated path",
                        hooks_path,
                    ),
                    intent,
                    success_criteria,
                    suggestion: Some("Run: lefthook install".to_string()),
                }
            }
        } else if hook_contains(repo_root, "lefthook") {
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

    fn write_hook(dir: &tempfile::TempDir, hook_name: &str, content: &str) {
        let hooks_dir = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        fs::write(hooks_dir.join(hook_name), content).unwrap();
    }

    fn write_prek_toml(dir: &tempfile::TempDir) {
        fs::write(
            dir.path().join("prek.toml"),
            "[hooks]\npre-commit = [\"cargo test\"]\n",
        )
        .unwrap();
    }

    // -- core.hooksPath conflict --

    #[test]
    fn prek_toml_with_core_hooks_path_set_reports_conflict() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        std::process::Command::new("git")
            .args(["config", "core.hooksPath", ".git/hooks"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("core.hooksPath"),
            "expected core.hooksPath conflict, got: {}",
            result.message
        );
        let suggestion = result.suggestion.unwrap_or_default();
        assert!(
            suggestion.contains("unset"),
            "expected unset hint, got: {}",
            suggestion
        );
    }

    // -- hook owner detection --

    #[test]
    fn prek_toml_with_bd_shim_reports_bd_as_owner() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# bd-shim v2\nexec bd hooks run pre-commit \"$@\"\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("bd"),
            "expected bd as owner, got: {}",
            result.message
        );
    }

    #[test]
    fn prek_toml_with_lefthook_shim_reports_lefthook_as_owner() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\nexec lefthook run pre-commit \"$@\"\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("lefthook"),
            "expected lefthook as owner, got: {}",
            result.message
        );
    }

    // -- prek pass cases --

    #[test]
    fn prek_toml_with_prek_in_precommit_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# managed by prek\nexec prek run pre-commit\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Pass);
        assert_eq!(result.message, "prek detected and installed");
    }

    #[test]
    fn prek_toml_with_prek_in_prepush_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-push",
            "#!/usr/bin/env sh\n# managed by prek\nexec prek run pre-push\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Pass);
        assert_eq!(result.message, "prek detected and installed");
    }

    #[test]
    fn bd_hook_that_chains_prek_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# bd-shim v2\n# chains prek\nexec prek run pre-commit\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Pass);
    }

    // -- lefthook via delegated core.hooksPath --

    #[test]
    fn lefthook_yml_with_delegated_hooks_path_detects_lefthook() {
        let dir = setup_git_repo();
        // Write lefthook.yml
        fs::write(dir.path().join("lefthook.yml"), "pre-commit: [echo test]\n").unwrap();
        // Set core.hooksPath to a custom directory
        let custom_hooks = dir.path().join(".my-hooks");
        fs::create_dir_all(&custom_hooks).unwrap();
        std::process::Command::new("git")
            .args(["config", "core.hooksPath", ".my-hooks"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        // Write a lefthook shim in the delegated hooks directory
        fs::write(
            custom_hooks.join("pre-commit"),
            "#!/usr/bin/env sh\nexec lefthook run pre-commit \"$@\"\n",
        )
        .unwrap();

        let result = check_git_hooks(dir.path());
        assert_eq!(
            result.status,
            Status::Pass,
            "expected Pass for lefthook via delegated hooksPath, got: {:?} — {}",
            result.status,
            result.message
        );
        assert!(
            result.message.contains("lefthook"),
            "expected message to mention lefthook, got: {}",
            result.message
        );
    }

    #[test]
    fn lefthook_yml_with_delegated_hooks_path_missing_lefthook_reports_conflict() {
        let dir = setup_git_repo();
        // Write lefthook.yml
        fs::write(dir.path().join("lefthook.yml"), "pre-commit: [echo test]\n").unwrap();
        // Set core.hooksPath to a custom directory but don't put lefthook there
        let custom_hooks = dir.path().join(".other-hooks");
        fs::create_dir_all(&custom_hooks).unwrap();
        std::process::Command::new("git")
            .args(["config", "core.hooksPath", ".other-hooks"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        // Write a non-lefthook script in the delegated hooks directory
        fs::write(
            custom_hooks.join("pre-commit"),
            "#!/usr/bin/env sh\necho \"some other hook\"\n",
        )
        .unwrap();

        let result = check_git_hooks(dir.path());
        assert_eq!(
            result.status,
            Status::Info,
            "expected Info for unresolved hooksPath conflict, got: {:?} — {}",
            result.status,
            result.message
        );
        assert!(
            result.message.contains("hooksPath"),
            "expected message to mention hooksPath conflict, got: {}",
            result.message
        );
    }

    #[test]
    fn lefthook_yml_without_any_hooks_still_suggests_install() {
        let dir = setup_git_repo();
        // Write lefthook.yml
        fs::write(dir.path().join("lefthook.yml"), "pre-commit: [echo test]\n").unwrap();

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("lefthook install"),
            "expected lefthook install suggestion, got: {}",
            result.message
        );
    }

    // -- no hooks at all --

    #[test]
    fn prek_toml_with_no_hooks_suggests_install() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("prek install"),
            "expected prek install suggestion, got: {}",
            result.message
        );
    }

    // -- no prek.toml --

    #[test]
    fn no_prek_toml_does_not_fire_prek_branch() {
        let dir = setup_git_repo();
        // No prek.toml written — prek branch must not trigger.

        let result = check_git_hooks(dir.path());
        assert!(
            !result.message.contains("prek.toml found"),
            "prek branch fired without prek.toml: {}",
            result.message
        );
    }
}
