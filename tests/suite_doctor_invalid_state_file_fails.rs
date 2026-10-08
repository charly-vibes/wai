#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn doctor_invalid_state_file_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "broken");

    fs::write(
        tmp.path().join(".wai/projects/broken/.state"),
        "{{not valid yaml at all",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout.contains("\"status\": \"fail\""));
    assert!(stdout.contains("broken"));
}

#[test]
fn doctor_uninitialised_directory_errors() {
    let stale_wai = std::path::Path::new("/tmp/.wai");
    if stale_wai.exists() {
        let _ = fs::remove_dir_all(stale_wai);
    }

    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("No project initialized"));
}

#[test]
fn doctor_fix_repairs_missing_directories() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Remove a directory
    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    // Run fix with --yes to skip confirmation
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Fixed"));

    // Verify the directory was recreated
    assert!(tmp.path().join(".wai/archives").is_dir());
}

#[test]
fn doctor_fix_skips_confirmation_with_yes() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a fixable issue
    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    // Run with --yes - should not prompt and should succeed
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Fixed"));
}

#[test]
fn doctor_fix_no_fixable_issues() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Healthy workspace - no issues to fix
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("No fixable issues found"));
}

#[test]
fn doctor_fix_repairs_agents_md_block() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create AGENTS.md without managed block
    fs::write(
        tmp.path().join("AGENTS.md"),
        "# Agent Instructions\n\nSome custom content here.\n",
    )
    .unwrap();

    // Run fix
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Fixed"));

    // Verify the managed block was added
    let content = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("<!-- WAI:START -->"));
    assert!(content.contains("<!-- WAI:END -->"));
}

#[test]
fn doctor_fix_skips_corrupted_state() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "broken");

    // Corrupt the .state file
    fs::write(
        tmp.path().join(".wai/projects/broken/.state"),
        "{{not valid yaml at all",
    )
    .unwrap();

    // Run fix - should not fix corrupted state files (no fix_fn for them)
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Verify the corrupted state file is still there (not fixed)
    let content = fs::read_to_string(tmp.path().join(".wai/projects/broken/.state")).unwrap();
    assert_eq!(content, "{{not valid yaml at all");
}

#[test]
fn doctor_fix_blocked_by_safe_mode() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a fixable issue
    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    // Run with --safe --fix - should refuse
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--safe"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("apply doctor fixes") || stderr.contains("--safe")));
}

// ─── progressive disclosure help ────────────────────────────────────────────

#[test]
fn help_shows_quick_start_and_commands() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["--help"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("QUICK START:"));
    assert!(stdout.contains("COMMANDS:"));
    assert!(stdout.contains("Use -v for advanced options"));
}

#[test]
fn help_default_hides_advanced_options() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["--help"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("ADVANCED OPTIONS:")));
}

#[test]
fn help_v_shows_advanced_options() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["--help", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("ADVANCED OPTIONS:"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("--safe"));
}

#[test]
fn help_vv_shows_environment_variables() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["--help", "-vv"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("ENVIRONMENT:"));
    assert!(stdout.contains("NO_COLOR"));
    assert!(stdout.contains("WAI_LOG"));
}

#[test]
fn help_vvv_shows_internals() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["--help", "-vvv"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("INTERNALS:"));
    assert!(stdout.contains("config.toml"));
    assert!(stdout.contains("PARA"));
}

#[test]
fn command_help_shows_examples_first() {
    let tmp = TempDir::new().unwrap();

    let output = wai_cmd(tmp.path())
        .args(["status", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let examples_pos = stdout.find("EXAMPLES:").expect("should have EXAMPLES");
    assert!(
        !stdout[..examples_pos].contains("OPTIONS:"),
        "EXAMPLES should appear before OPTIONS"
    );
}

#[test]
fn command_help_verbose_shows_all_sections() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["search", "--help", "-vvv"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("EXAMPLES:"));
    assert!(stdout.contains("ADVANCED OPTIONS:"));
    assert!(stdout.contains("ENVIRONMENT:"));
    assert!(stdout.contains("INTERNALS:"));
}

// ─── error cases ────────────────────────────────────────────────────────────

#[test]
fn commands_fail_without_init() {
    // Clean up any stale .wai/ left in /tmp by previous test runs, which would
    // cause find_project_root to walk up and falsely detect an initialized workspace.
    let stale_wai = std::path::Path::new("/tmp/.wai");
    if stale_wai.exists() {
        let _ = fs::remove_dir_all(stale_wai);
    }

    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("No project initialized"));

    let out = wai_cmd(tmp.path())
        .args(["show"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("No project initialized"));
}

// ─── First-Run Detection ────────────────────────────────────────────────────

#[test]
fn first_run_detects_no_user_config() {
    // Create a temporary directory for user config
    let tmp_config = TempDir::new().unwrap();
    let tmp_project = TempDir::new().unwrap();

    // Set XDG_CONFIG_HOME to use our temp directory
    let out = wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Verify user config was created
    assert!(tmp_config.path().join("wai/config.toml").exists());
}

#[test]
fn first_run_creates_default_user_config() {
    let tmp_config = TempDir::new().unwrap();
    let tmp_project = TempDir::new().unwrap();

    let out = wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Read the created config
    let config_content = fs::read_to_string(tmp_config.path().join("wai/config.toml")).unwrap();
    // seen_tutorial should default to false (not present or explicitly false)
    assert!(!config_content.contains("seen_tutorial = true"));
}

#[test]
fn user_config_persists_seen_tutorial_flag() {
    use std::io::Write;

    let tmp_config = TempDir::new().unwrap();
    let tmp_project = TempDir::new().unwrap();

    // Pre-create the user config with seen_tutorial = true
    let wai_config_dir = tmp_config.path().join("wai");
    fs::create_dir_all(&wai_config_dir).unwrap();
    let mut config_file = fs::File::create(wai_config_dir.join("config.toml")).unwrap();
    writeln!(config_file, "seen_tutorial = true").unwrap();
    drop(config_file);

    // Run wai - it should read the existing config
    let out = wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Verify the flag is still true
    let config_content = fs::read_to_string(wai_config_dir.join("config.toml")).unwrap();
    assert!(config_content.contains("seen_tutorial = true"));
}

// ─── wai way ────────────────────────────────────────────────────────────────

#[test]
fn way_works_without_wai_init() {
    let tmp = TempDir::new().unwrap();

    // Don't init workspace - way should still work
    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Repo Hygiene & Agent Workflow Conventions"));
}

#[test]
fn way_always_exits_zero_empty_repo() {
    let tmp = TempDir::new().unwrap();

    // Empty repo with no files
    let output = wai_cmd(tmp.path()).args(["way"]).output().unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn way_always_exits_zero_partial_adoption() {
    let tmp = TempDir::new().unwrap();

    // Partial adoption - just README
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    let output = wai_cmd(tmp.path()).args(["way"]).output().unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn way_always_exits_zero_all_passing() {
    let tmp = TempDir::new().unwrap();
    seed_repo_with_all_way_checks(tmp.path());

    let output = wai_cmd(tmp.path()).args(["way"]).output().unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

/// Populate `dir` with every artifact the `wai way` checks look for.
fn seed_repo_with_all_way_checks(dir: &std::path::Path) {
    fs::write(dir.join("justfile"), "test:\n\techo test").unwrap();
    fs::write(dir.join("prek.toml"), "[hooks]").unwrap();
    fs::create_dir_all(dir.join(".git/hooks")).unwrap();
    fs::write(dir.join(".git/hooks/pre-commit"), "#!/bin/sh\nprek run").unwrap();
    fs::write(dir.join(".editorconfig"), "root = true").unwrap();
    fs::write(dir.join("README.md"), "# Test").unwrap();
    fs::write(dir.join("LICENSE"), "MIT").unwrap();
    fs::write(dir.join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(dir.join(".gitignore"), "*.tmp").unwrap();
    fs::write(dir.join("CLAUDE.md"), "# Instructions").unwrap();
    fs::create_dir_all(dir.join(".github/workflows")).unwrap();
    fs::write(dir.join(".github/workflows/ci.yml"), "name: CI").unwrap();
    fs::create_dir_all(dir.join(".devcontainer")).unwrap();
}

#[test]
fn way_json_output_valid() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"checks\""));
    assert!(stdout.contains("\"summary\""));
    assert!(stdout.contains("\"pass\""));
    assert!(stdout.contains("\"recommendations\""));
}

#[test]
fn way_json_includes_check_fields() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"name\""));
    assert!(stdout.contains("\"status\""));
    assert!(stdout.contains("\"message\""));
}

#[test]
fn way_json_exits_zero() {
    let tmp = TempDir::new().unwrap();

    let output = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn way_minimal_repository_shows_info() {
    let tmp = TempDir::new().unwrap();

    // Only README.md
    fs::write(tmp.path().join("README.md"), "# Test Project").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stdout.contains("ℹ"));
    assert!(stderr.contains("best practices adopted"));
}

#[test]
fn way_minimal_repository_has_suggestions() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("→"));
    assert!(stdout.contains("https://"));
}

#[test]
fn way_complete_repository_all_pass() {
    let tmp = TempDir::new().unwrap();

    seed_complete_repository(tmp.path());

    let output = wai_cmd(tmp.path()).args(["way"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have many/all Pass statuses
    assert!(stdout.contains("✓"));
    // Summary should show high adoption
    assert!(stdout.contains("configured"));
}

#[test]
fn way_complete_repository_minimal_suggestions() {
    let tmp = TempDir::new().unwrap();

    seed_complete_repository(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("✓"));
}

// ─── wai way: unit tests for individual checks ─────────────────────────────

#[test]
fn way_check_task_runner_justfile() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("justfile"), "test:\n\techo test").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Command standardization"));
    assert!(stdout.contains("justfile detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_task_runner_makefile() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("Makefile"), "test:\n\techo test").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Command standardization"));
    assert!(stdout.contains("Makefile detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_task_runner_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Command standardization"));
    assert!(stdout.contains("No task runner detected"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_git_hooks_prek() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("prek.toml"), "[hooks]").unwrap();
    fs::create_dir_all(tmp.path().join(".git/hooks")).unwrap();
    fs::write(
        tmp.path().join(".git/hooks/pre-commit"),
        "#!/bin/sh\nprek run",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pre-commit quality gates"));
    assert!(stdout.contains("prek detected and installed"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_git_hooks_prek_not_installed() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("prek.toml"), "[hooks]").unwrap();
    // No .git/hooks/pre-commit — hooks not installed

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pre-commit quality gates"));
    assert!(stdout.contains("prek.toml found but hooks not installed"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_git_hooks_precommit() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".pre-commit-config.yaml"), "repos: []").unwrap();
    fs::create_dir_all(tmp.path().join(".git/hooks")).unwrap();
    fs::write(
        tmp.path().join(".git/hooks/pre-commit"),
        "#!/bin/sh\npre-commit run",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pre-commit quality gates"));
    assert!(stdout.contains("pre-commit detected and installed"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_git_hooks_precommit_not_installed() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".pre-commit-config.yaml"), "repos: []").unwrap();
    // No .git/hooks/pre-commit

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pre-commit quality gates"));
    assert!(stdout.contains(".pre-commit-config.yaml found but hooks not installed"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_git_hooks_lefthook_installed() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("lefthook.yml"),
        "pre-commit:\n  commands: {}",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join(".git/hooks")).unwrap();
    fs::write(
        tmp.path().join(".git/hooks/pre-commit"),
        "#!/bin/sh\nlefthook run pre-commit",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pre-commit quality gates"));
    assert!(stdout.contains("lefthook detected and installed"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_git_hooks_lefthook_not_installed() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("lefthook.yml"),
        "pre-commit:\n  commands: {}",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pre-commit quality gates"));
    assert!(stdout.contains("lefthook.yml found but hooks not installed"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_git_hooks_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pre-commit quality gates"));
    assert!(stdout.contains("No git hook manager detected"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_editorconfig_present() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".editorconfig"), "root = true").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Consistent formatting"));
    assert!(stdout.contains(".editorconfig detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_editorconfig_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Consistent formatting"));
    assert!(stdout.contains("No .editorconfig detected"));
    assert!(stdout.contains("\"warn\""));
}

/// Write every best-practice file the `way` checks look for.
fn seed_complete_repository(tmp: &std::path::Path) {
    fs::write(tmp.join("justfile"), "test:\n\techo test").unwrap();
    fs::write(tmp.join("prek.toml"), "[hooks]").unwrap();
    fs::create_dir_all(tmp.join(".git/hooks")).unwrap();
    fs::write(tmp.join(".git/hooks/pre-commit"), "#!/bin/sh\nprek run").unwrap();
    fs::write(tmp.join(".editorconfig"), "root = true").unwrap();
    fs::write(tmp.join("README.md"), "# Test").unwrap();
    fs::write(tmp.join("LICENSE"), "MIT").unwrap();
    fs::write(tmp.join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(tmp.join(".gitignore"), "*.tmp").unwrap();
    fs::write(tmp.join("CLAUDE.md"), "# Instructions").unwrap();
    fs::create_dir_all(tmp.join(".github/workflows")).unwrap();
    fs::write(tmp.join(".github/workflows/ci.yml"), "name: CI").unwrap();
    fs::create_dir_all(tmp.join(".devcontainer")).unwrap();
}
