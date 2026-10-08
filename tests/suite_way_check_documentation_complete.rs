#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn way_check_documentation_complete() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join("LICENSE"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();
    fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::create_dir(tmp.path().join("docs")).unwrap();
    fs::write(tmp.path().join("docs").join("index.md"), "# Docs").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project documentation"));
    assert!(stdout.contains("Complete"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_documentation_not_configured() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project documentation"));
    assert!(stdout.contains("Missing critical files"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_documentation_missing_critical() {
    let tmp = TempDir::new().unwrap();
    // Only LICENSE and CONTRIBUTING (missing critical README and .gitignore)
    fs::write(tmp.path().join("LICENSE"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project documentation"));
    assert!(stdout.contains("Missing critical files"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn init_creates_para_structure() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Verify PARA directories
    assert!(tmp.path().join(".wai/projects").is_dir());
    assert!(tmp.path().join(".wai/areas").is_dir());
    assert!(tmp.path().join(".wai/resources").is_dir());
    assert!(tmp.path().join(".wai/archives").is_dir());
    assert!(tmp.path().join(".wai/plugins").is_dir());

    // Verify agent-config structure
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills")
            .is_dir()
    );
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/rules")
            .is_dir()
    );
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/context")
            .is_dir()
    );
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/.projections.yml")
            .is_file()
    );

    // Verify config.toml exists and contains project name
    let config = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert!(config.contains("test-ws"));
}

#[test]
fn init_warns_if_already_initialized() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Second init outputs warning to stdout (plain println, not cliclack log)
    let out = wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("already initialized"));
}

#[test]
fn init_json_fresh_includes_wai_way_suggestion() {
    let tmp = TempDir::new().unwrap();
    let out = wai_cmd(tmp.path())
        .args(["--json", "init", "--name", "test-ws", "--yes"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let stdout = String::from_utf8_lossy(&out);
    let payload: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(
        payload["data"]["already_initialized"], false,
        "fresh init should report already_initialized: false"
    );
    assert_eq!(
        payload["data"]["project_name"], "test-ws",
        "JSON should include the workspace name"
    );
    let suggestions = payload["data"]["suggestions"].as_array().unwrap();
    let commands: Vec<&str> = suggestions
        .iter()
        .map(|s| s["command"].as_str().unwrap())
        .collect();
    assert!(
        commands.contains(&"wai way"),
        "wai way not in suggestions: {:?}",
        commands
    );
}

#[test]
fn init_json_reinit_includes_wai_way_suggestion() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let out = wai_cmd(tmp.path())
        .args(["--json", "init", "--name", "test-ws", "--yes"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let stdout = String::from_utf8_lossy(&out);
    let payload: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(
        payload["data"]["already_initialized"], true,
        "reinit should report already_initialized: true"
    );
    let suggestions = payload["data"]["suggestions"].as_array().unwrap();
    let commands: Vec<&str> = suggestions
        .iter()
        .map(|s| s["command"].as_str().unwrap())
        .collect();
    assert!(
        commands.contains(&"wai way"),
        "wai way not in suggestions: {:?}",
        commands
    );
}

// ─── Robust Reinit (wai-exc) ─────────────────────────────────────────────────

#[test]
fn doctor_detects_version_mismatch_and_fix_repairs_it() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Write a config.toml with a stale commit to simulate version mismatch.
    // Doctor now uses tool_commit for version checking (takes priority over version string),
    // so we must stale the commit hash, not just the version.
    let config_path = tmp.path().join(".wai/config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let stale_config = config
        .replace(env!("CARGO_PKG_VERSION"), "0.0.0-stale")
        .replace(env!("WAI_GIT_COMMIT"), "deadbeef-stale");
    fs::write(&config_path, stale_config).unwrap();

    // Doctor should detect the mismatch (Warn status) — exits 0 with warnings
    let out = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(
        ((stdout.contains("deadbeef-stale") || stdout.contains("stale"))
            || stdout.contains("differs"))
    );

    // Fix should update the version
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Config should now have the current version
    let repaired = fs::read_to_string(&config_path).unwrap();
    assert!(
        repaired.contains(env!("CARGO_PKG_VERSION")),
        "config.toml should have current version after fix"
    );
}

#[test]
fn doctor_detects_missing_agent_config_subdirs_and_fix_creates_them() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Remove agent-config subdirectories
    fs::remove_dir_all(tmp.path().join(".wai/resources/agent-config/skills")).unwrap();
    fs::remove_dir_all(tmp.path().join(".wai/resources/agent-config/rules")).unwrap();

    // Doctor should detect them as missing — exits 1 on failures
    let out = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success());
    assert!((stdout.contains("skills") || stdout.contains("Missing")));

    // Fix should recreate the directories
    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Fixed"));

    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills")
            .is_dir(),
        "skills dir should be recreated"
    );
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/rules")
            .is_dir(),
        "rules dir should be recreated"
    );
}

#[test]
fn reinit_creates_missing_directories_and_updates_version() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Simulate damage: remove directories and stale version
    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();
    fs::remove_dir_all(tmp.path().join(".wai/resources/agent-config/context")).unwrap();

    let config_path = tmp.path().join(".wai/config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let stale_config = config.replace(env!("CARGO_PKG_VERSION"), "0.0.0-stale");
    fs::write(&config_path, stale_config).unwrap();

    // Re-init should repair without prompting (--name reuses existing workspace)
    let out = wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Missing directories should be restored
    assert!(
        tmp.path().join(".wai/archives").is_dir(),
        "archives dir should be restored by reinit"
    );
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/context")
            .is_dir(),
        "agent-config/context should be restored by reinit"
    );

    // Version should be updated
    let repaired = fs::read_to_string(&config_path).unwrap();
    assert!(
        repaired.contains(env!("CARGO_PKG_VERSION")),
        "config.toml should have current version after reinit"
    );
}

#[test]
fn reinit_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Capture state after first init
    let config_before = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();

    // Re-init twice
    let out = wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Config content should be stable (no duplicates, no corruption)
    let config_after = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert_eq!(
        config_before.trim(),
        config_after.trim(),
        "config.toml should be identical after idempotent reinit"
    );

    // All expected directories still exist
    assert!(tmp.path().join(".wai/projects").is_dir());
    assert!(tmp.path().join(".wai/areas").is_dir());
    assert!(tmp.path().join(".wai/resources").is_dir());
    assert!(tmp.path().join(".wai/archives").is_dir());
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills")
            .is_dir()
    );
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/rules")
            .is_dir()
    );
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/context")
            .is_dir()
    );
}

// ─── wai new project ────────────────────────────────────────────────────────

#[test]
fn new_project_creates_structure_with_state() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let proj = tmp.path().join(".wai/projects/my-app");
    assert!(proj.join("research").is_dir());
    assert!(proj.join("plans").is_dir());
    assert!(proj.join("designs").is_dir());
    assert!(proj.join("handoffs").is_dir());
    assert!(proj.join(".state").is_file());

    // Verify state file is valid YAML with research phase
    let state = fs::read_to_string(proj.join(".state")).unwrap();
    assert!(state.contains("current: research"));
}

#[test]
fn new_project_fails_if_exists() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["new", "project", "my-app"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("already exists"));
}

// ─── wai new area / resource ────────────────────────────────────────────────

#[test]
fn new_area_creates_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["new", "area", "dev-standards"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        tmp.path()
            .join(".wai/areas/dev-standards/research")
            .is_dir()
    );
    assert!(tmp.path().join(".wai/areas/dev-standards/plans").is_dir());
}

#[test]
fn new_resource_creates_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["new", "resource", "cheatsheets"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(tmp.path().join(".wai/resources/cheatsheets").is_dir());
}

// ─── wai phase ──────────────────────────────────────────────────────────────

#[test]
fn phase_show_displays_current_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["phase", "show"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("research"));
}

#[test]
fn phase_next_advances_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["phase", "next"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("design"));

    // Verify state persisted
    let state = fs::read_to_string(tmp.path().join(".wai/projects/my-app/.state")).unwrap();
    assert!(state.contains("current: design"));
}

#[test]
fn phase_back_goes_to_previous() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Advance to design first
    let out = wai_cmd(tmp.path())
        .args(["phase", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Go back to research (cliclack outputs to stderr)
    let out = wai_cmd(tmp.path())
        .args(["phase", "back"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("research"));
}

#[test]
fn phase_back_fails_at_research() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["phase", "back"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("already at first phase"));
}

#[test]
fn phase_set_jumps_to_target() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("implement"));

    let state = fs::read_to_string(tmp.path().join(".wai/projects/my-app/.state")).unwrap();
    assert!(state.contains("current: implement"));
}

#[test]
fn phase_set_rejects_invalid_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["phase", "set", "banana"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("Unknown phase"));
}

// ─── wai add research ───────────────────────────────────────────────────────

#[test]
fn add_research_creates_dated_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "Initial findings on the topic",
            "--project",
            "my-app",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let research_dir = tmp.path().join(".wai/projects/my-app/research");
    let files: Vec<_> = fs::read_dir(&research_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_str().unwrap_or("").ends_with(".md"))
        .collect();

    assert_eq!(files.len(), 1);
    let filename = files[0].file_name();
    let name = filename.to_str().unwrap();
    // Should start with YYYY-MM-DD
    assert!(name.len() >= 10);
    assert_eq!(&name[4..5], "-");
    assert_eq!(&name[7..8], "-");

    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.contains("Initial findings on the topic"));
}

#[test]
fn add_research_with_tags_includes_frontmatter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "Tagged research",
            "--project",
            "my-app",
            "--tags",
            "api,design",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let research_dir = tmp.path().join(".wai/projects/my-app/research");
    let files: Vec<_> = fs::read_dir(&research_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.contains("---"));
    assert!(content.contains("tags:"));
    assert!(content.contains("api"));
}

// ─── wai show ───────────────────────────────────────────────────────────────

#[test]
fn show_overview_lists_para_categories() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["show"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Projects"));
    assert!(stdout.contains("my-app"));
    assert!(stdout.contains("Areas"));
    assert!(stdout.contains("Resources"));
    assert!(stdout.contains("Archives"));
}

#[test]
fn show_specific_project_lists_contents() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // ANSI codes may split "Project: my-app", so check parts separately
    let out = wai_cmd(tmp.path())
        .args(["show", "my-app"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project:"));
    assert!(stdout.contains("my-app"));
    assert!(stdout.contains("research"));
    assert!(stdout.contains("plans"));
}

#[test]
fn show_nonexistent_item_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["show", "nonexistent"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}

// ─── wai move ───────────────────────────────────────────────────────────────

#[test]
fn move_project_to_archives() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "old-project");

    let out = wai_cmd(tmp.path())
        .args(["move", "old-project", "archives"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Moved"));

    assert!(!tmp.path().join(".wai/projects/old-project").exists());
    assert!(tmp.path().join(".wai/archives/old-project").exists());
}

#[test]
fn move_nonexistent_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["move", "ghost", "archives"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}

// ─── wai search ─────────────────────────────────────────────────────────────

#[test]
fn search_finds_content_in_artifacts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-api-notes.md",
        "# API Research\nThe REST API uses JSON for serialization.\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "JSON"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Search results"));
    assert!(stdout.contains("JSON"));
}

#[test]
fn search_case_insensitive() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "Hello World\n",
    );

    // "Hello" is highlighted with ANSI codes, so check for "World" separately
    let out = wai_cmd(tmp.path())
        .args(["search", "hello"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Search results"));
    assert!(stdout.contains("World"));
}

#[test]
fn search_rejects_unknown_type() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["search", "x", "--type", "blogposts"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("Valid --type values"));
    assert!(stderr.contains("handoff"));
}

#[test]
fn search_accepts_canonical_type_values() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "canonical_type_match here\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "canonical_type_match", "--type", "research"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("canonical_type_match"));
}

#[test]
fn search_accepts_plural_type_aliases() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "alias_type_match here\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "alias_type_match", "--type", "plans"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("alias_type_match"));
}

#[test]
fn search_json_honors_limit() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-a.md",
        "match_one match_two match_three\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "match_", "--json", "-n", "1"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let out_str = String::from_utf8(out).unwrap();
    let results: serde_json::Value = serde_json::from_str(&out_str).unwrap();
    assert_eq!(results["data"]["results"].as_array().unwrap().len(), 1);
}
