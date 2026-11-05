#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn doctor_invalid_state_file_fails() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "broken");

    fs::write(
        tmp.path().join(".wai/projects/broken/.state"),
        "{{not valid yaml at all",
    )
    .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
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

    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("No project initialized"));
}

#[test]
fn doctor_fix_repairs_missing_directories_without_prompting() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());

    // Remove a directory
    fs::remove_dir_all(tmp.path().join(".wai/archives"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    // Run fix with --yes to skip confirmation - should not prompt and should succeed
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Fixed"));

    // Verify the directory was recreated
    assert!(tmp.path().join(".wai/archives").is_dir());
}

#[test]
fn doctor_fix_no_fixable_issues() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());

    // Healthy workspace - no issues to fix
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("No fixable issues found"));
}

#[test]
fn doctor_fix_repairs_agents_md_block() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());

    // Create AGENTS.md without managed block
    fs::write(
        tmp.path().join("AGENTS.md"),
        "# Agent Instructions\n\nSome custom content here.\n",
    )
    .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    // Run fix
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Fixed"));

    // Verify the managed block was added
    let content = fs::read_to_string(tmp.path().join("AGENTS.md"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    assert!(content.contains("<!-- WAI:START -->"));
    assert!(content.contains("<!-- WAI:END -->"));
}

#[test]
fn doctor_fix_skips_corrupted_state() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "broken");

    // Corrupt the .state file
    fs::write(
        tmp.path().join(".wai/projects/broken/.state"),
        "{{not valid yaml at all",
    )
    .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    // Run fix - should not fix corrupted state files (no fix_fn for them)
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    assert!(out.status.success());

    // Verify the corrupted state file is still there (not fixed)
    let content = fs::read_to_string(tmp.path().join(".wai/projects/broken/.state"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    assert_eq!(content, "{{not valid yaml at all");
}

#[test]
fn doctor_fix_blocked_by_safe_mode() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());

    // Create a fixable issue
    fs::remove_dir_all(tmp.path().join(".wai/archives"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    // Run with --safe --fix - should refuse
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--safe"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("apply doctor fixes") || stderr.contains("--safe")));
}

// ─── progressive disclosure help ────────────────────────────────────────────

const HELP_CASES: &[(&[&str], &[&str], &[&str])] = &[
    (
        &["--help"],
        &["QUICK START:", "COMMANDS:", "Use -v for advanced options"],
        &["ADVANCED OPTIONS:"],
    ),
    (&["--help"], &[], &["ADVANCED OPTIONS:"]),
    (
        &["--help", "-v"],
        &["ADVANCED OPTIONS:", "--json", "--safe"],
        &[],
    ),
    (
        &["--help", "-vv"],
        &["ENVIRONMENT:", "NO_COLOR", "WAI_LOG"],
        &[],
    ),
    (
        &["--help", "-vvv"],
        &["INTERNALS:", "config.toml", "PARA"],
        &[],
    ),
];

#[test]
fn help_levels_show_progressively_more_sections() {
    for (args, expected, forbidden) in HELP_CASES {
        let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

        let out = wai_cmd(tmp.path())
            .args(*args)
            .output()
            .unwrap_or_else(|e| panic!("command should run: {e}"));
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success());
        for needle in *expected {
            assert!(stdout.contains(needle), "missing {needle} for {args:?}");
        }
        for needle in *forbidden {
            assert!(!stdout.contains(needle), "unexpected {needle} for {args:?}");
        }
    }
}

#[test]
fn command_help_shows_examples_first() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let output = wai_cmd(tmp.path())
        .args(["status", "--help"])
        .output()
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);

    let examples_pos = stdout
        .find("EXAMPLES:")
        .unwrap_or_else(|| panic!("should have EXAMPLES"));
    assert!(
        !stdout[..examples_pos].contains("OPTIONS:"),
        "EXAMPLES should appear before OPTIONS"
    );
}

#[test]
fn command_help_verbose_shows_all_sections() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(tmp.path())
        .args(["search", "--help", "-vvv"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
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

    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("No project initialized"));

    let out = wai_cmd(tmp.path())
        .args(["show"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("No project initialized"));
}

// ─── First-Run Detection ────────────────────────────────────────────────────

#[test]
fn first_run_creates_default_user_config() {
    let tmp_config = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    let tmp_project = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    assert!(out.status.success());

    // Verify user config was created and seen_tutorial defaults to false
    // (not present or explicitly false)
    let config_content = fs::read_to_string(tmp_config.path().join("wai/config.toml"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    assert!(!config_content.contains("seen_tutorial = true"));
}

#[test]
fn user_config_persists_seen_tutorial_flag() {
    use std::io::Write;

    let tmp_config = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    let tmp_project = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    // Pre-create the user config with seen_tutorial = true
    let wai_config_dir = tmp_config.path().join("wai");
    fs::create_dir_all(&wai_config_dir).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    let mut config_file = fs::File::create(wai_config_dir.join("config.toml"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    writeln!(config_file, "seen_tutorial = true").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    drop(config_file);

    // Run wai - it should read the existing config
    let out = wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    assert!(out.status.success());

    // Verify the flag is still true
    let config_content = fs::read_to_string(wai_config_dir.join("config.toml"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    assert!(config_content.contains("seen_tutorial = true"));
}

// ─── wai way ────────────────────────────────────────────────────────────────

#[test]
fn way_works_without_wai_init() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    // Don't init workspace - way should still work
    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Repo Hygiene & Agent Workflow Conventions"));
}

#[test]
fn way_always_exits_zero_across_adoption_levels() {
    fn seed_empty(_dir: &std::path::Path) {}
    fn seed_partial(dir: &std::path::Path) {
        fs::write(dir.join("README.md"), "# Test").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    }
    fn seed_full(dir: &std::path::Path) {
        seed_repo_with_all_way_checks(dir);
    }
    for seed in [seed_empty, seed_partial, seed_full] {
        let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
        seed(tmp.path());

        let output = wai_cmd(tmp.path())
            .args(["way"])
            .output()
            .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

        assert_eq!(
            output
                .status
                .code()
                .unwrap_or_else(|| panic!("unwrap failed")),
            0
        );
    }
}

/// Populate `dir` with every artifact the `wai way` checks look for.
fn seed_repo_with_all_way_checks(dir: &std::path::Path) {
    fs::write(dir.join("justfile"), "test:\n\techo test")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join("prek.toml"), "[hooks]").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::create_dir_all(dir.join(".git/hooks")).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join(".git/hooks/pre-commit"), "#!/bin/sh\nprek run")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join(".editorconfig"), "root = true")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join("README.md"), "# Test").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join("LICENSE"), "MIT").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join("CONTRIBUTING.md"), "# Contributing")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join(".gitignore"), "*.tmp").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join("CLAUDE.md"), "# Instructions")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::create_dir_all(dir.join(".github/workflows"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(dir.join(".github/workflows/ci.yml"), "name: CI")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::create_dir_all(dir.join(".devcontainer")).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
}

#[test]
fn way_json_output_is_valid_and_complete() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.path().join("README.md"), "# Test")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"checks\""));
    assert!(stdout.contains("\"summary\""));
    assert!(stdout.contains("\"pass\""));
    assert!(stdout.contains("\"recommendations\""));
    assert!(stdout.contains("\"name\""));
    assert!(stdout.contains("\"status\""));
    assert!(stdout.contains("\"message\""));
}

#[test]
fn way_minimal_repository_shows_info_and_suggestions() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    // Only README.md
    fs::write(tmp.path().join("README.md"), "# Test Project")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stdout.contains("ℹ"));
    assert!(stdout.contains("→"));
    assert!(stdout.contains("https://"));
    assert!(stderr.contains("best practices adopted"));
}

#[test]
fn way_complete_repository_all_pass() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    seed_complete_repository(tmp.path());

    let output = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have many/all Pass statuses
    assert!(stdout.contains("✓"));
    // Summary should show high adoption
    assert!(stdout.contains("configured"));
}

#[test]
fn way_complete_repository_minimal_suggestions() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    seed_complete_repository(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("✓"));
}

// ─── wai way: unit tests for individual checks ─────────────────────────────

/// Case table for individual `way` check scenarios.
/// Each case: setup (file to write, or none), check label, message, status.
struct WayCheckCase {
    file: Option<(&'static str, &'static str)>,
    install_hook: bool,
    hook_script: Option<&'static str>,
    label: &'static str,
    message: &'static str,
    status: &'static str,
}

const WAY_CHECK_CASES: &[WayCheckCase] = &[
    // ── task runner ──
    WayCheckCase {
        file: Some(("justfile", "test:\n\techo test")),
        install_hook: false,
        hook_script: None,
        label: "Command standardization",
        message: "justfile detected",
        status: "\"pass\"",
    },
    WayCheckCase {
        file: Some(("Makefile", "test:\n\techo test")),
        install_hook: false,
        hook_script: None,
        label: "Command standardization",
        message: "Makefile detected",
        status: "\"pass\"",
    },
    WayCheckCase {
        file: None,
        install_hook: false,
        hook_script: None,
        label: "Command standardization",
        message: "No task runner detected",
        status: "\"warn\"",
    },
    // ── git hooks ──
    WayCheckCase {
        file: Some(("prek.toml", "[hooks]")),
        install_hook: true,
        hook_script: Some("#!/bin/sh\nprek run"),
        label: "Pre-commit quality gates",
        message: "prek detected and installed",
        status: "\"pass\"",
    },
    WayCheckCase {
        file: Some(("prek.toml", "[hooks]")),
        install_hook: false,
        hook_script: None,
        label: "Pre-commit quality gates",
        message: "prek.toml found but hooks not installed",
        status: "\"warn\"",
    },
    WayCheckCase {
        file: Some((".pre-commit-config.yaml", "repos: []")),
        install_hook: true,
        hook_script: Some("#!/bin/sh\npre-commit run"),
        label: "Pre-commit quality gates",
        message: "pre-commit detected and installed",
        status: "\"pass\"",
    },
    WayCheckCase {
        file: Some((".pre-commit-config.yaml", "repos: []")),
        install_hook: false,
        hook_script: None,
        label: "Pre-commit quality gates",
        message: ".pre-commit-config.yaml found but hooks not installed",
        status: "\"warn\"",
    },
    WayCheckCase {
        file: Some(("lefthook.yml", "pre-commit:\n  commands: {}")),
        install_hook: true,
        hook_script: Some("#!/bin/sh\nlefthook run pre-commit"),
        label: "Pre-commit quality gates",
        message: "lefthook detected and installed",
        status: "\"pass\"",
    },
    WayCheckCase {
        file: Some(("lefthook.yml", "pre-commit:\n  commands: {}")),
        install_hook: false,
        hook_script: None,
        label: "Pre-commit quality gates",
        message: "lefthook.yml found but hooks not installed",
        status: "\"warn\"",
    },
    WayCheckCase {
        file: None,
        install_hook: false,
        hook_script: None,
        label: "Pre-commit quality gates",
        message: "No git hook manager detected",
        status: "\"warn\"",
    },
    // ── editorconfig ──
    WayCheckCase {
        file: Some((".editorconfig", "root = true")),
        install_hook: false,
        hook_script: None,
        label: "Consistent formatting",
        message: ".editorconfig detected",
        status: "\"pass\"",
    },
    WayCheckCase {
        file: None,
        install_hook: false,
        hook_script: None,
        label: "Consistent formatting",
        message: "No .editorconfig detected",
        status: "\"warn\"",
    },
];

#[test]
fn way_check_scenarios_report_expected_labels_and_status() {
    for case in WAY_CHECK_CASES {
        let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
        if let Some((name, contents)) = case.file {
            fs::write(tmp.path().join(name), contents)
                .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
        }
        if case.install_hook {
            fs::create_dir_all(tmp.path().join(".git/hooks"))
                .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
            fs::write(
                tmp.path().join(".git/hooks/pre-commit"),
                case.hook_script.unwrap(),
            )
            .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
        }

        let out = wai_cmd(tmp.path())
            .args(["way", "--json"])
            .output()
            .unwrap_or_else(|e| panic!("command should run: {e}"));
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success());
        assert!(stdout.contains(case.label), "missing {}", case.label);
        assert!(stdout.contains(case.message), "missing {}", case.message);
        assert!(stdout.contains(case.status), "missing {}", case.status);
    }
}

/// Write every best-practice file the `way` checks look for.
fn seed_complete_repository(tmp: &std::path::Path) {
    fs::write(tmp.join("justfile"), "test:\n\techo test")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join("prek.toml"), "[hooks]").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::create_dir_all(tmp.join(".git/hooks")).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join(".git/hooks/pre-commit"), "#!/bin/sh\nprek run")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join(".editorconfig"), "root = true")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join("README.md"), "# Test").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join("LICENSE"), "MIT").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join("CONTRIBUTING.md"), "# Contributing")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join(".gitignore"), "*.tmp").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join("CLAUDE.md"), "# Instructions")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::create_dir_all(tmp.join(".github/workflows"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(tmp.join(".github/workflows/ci.yml"), "name: CI")
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::create_dir_all(tmp.join(".devcontainer")).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
}
