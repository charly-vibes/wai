use std::path::{Path, PathBuf};

use super::WayCheckEntry;
use genesis::doctor::CheckStatus;
use genesis::git_hooks::{self, EffectiveHooksDir, HookName, HooksDirScope, Owner};

/// The hooks directory git will invoke hooks from, honoring a repo-local
/// `core.hooksPath` (genesis `effective_hooks_dir`). An empty/disabled value,
/// a global/system-scope setting, or an unset value all fall back to the
/// default `.git/hooks` — matching the previous local logic, which only
/// reacted to a repo-local setting.
fn local_or_default_hooks_dir(repo_root: &Path) -> PathBuf {
    match git_hooks::effective_hooks_dir(repo_root) {
        Ok(EffectiveHooksDir {
            path,
            scope: HooksDirScope::Local,
        }) => path,
        _ => repo_root.join(".git/hooks"),
    }
}

/// Read a hook file from the effective hooks directory. Returns `None` if
/// the file does not exist or cannot be read.
fn read_hook(repo_root: &Path, hook_name: &str) -> Option<String> {
    std::fs::read_to_string(local_or_default_hooks_dir(repo_root).join(hook_name)).ok()
}

/// Return `true` if pre-commit **or** pre-push hook contains `needle`.
fn hook_contains(repo_root: &Path, needle: &str) -> bool {
    read_hook(repo_root, "pre-commit").is_some_and(|c| c.contains(needle))
        || read_hook(repo_root, "pre-push").is_some_and(|c| c.contains(needle))
}

/// Return `true` if the pre-commit hook file exists and is non-empty.
fn hook_exists_nonempty(repo_root: &Path) -> bool {
    read_hook(repo_root, "pre-commit").is_some_and(|c| !c.trim().is_empty())
}

/// Human-readable name for a hook owner, for user-facing messages.
fn owner_name(owner: Owner) -> &'static str {
    match owner {
        Owner::Lefthook => "lefthook",
        Owner::Husky => "husky",
        Owner::Bd => "bd",
        Owner::Prek => "prek",
        Owner::PreCommit => "pre-commit",
    }
}

/// Identify which known tool owns the pre-commit hook, or `None`.
///
/// Delegates to the genesis sigil table (`genesis::git_hooks::owner`):
/// `bd`, `lefthook`, `husky`, `prek`, `pre-commit` (the framework).
fn hook_owner(repo_root: &Path) -> Option<&'static str> {
    git_hooks::owner(repo_root, &HookName::PreCommit).map(owner_name)
}

/// Outcome for the prek branch of `check_git_hooks`.
fn prek_outcome(repo_root: &Path) -> (CheckStatus, String, Option<String>) {
    // A repo-local core.hooksPath means prek refuses to install — report this first.
    if let Ok(EffectiveHooksDir {
        path,
        scope: HooksDirScope::Local,
    }) = git_hooks::effective_hooks_dir(repo_root)
    {
        return (
            CheckStatus::Warn,
            format!(
                "prek.toml found but core.hooksPath is set ('{}') — prek cannot install",
                path.display()
            ),
            Some("Unset it first: git config --local --unset core.hooksPath".to_string()),
        );
    }
    // Check both pre-commit and pre-push for prek signature.
    if hook_contains(repo_root, "prek") {
        return (
            CheckStatus::Pass,
            "prek detected and installed".to_string(),
            None,
        );
    }
    // Another tool owns the hook — name it rather than suggest a re-install.
    if let Some(owner) = hook_owner(repo_root) {
        return (
            CheckStatus::Warn,
            format!(
                "prek.toml found but hook is owned by {} — chain prek or use {}'s runner",
                owner, owner
            ),
            None,
        );
    }
    (
        CheckStatus::Warn,
        "prek.toml found but hooks not installed — run: prek install".to_string(),
        None,
    )
}

/// Outcome for the lefthook branch of `check_git_hooks`.
fn lefthook_outcome(repo_root: &Path) -> (CheckStatus, String, Option<String>) {
    // Check the delegated core.hooksPath first (e.g., when beads/bd sets hooksPath)
    if let Ok(EffectiveHooksDir {
        path: delegated,
        scope: HooksDirScope::Local,
    }) = git_hooks::effective_hooks_dir(repo_root)
    {
        let lefthook_in_delegated = ["pre-commit", "pre-push"].iter().any(|hook| {
            std::fs::read_to_string(delegated.join(hook))
                .ok()
                .is_some_and(|c| c.contains("lefthook"))
        });

        if lefthook_in_delegated {
            return (
                CheckStatus::Pass,
                "lefthook detected and installed".to_string(),
                None,
            );
        }
        return (
            CheckStatus::Warn,
            format!(
                "lefthook.yml found but core.hooksPath is set ('{}') — lefthook not found in delegated path",
                delegated.display()
            ),
            Some("Run: lefthook install".to_string()),
        );
    }
    if hook_contains(repo_root, "lefthook") {
        return (
            CheckStatus::Pass,
            "lefthook detected and installed".to_string(),
            None,
        );
    }
    (
        CheckStatus::Warn,
        "lefthook.yml found but hooks not installed — run: lefthook install".to_string(),
        None,
    )
}

/// Outcome for the husky branch of `check_git_hooks`.
fn husky_outcome(repo_root: &Path) -> (CheckStatus, String, Option<String>) {
    if hook_exists_nonempty(repo_root) {
        (
            CheckStatus::Pass,
            "husky detected and installed".to_string(),
            None,
        )
    } else {
        (
            CheckStatus::Warn,
            ".husky/ found but hooks not installed — run: npx husky install".to_string(),
            None,
        )
    }
}

/// Outcome for the pre-commit branch of `check_git_hooks`.
fn precommit_outcome(repo_root: &Path) -> (CheckStatus, String, Option<String>) {
    const PREK_HINT: &str =
        "Consider prek for simpler hook management — https://github.com/chshersh/prek";
    if hook_contains(repo_root, "pre-commit") {
        (
            CheckStatus::Pass,
            "pre-commit detected and installed".to_string(),
            Some(PREK_HINT.to_string()),
        )
    } else {
        (
            CheckStatus::Warn,
            ".pre-commit-config.yaml found but hooks not installed — run: pre-commit install"
                .to_string(),
            Some(PREK_HINT.to_string()),
        )
    }
}

pub(super) fn check_git_hooks(repo_root: &Path) -> WayCheckEntry {
    let name = "Pre-commit quality gates";
    let intent = Some(
        "Prevent low-quality commits by running automated checks before code is saved to history."
            .to_string(),
    );
    let success_criteria = Some(
        "Automated checks (linters, tests) run automatically before code is committed.".to_string(),
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

    let outcome = if repo_root.join("prek.toml").exists() {
        prek_outcome(repo_root)
    } else if repo_root.join("lefthook.yml").exists() || repo_root.join("lefthook.yaml").exists() {
        lefthook_outcome(repo_root)
    } else if repo_root.join(".husky").is_dir() {
        husky_outcome(repo_root)
    } else if repo_root.join(".pre-commit-config.yaml").exists() {
        precommit_outcome(repo_root)
    } else {
        (
            CheckStatus::Warn,
            "No git hook manager detected".to_string(),
            Some("Add prek to manage git hooks — https://github.com/chshersh/prek".to_string()),
        )
    };

    let (status, message, suggestion) = outcome;
    mk(status, message, suggestion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_git_repo() -> tempfile::TempDir {
        // Hermetic git config: the ambient global/system config may set
        // core.hooksPath (e.g., a machine-wide hook shim), which
        // `genesis::git_hooks` correctly honors and would make hook
        // detection miss the test repo's `.git/hooks`. serial_test keeps
        // process-global env change off other tests.
        // SAFETY: test setup; serialized via #[serial_test::serial] below.
        unsafe {
            std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
            std::env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
        }
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
    #[serial_test::serial]
    fn prek_toml_with_core_hooks_path_set_reports_conflict() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        std::process::Command::new("git")
            .args(["config", "core.hooksPath", ".git/hooks"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
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
    #[serial_test::serial]
    fn prek_toml_with_bd_shim_reports_bd_as_owner() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# bd-shim v2\nexec bd hooks run pre-commit \"$@\"\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(
            result.message.contains("bd"),
            "expected bd as owner, got: {}",
            result.message
        );
    }

    #[test]
    #[serial_test::serial]
    fn prek_toml_with_lefthook_shim_reports_lefthook_as_owner() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\nexec lefthook run pre-commit \"$@\"\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(
            result.message.contains("lefthook"),
            "expected lefthook as owner, got: {}",
            result.message
        );
    }

    // -- prek pass cases --

    #[test]
    #[serial_test::serial]
    fn prek_toml_with_prek_in_precommit_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# managed by prek\nexec prek run pre-commit\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert_eq!(result.message, "prek detected and installed");
    }

    #[test]
    #[serial_test::serial]
    fn prek_toml_with_prek_in_prepush_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-push",
            "#!/usr/bin/env sh\n# managed by prek\nexec prek run pre-push\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert_eq!(result.message, "prek detected and installed");
    }

    #[test]
    #[serial_test::serial]
    fn bd_hook_that_chains_prek_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# bd-shim v2\n# chains prek\nexec prek run pre-commit\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    // -- lefthook via delegated core.hooksPath --

    #[test]
    #[serial_test::serial]
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
            CheckStatus::Pass,
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
    #[serial_test::serial]
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
            CheckStatus::Warn,
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
    #[serial_test::serial]
    fn lefthook_yml_without_any_hooks_still_suggests_install() {
        let dir = setup_git_repo();
        // Write lefthook.yml
        fs::write(dir.path().join("lefthook.yml"), "pre-commit: [echo test]\n").unwrap();

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(
            result.message.contains("lefthook install"),
            "expected lefthook install suggestion, got: {}",
            result.message
        );
    }

    // -- no hooks at all --

    #[test]
    #[serial_test::serial]
    fn prek_toml_with_no_hooks_suggests_install() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(
            result.message.contains("prek install"),
            "expected prek install suggestion, got: {}",
            result.message
        );
    }

    // -- no prek.toml --

    #[test]
    #[serial_test::serial]
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
