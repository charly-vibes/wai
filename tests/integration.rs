use assert_cmd::Command;
use chrono::Local;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Strip ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Skip escape sequence: ESC [ ... final-byte
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Helper: create a wai command that runs in the given directory.
#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    // Disable color output for predictable assertions
    cmd.env("NO_COLOR", "1");
    cmd
}

/// Helper: initialize a wai workspace non-interactively.
fn init_workspace(dir: &std::path::Path) {
    wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();
}

/// Helper: create a project inside an initialized workspace.
fn create_project(dir: &std::path::Path, name: &str) {
    wai_cmd(dir)
        .args(["new", "project", name])
        .assert()
        .success();
}

/// Helper: write a dated artifact file directly into a project subdirectory.
fn write_artifact(
    dir: &std::path::Path,
    project: &str,
    subdir: &str,
    filename: &str,
    content: &str,
) {
    let path = dir
        .join(".wai")
        .join("projects")
        .join(project)
        .join(subdir)
        .join(filename);
    fs::write(path, content).unwrap();
}

// ─── wai init ────────────────────────────────────────────────────────────────

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
    wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already initialized"));
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
    wai_cmd(tmp.path())
        .args(["doctor"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("deadbeef-stale")
                .or(predicate::str::contains("stale"))
                .or(predicate::str::contains("differs")),
        );

    // Fix should update the version
    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .assert()
        .success();

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
    wai_cmd(tmp.path())
        .args(["doctor"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("skills").or(predicate::str::contains("Missing")));

    // Fix should recreate the directories
    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Fixed"));

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
    wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();

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
    wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();

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

    wai_cmd(tmp.path())
        .args(["new", "project", "my-app"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

// ─── wai new area / resource ────────────────────────────────────────────────

#[test]
fn new_area_creates_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["new", "area", "dev-standards"])
        .assert()
        .success();

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

    wai_cmd(tmp.path())
        .args(["new", "resource", "cheatsheets"])
        .assert()
        .success();

    assert!(tmp.path().join(".wai/resources/cheatsheets").is_dir());
}

// ─── wai phase ──────────────────────────────────────────────────────────────

#[test]
fn phase_show_displays_current_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["phase", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("research"));
}

#[test]
fn phase_next_advances_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["phase", "next"])
        .assert()
        .success()
        .stderr(predicate::str::contains("design"));

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
    wai_cmd(tmp.path())
        .args(["phase", "next"])
        .assert()
        .success();

    // Go back to research (cliclack outputs to stderr)
    wai_cmd(tmp.path())
        .args(["phase", "back"])
        .assert()
        .success()
        .stderr(predicate::str::contains("research"));
}

#[test]
fn phase_back_fails_at_research() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["phase", "back"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already at first phase"));
}

#[test]
fn phase_set_jumps_to_target() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .assert()
        .success()
        .stderr(predicate::str::contains("implement"));

    let state = fs::read_to_string(tmp.path().join(".wai/projects/my-app/.state")).unwrap();
    assert!(state.contains("current: implement"));
}

#[test]
fn phase_set_rejects_invalid_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["phase", "set", "banana"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown phase"));
}

// ─── wai add research ───────────────────────────────────────────────────────

#[test]
fn add_research_creates_dated_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "Initial findings on the topic",
            "--project",
            "my-app",
        ])
        .assert()
        .success();

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

    wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "Tagged research",
            "--project",
            "my-app",
            "--tags",
            "api,design",
        ])
        .assert()
        .success();

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

    wai_cmd(tmp.path())
        .args(["show"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Projects")
                .and(predicate::str::contains("my-app"))
                .and(predicate::str::contains("Areas"))
                .and(predicate::str::contains("Resources"))
                .and(predicate::str::contains("Archives")),
        );
}

#[test]
fn show_specific_project_lists_contents() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // ANSI codes may split "Project: my-app", so check parts separately
    wai_cmd(tmp.path())
        .args(["show", "my-app"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Project:")
                .and(predicate::str::contains("my-app"))
                .and(predicate::str::contains("research"))
                .and(predicate::str::contains("plans")),
        );
}

#[test]
fn show_nonexistent_item_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["show", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// ─── wai move ───────────────────────────────────────────────────────────────

#[test]
fn move_project_to_archives() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "old-project");

    wai_cmd(tmp.path())
        .args(["move", "old-project", "archives"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Moved"));

    assert!(!tmp.path().join(".wai/projects/old-project").exists());
    assert!(tmp.path().join(".wai/archives/old-project").exists());
}

#[test]
fn move_nonexistent_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["move", "ghost", "archives"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
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

    wai_cmd(tmp.path())
        .args(["search", "JSON"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search results").and(predicate::str::contains("JSON")));
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
    wai_cmd(tmp.path())
        .args(["search", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search results").and(predicate::str::contains("World")));
}

#[test]
fn search_no_results() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["search", "zzz_nonexistent_zzz"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results found"));
}

#[test]
fn search_json_outputs_results() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-api-notes.md",
        "JSON output is required.\n",
    );

    wai_cmd(tmp.path())
        .args(["search", "JSON", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"results\""));
}

#[test]
fn search_with_type_filter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "keyword_match here\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "plans",
        "2026-01-15-plan.md",
        "keyword_match in plan\n",
    );

    // Filter to research only
    wai_cmd(tmp.path())
        .args(["search", "keyword_match", "--type", "research"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("research/")
                .and(predicate::str::contains("keyword_match"))
                // Should NOT contain the plans match
                .and(predicate::str::contains("plans/").not()),
        );
}

#[test]
fn search_with_project_filter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "app-a");
    create_project(tmp.path(), "app-b");
    write_artifact(
        tmp.path(),
        "app-a",
        "research",
        "2026-01-15-a.md",
        "unique_a\n",
    );
    write_artifact(
        tmp.path(),
        "app-b",
        "research",
        "2026-01-15-b.md",
        "unique_b\n",
    );

    wai_cmd(tmp.path())
        .args(["search", "unique_a", "--in", "app-a"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_a"));

    wai_cmd(tmp.path())
        .args(["search", "unique_b", "--in", "app-a"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results found"));
}

#[test]
fn search_with_regex() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "error_code: 404\nerror_code: 500\nsuccess_code: 200\n",
    );

    wai_cmd(tmp.path())
        .args(["search", "error_code: \\d+", "--regex"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("2 matches")
                .and(predicate::str::contains("404"))
                .and(predicate::str::contains("500")),
        );
}

#[test]
fn search_with_limit() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "line one match\nline two match\nline three match\nline four match\n",
    );

    wai_cmd(tmp.path())
        .args(["search", "match", "-n", "2"])
        .assert()
        .success()
        // header shows total count; truncation notice goes to stderr
        .stdout(predicate::str::contains("4 matches"))
        .stderr(predicate::str::contains("Showing first 2 of 4"));
}

#[test]
fn search_invalid_regex_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["search", "[invalid", "--regex"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid regex"));
}

// ─── add plan/design with --tags ─────────────────────────────────────────────

#[test]
fn add_plan_with_tags_includes_frontmatter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args([
            "add",
            "plan",
            "My plan content",
            "--project",
            "my-app",
            "--tags",
            "backend,api",
        ])
        .assert()
        .success();

    let plans_dir = tmp.path().join(".wai/projects/my-app/plans");
    let files: Vec<_> = fs::read_dir(&plans_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.contains("---"), "plan should have frontmatter");
    assert!(content.contains("tags:"), "plan should have tags key");
    assert!(content.contains("backend"), "plan should have backend tag");
    assert!(content.contains("api"), "plan should have api tag");
    assert!(
        content.contains("My plan content"),
        "plan body should be present"
    );
}

#[test]
fn add_design_with_tags_includes_frontmatter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args([
            "add",
            "design",
            "My design content",
            "--project",
            "my-app",
            "--tags",
            "ux",
        ])
        .assert()
        .success();

    let designs_dir = tmp.path().join(".wai/projects/my-app/designs");
    let files: Vec<_> = fs::read_dir(&designs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.contains("---"), "design should have frontmatter");
    assert!(content.contains("ux"), "design should have ux tag");
}

// ─── search --tag and --latest ────────────────────────────────────────────────

#[test]
fn search_tag_filter_returns_only_tagged_files() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Write one tagged and one untagged research file.
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-tagged.md",
        "---\ntags: [rust, perf]\n---\n\nfoo bar baz",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-11-untagged.md",
        "foo bar baz",
    );

    let output = wai_cmd(tmp.path())
        .args(["search", "foo", "--tag", "rust"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("tagged"), "should match the tagged file");
    assert!(
        !stdout.contains("untagged"),
        "should not match the untagged file"
    );
}

#[test]
fn search_tag_filter_no_match_returns_no_results() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-file.md",
        "---\ntags: [rust]\n---\n\ncontent",
    );

    let output = wai_cmd(tmp.path())
        .args(["search", "content", "--tag", "nonexistent-tag"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No results"), "should report no results");
}

#[test]
fn search_malformed_frontmatter_does_not_abort_tag_search() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-bad.md",
        "---\n[[ invalid\n---\ncontent",
    );

    // Should succeed (not panic / not fail), even with malformed frontmatter.
    wai_cmd(tmp.path())
        .args(["search", "content", "--tag", "anything"])
        .assert()
        .success();
}

#[test]
fn search_latest_returns_only_most_recent_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-older.md",
        "needle content",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-20-newer.md",
        "needle content",
    );

    let output = wai_cmd(tmp.path())
        .args(["search", "needle", "--latest"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("newer"), "should contain the newer file");
    assert!(
        !stdout.contains("older"),
        "should not contain the older file"
    );
}

#[test]
fn search_tag_and_type_and_latest_combined() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-01-r1.md",
        "---\ntags: [perf]\n---\n\ndata",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-01-r2.md",
        "---\ntags: [perf]\n---\n\ndata",
    );
    // plan with same tag — should be excluded by --type research
    write_artifact(
        tmp.path(),
        "my-app",
        "plans",
        "2026-03-01-p1.md",
        "---\ntags: [perf]\n---\n\ndata",
    );

    let output = wai_cmd(tmp.path())
        .args([
            "search", "data", "--tag", "perf", "--type", "research", "--latest",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("r2"),
        "should match the latest research file"
    );
    assert!(!stdout.contains("r1"), "should exclude older research file");
    assert!(!stdout.contains("p1"), "should exclude the plan file");
}

// ─── wai timeline ───────────────────────────────────────────────────────────

#[test]
fn timeline_shows_dated_artifacts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-first.md",
        "First\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "plans",
        "2026-01-20-second.md",
        "Second\n",
    );

    wai_cmd(tmp.path())
        .args(["timeline", "my-app"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Timeline for")
                .and(predicate::str::contains("2026-01-20"))
                .and(predicate::str::contains("2026-01-10")),
        );
}

#[test]
fn timeline_empty_project_shows_message() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["timeline", "my-app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No dated artifacts found"));
}

#[test]
fn timeline_nonexistent_project_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["timeline", "ghost"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn timeline_from_filter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-05-old.md",
        "Old\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-15-new.md",
        "New\n",
    );

    wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--from", "2026-02-01"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("2026-02-15")
                .and(predicate::str::contains("2026-01-05").not()),
        );
}

#[test]
fn timeline_to_filter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-05-old.md",
        "Old\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-15-new.md",
        "New\n",
    );

    wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--to", "2026-01-31"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("2026-01-05")
                .and(predicate::str::contains("2026-02-15").not()),
        );
}

#[test]
fn timeline_from_to_range() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-01-jan.md",
        "Jan\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-01-feb.md",
        "Feb\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-03-01-mar.md",
        "Mar\n",
    );

    wai_cmd(tmp.path())
        .args([
            "timeline",
            "my-app",
            "--from",
            "2026-01-15",
            "--to",
            "2026-02-15",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("2026-02-01")
                .and(predicate::str::contains("2026-01-01").not())
                .and(predicate::str::contains("2026-03-01").not()),
        );
}

#[test]
fn timeline_reverse_shows_oldest_first() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-first.md",
        "First\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-03-20-third.md",
        "Third\n",
    );

    let output = wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--reverse"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let pos_jan = stdout.find("2026-01-10").unwrap();
    let pos_mar = stdout.find("2026-03-20").unwrap();
    assert!(
        pos_jan < pos_mar,
        "In reverse mode, oldest date should appear first"
    );
}

#[test]
fn timeline_json_outputs_entries() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-first.md",
        "First\n",
    );

    wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entries\""));
}

// ─── wai plugin list ────────────────────────────────────────────────────────

#[test]
fn plugin_list_shows_builtin_plugins() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["plugin", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("git")
                .or(predicate::str::contains("beads"))
                .or(predicate::str::contains("openspec")),
        );
}

#[test]
fn plugin_list_json_outputs_plugins() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["plugin", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\""));
}

// ─── wai plugin enable / disable ────────────────────────────────────────────

#[test]
fn plugin_enable_persists_to_config() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["plugin", "enable", "git"])
        .assert()
        .success()
        .stdout(predicate::str::contains("enabled"));

    let config = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert!(config.contains("git"));
    assert!(config.contains("enabled = true"));
}

#[test]
fn plugin_disable_persists_to_config() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Enable first, then disable
    wai_cmd(tmp.path())
        .args(["plugin", "enable", "git"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["plugin", "disable", "git"])
        .assert()
        .success()
        .stdout(predicate::str::contains("disabled"));

    let config = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert!(config.contains("git"));
    assert!(config.contains("enabled = false"));
}

#[test]
fn plugin_enable_unknown_plugin_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["plugin", "enable", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn plugin_enable_json_outputs_state() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["plugin", "enable", "git", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"plugin\"")
                .and(predicate::str::contains("\"enabled\": true")),
        );
}

// ─── wai status ─────────────────────────────────────────────────────────────

#[test]
fn status_shows_project_info() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-app"));
}

#[test]
fn status_json_outputs_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"suggestions\""));
}

#[test]
fn status_without_openspec_omits_section() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OpenSpec").not());
}

#[test]
fn status_with_openspec_shows_change_counts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Set up an openspec directory with a change
    let changes_dir = tmp.path().join("openspec/changes/add-feature");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Setup\n\n- [x] done\n- [ ] todo\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("OpenSpec")
                .and(predicate::str::contains("add-feature"))
                .and(predicate::str::contains("1/2")),
        );
}

#[test]
fn status_verbose_shows_section_breakdown() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let changes_dir = tmp.path().join("openspec/changes/add-feature");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Setup\n\n- [x] done\n- [ ] todo\n\n## 2. Implement\n\n- [ ] a\n- [ ] b\n",
    )
    .unwrap();

    // Also add a spec
    fs::create_dir_all(tmp.path().join("openspec/specs/core")).unwrap();

    wai_cmd(tmp.path())
        .args(["status", "-v"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Setup")
                .and(predicate::str::contains("Implement"))
                .and(predicate::str::contains("core")),
        );
}

#[test]
fn status_json_with_openspec_includes_field() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let changes_dir = tmp.path().join("openspec/changes/my-change");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Work\n\n- [x] a\n- [ ] b\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"openspec\":") // JSON key, not plugin name
                .and(predicate::str::contains("\"my-change\""))
                .and(predicate::str::contains("\"done\": 1")),
        );
}

#[test]
fn status_json_without_openspec_omits_field() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // The openspec *plugin* name appears in plugins list, so check for the JSON key specifically
    wai_cmd(tmp.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"openspec\":").not());
}

// ─── wai status: phase-aware suggestions ─────────────────────────────────────

#[test]
fn status_suggests_advance_when_enough_research() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Write 3 research artifacts to cross the readiness threshold
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-01-first.md",
        "First finding",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-02-second.md",
        "Second finding",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-03-third.md",
        "Third finding",
    );

    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("design").and(predicate::str::contains("Advance")));
}

#[test]
fn status_suggests_research_needed_when_in_implement_phase_without_research() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Jump to implement phase without adding any research
    wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("research"));
}

#[test]
fn status_shows_minimal_research_suggestion_for_new_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // New project with no artifacts → NewProject pattern → suggest adding research
    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("research"));
}

// ─── wai (no args) ─────────────────────────────────────────────────────────

#[test]
fn no_args_shows_welcome() {
    let stale_wai = std::path::Path::new("/tmp/.wai");
    if stale_wai.exists() {
        let _ = fs::remove_dir_all(stale_wai);
    }

    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("wai init"));
}

#[test]
fn no_args_in_initialized_dir_shows_commands() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("wai status"));
}

// ─── wai doctor ─────────────────────────────────────────────────────────────

#[test]
fn doctor_healthy_workspace_all_pass() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"fail\": 0")
                .and(predicate::str::contains("\"summary\""))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn doctor_missing_directories_fails_with_fix() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .assert()
        .code(1)
        .stdout(
            predicate::str::contains("\"status\": \"fail\"")
                .and(predicate::str::contains("archives"))
                .and(predicate::str::contains("doctor --fix")),
        );
}

#[test]
fn doctor_invalid_config_toml_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::write(tmp.path().join(".wai/config.toml"), "{{invalid toml!!!").unwrap();

    wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .assert()
        .code(1)
        .stdout(
            predicate::str::contains("\"status\": \"fail\"")
                .and(predicate::str::contains("Configuration")),
        );
}

#[test]
fn doctor_missing_plugin_tool_warns() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create .git dir so the git plugin is detected
    fs::create_dir(tmp.path().join(".git")).unwrap();

    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\": \"warn\""));
    assert!(stdout.contains("not found in PATH"));
}

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

    wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .assert()
        .code(1)
        .stdout(
            predicate::str::contains("\"status\": \"fail\"")
                .and(predicate::str::contains("broken")),
        );
}

#[test]
fn doctor_uninitialised_directory_errors() {
    let stale_wai = std::path::Path::new("/tmp/.wai");
    if stale_wai.exists() {
        let _ = fs::remove_dir_all(stale_wai);
    }

    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["doctor"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No project initialized"));
}

#[test]
fn doctor_fix_repairs_missing_directories() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Remove a directory
    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    // Run fix with --yes to skip confirmation
    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Fixed"));

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
    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Fixed"));
}

#[test]
fn doctor_fix_no_fixable_issues() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create CLAUDE.md with managed block to ensure healthy workspace
    fs::write(
        tmp.path().join("CLAUDE.md"),
        "<!-- WAI:START -->\n# Test\n<!-- WAI:END -->\n",
    )
    .unwrap();

    // Healthy workspace - no issues to fix
    wai_cmd(tmp.path())
        .args(["doctor", "--fix"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No fixable issues found"));
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
    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Fixed"));

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
    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .assert()
        .success();

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
    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--safe"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("apply doctor fixes").or(predicate::str::contains("--safe")),
        );
}

// ─── progressive disclosure help ────────────────────────────────────────────

#[test]
fn help_shows_quick_start_and_commands() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("QUICK START:")
                .and(predicate::str::contains("COMMANDS:"))
                .and(predicate::str::contains("Use -v for advanced options")),
        );
}

#[test]
fn help_default_hides_advanced_options() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ADVANCED OPTIONS:").not());
}

#[test]
fn help_v_shows_advanced_options() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["--help", "-v"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("ADVANCED OPTIONS:")
                .and(predicate::str::contains("--json"))
                .and(predicate::str::contains("--safe")),
        );
}

#[test]
fn help_vv_shows_environment_variables() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["--help", "-vv"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("ENVIRONMENT:")
                .and(predicate::str::contains("NO_COLOR"))
                .and(predicate::str::contains("WAI_LOG")),
        );
}

#[test]
fn help_vvv_shows_internals() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["--help", "-vvv"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("INTERNALS:")
                .and(predicate::str::contains("config.toml"))
                .and(predicate::str::contains("PARA")),
        );
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

    wai_cmd(tmp.path())
        .args(["search", "--help", "-vvv"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("EXAMPLES:")
                .and(predicate::str::contains("ADVANCED OPTIONS:"))
                .and(predicate::str::contains("ENVIRONMENT:"))
                .and(predicate::str::contains("INTERNALS:")),
        );
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

    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No project initialized"));

    wai_cmd(tmp.path())
        .args(["show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No project initialized"));
}

// ─── First-Run Detection ────────────────────────────────────────────────────

#[test]
fn first_run_detects_no_user_config() {
    // Create a temporary directory for user config
    let tmp_config = TempDir::new().unwrap();
    let tmp_project = TempDir::new().unwrap();

    // Set XDG_CONFIG_HOME to use our temp directory
    wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .assert()
        .success();

    // Verify user config was created
    assert!(tmp_config.path().join("wai/config.toml").exists());
}

#[test]
fn first_run_creates_default_user_config() {
    let tmp_config = TempDir::new().unwrap();
    let tmp_project = TempDir::new().unwrap();

    wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .assert()
        .success();

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
    wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .assert()
        .success();

    // Verify the flag is still true
    let config_content = fs::read_to_string(wai_config_dir.join("config.toml")).unwrap();
    assert!(config_content.contains("seen_tutorial = true"));
}

// ─── wai way ────────────────────────────────────────────────────────────────

#[test]
fn way_works_without_wai_init() {
    let tmp = TempDir::new().unwrap();

    // Don't init workspace - way should still work
    wai_cmd(tmp.path()).args(["way"]).assert().success().stdout(
        predicate::str::contains("Repo Hygiene & Agent Workflow Conventions"),
    );
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

    // All checks passing
    fs::write(tmp.path().join("justfile"), "test:\n\techo test").unwrap();
    fs::write(tmp.path().join("prek.toml"), "[hooks]").unwrap();
    fs::create_dir_all(tmp.path().join(".git/hooks")).unwrap();
    fs::write(tmp.path().join(".git/hooks/pre-commit"), "#!/bin/sh\nprek run").unwrap();
    fs::write(tmp.path().join(".editorconfig"), "root = true").unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join("LICENSE"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "# Instructions").unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".github/workflows/ci.yml"), "name: CI").unwrap();
    fs::create_dir_all(tmp.path().join(".devcontainer")).unwrap();

    let output = wai_cmd(tmp.path()).args(["way"]).output().unwrap();

    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn way_json_output_valid() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"checks\"")
                .and(predicate::str::contains("\"summary\""))
                .and(predicate::str::contains("\"pass\""))
                .and(predicate::str::contains("\"recommendations\"")),
        );
}

#[test]
fn way_json_includes_check_fields() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"name\"")
                .and(predicate::str::contains("\"status\""))
                .and(predicate::str::contains("\"message\"")),
        );
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

    wai_cmd(tmp.path())
        .args(["way"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ℹ"))
        .stderr(predicate::str::contains("best practices adopted"));
}

#[test]
fn way_minimal_repository_has_suggestions() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    wai_cmd(tmp.path())
        .args(["way"])
        .assert()
        .success()
        .stdout(predicate::str::contains("→").and(predicate::str::contains("https://")));
}

#[test]
fn way_complete_repository_all_pass() {
    let tmp = TempDir::new().unwrap();

    // Set up all best practices
    fs::write(tmp.path().join("justfile"), "test:\n\techo test").unwrap();
    fs::write(tmp.path().join("prek.toml"), "[hooks]").unwrap();
    fs::create_dir_all(tmp.path().join(".git/hooks")).unwrap();
    fs::write(tmp.path().join(".git/hooks/pre-commit"), "#!/bin/sh\nprek run").unwrap();
    fs::write(tmp.path().join(".editorconfig"), "root = true").unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join("LICENSE"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "# Instructions").unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".github/workflows/ci.yml"), "name: CI").unwrap();
    fs::create_dir_all(tmp.path().join(".devcontainer")).unwrap();

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

    // Set up all best practices
    fs::write(tmp.path().join("justfile"), "test:\n\techo test").unwrap();
    fs::write(tmp.path().join("prek.toml"), "[hooks]").unwrap();
    fs::create_dir_all(tmp.path().join(".git/hooks")).unwrap();
    fs::write(tmp.path().join(".git/hooks/pre-commit"), "#!/bin/sh\nprek run").unwrap();
    fs::write(tmp.path().join(".editorconfig"), "root = true").unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join("LICENSE"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "# Instructions").unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".github/workflows/ci.yml"), "name: CI").unwrap();
    fs::create_dir_all(tmp.path().join(".devcontainer")).unwrap();

    wai_cmd(tmp.path())
        .args(["way"])
        .assert()
        .success()
        .stdout(predicate::str::contains("✓"));
}

// ─── wai way: unit tests for individual checks ─────────────────────────────

#[test]
fn way_check_task_runner_justfile() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("justfile"), "test:\n\techo test").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Command standardization")
                .and(predicate::str::contains("justfile detected"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_task_runner_makefile() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("Makefile"), "test:\n\techo test").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Command standardization")
                .and(predicate::str::contains("Makefile detected"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_task_runner_missing() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Command standardization")
                .and(predicate::str::contains("No task runner detected"))
                .and(predicate::str::contains("\"info\"")),
        );
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

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Pre-commit quality gates")
                .and(predicate::str::contains("prek detected and installed"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_git_hooks_prek_not_installed() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("prek.toml"), "[hooks]").unwrap();
    // No .git/hooks/pre-commit — hooks not installed

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Pre-commit quality gates")
                .and(predicate::str::contains("prek.toml found but hooks not installed"))
                .and(predicate::str::contains("\"info\"")),
        );
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

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Pre-commit quality gates")
                .and(predicate::str::contains("pre-commit detected and installed"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_git_hooks_precommit_not_installed() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".pre-commit-config.yaml"), "repos: []").unwrap();
    // No .git/hooks/pre-commit

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Pre-commit quality gates")
                .and(predicate::str::contains(
                    ".pre-commit-config.yaml found but hooks not installed",
                ))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_git_hooks_lefthook_installed() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("lefthook.yml"), "pre-commit:\n  commands: {}").unwrap();
    fs::create_dir_all(tmp.path().join(".git/hooks")).unwrap();
    fs::write(
        tmp.path().join(".git/hooks/pre-commit"),
        "#!/bin/sh\nlefthook run pre-commit",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Pre-commit quality gates")
                .and(predicate::str::contains("lefthook detected and installed"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_git_hooks_lefthook_not_installed() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("lefthook.yml"), "pre-commit:\n  commands: {}").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Pre-commit quality gates")
                .and(predicate::str::contains(
                    "lefthook.yml found but hooks not installed",
                ))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_git_hooks_missing() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Pre-commit quality gates")
                .and(predicate::str::contains("No git hook manager detected"))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_editorconfig_present() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".editorconfig"), "root = true").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Consistent formatting")
                .and(predicate::str::contains(".editorconfig detected"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_editorconfig_missing() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Consistent formatting")
                .and(predicate::str::contains("No .editorconfig detected"))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_documentation_complete() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join("LICENSE"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Project documentation")
                .and(predicate::str::contains("Complete"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_documentation_not_configured() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Project documentation")
                .and(predicate::str::contains("Not configured"))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_documentation_missing_critical() {
    let tmp = TempDir::new().unwrap();
    // Only LICENSE and CONTRIBUTING (missing critical README and .gitignore)
    fs::write(tmp.path().join("LICENSE"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Project documentation")
                .and(predicate::str::contains("Missing critical files"))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_documentation_partial() {
    let tmp = TempDir::new().unwrap();
    // Has critical files but missing LICENSE and CONTRIBUTING
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Project documentation")
                .and(predicate::str::contains("Partial documentation"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_documentation_license_md_variant() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join("LICENSE.md"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Project documentation")
                .and(predicate::str::contains("Complete"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_ai_instructions_claude_md() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "# Instructions").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("AI-agent context")
                .and(predicate::str::contains("CLAUDE.md detected"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_ai_instructions_agents_md() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("AGENTS.md"), "# Instructions").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("AI-agent context")
                .and(predicate::str::contains("AGENTS.md detected"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_ai_instructions_missing() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("AI-agent context")
                .and(predicate::str::contains("No AI instruction files detected"))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_cicd_github_actions() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".github/workflows/ci.yml"), "name: CI").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Automated verification")
                .and(predicate::str::contains("GitHub Actions configured"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_cicd_gitlab() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".gitlab-ci.yml"), "stages: [test]").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Automated verification")
                .and(predicate::str::contains("GitLab CI configured"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_cicd_circleci() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".circleci")).unwrap();
    fs::write(tmp.path().join(".circleci/config.yml"), "version: 2.1").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Automated verification")
                .and(predicate::str::contains("CircleCI configured"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_cicd_missing() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Automated verification")
                .and(predicate::str::contains("No CI/CD configuration detected"))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_devcontainer_dir() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".devcontainer")).unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Reproducible environments")
                .and(predicate::str::contains(
                    ".devcontainer/ directory detected",
                ))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_devcontainer_json() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".devcontainer.json"), "{}").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Reproducible environments")
                .and(predicate::str::contains(".devcontainer.json detected"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_devcontainer_missing() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Reproducible environments")
                .and(predicate::str::contains(
                    "No dev container configuration detected",
                ))
                .and(predicate::str::contains("\"info\"")),
        );
}

// ─── wai way: new feature checks ────────────────────────────────────────────

#[test]
fn way_check_llm_txt_present() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("llm.txt"), "# Project docs").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("LLM-friendly context")
                .and(predicate::str::contains("llm.txt detected"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_llm_txt_missing() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("LLM-friendly context")
                .and(predicate::str::contains("No llm.txt detected"))
                .and(predicate::str::contains("llmstxt.org"))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_agent_skills_present() {
    let tmp = TempDir::new().unwrap();
    let skills_base = tmp.path().join(".wai/resources/agent-config/skills");
    fs::create_dir_all(skills_base.join("rule-of-5-universal")).unwrap();
    fs::write(
        skills_base.join("rule-of-5-universal/SKILL.md"),
        "# Rule of 5 Universal",
    )
    .unwrap();
    fs::create_dir_all(skills_base.join("commit")).unwrap();
    fs::write(skills_base.join("commit/SKILL.md"), "# Commit").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Extended agent capabilities")
                .and(predicate::str::contains("2 skill(s) configured"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

#[test]
fn way_check_agent_skills_empty_dir() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".wai/resources/agent-config/skills")).unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Extended agent capabilities")
                .and(predicate::str::contains(
                    "Skills directory present but empty",
                ))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_agent_skills_missing() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Extended agent capabilities")
                .and(predicate::str::contains("No skills configured"))
                .and(predicate::str::contains("\"info\"")),
        );
}

#[test]
fn way_check_justfile_recipes() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("justfile"),
        "test:\n\techo test\n\ninstall:\n\techo install\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Command standardization")
                .and(predicate::str::contains("recipes:"))
                .and(predicate::str::contains("\"pass\"")),
        );
}

// ─── Post-Command Suggestions ────────────────────────────────────────────────

#[test]
fn new_project_shows_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["new", "project", "test-proj"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Add research: wai add research")
                .and(predicate::str::contains("Check project phase: wai phase"))
                .and(predicate::str::contains("Check status: wai status")),
        );
}

#[test]
fn add_research_shows_context_aware_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-proj");

    // First research artifact - should suggest adding more research
    wai_cmd(tmp.path())
        .args(["add", "research", "Initial research"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Add more research: wai add research")
                .and(predicate::str::contains("Check phase: wai phase")),
        );

    // Second research artifact - should suggest moving to design phase
    wai_cmd(tmp.path())
        .args(["add", "research", "More detailed findings"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Add more research: wai add research")
                .and(predicate::str::contains(
                    "Move to design phase: wai phase set design",
                ))
                .and(predicate::str::contains("Review research: wai search")),
        );
}

#[test]
fn phase_next_shows_phase_specific_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-proj");

    // Advance from research to design - should show design suggestions
    wai_cmd(tmp.path())
        .args(["phase", "next"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Add design: wai add design")
                .and(predicate::str::contains("Review research: wai search"))
                .and(predicate::str::contains("Show project details: wai show")),
        );
}

#[test]
fn phase_set_shows_phase_specific_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-proj");

    // Set to implement phase - should show implement suggestions
    wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Show project details: wai show")
                .and(predicate::str::contains(
                    "Add implementation notes: wai add plan",
                ))
                .and(predicate::str::contains("Check status: wai status")),
        );
}

// ─── Context Inference ───────────────────────────────────────────────────────

#[test]
fn typo_suggestion_shown_outside_workspace() {
    // Outside any workspace, a typo should still show "Did you mean?" rather
    // than just "No project initialized".
    let tmp = TempDir::new().unwrap();
    // NOT calling init_workspace — this dir has no .wai/

    wai_cmd(tmp.path())
        .args(["statu"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Did you mean 'status'")
                .or(predicate::str::contains("did you mean 'status'")),
        );
}

#[test]
fn wrong_order_shown_outside_workspace() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["project", "new"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("new project").and(predicate::str::contains("Did you mean")),
        );
}

// ─── Wrong Order Detection ────────────────────────────────────────────────────

#[test]
fn wrong_order_project_new_suggests_correction() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // "wai project new" should suggest "wai new project"
    wai_cmd(tmp.path())
        .args(["project", "new"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("new project").and(predicate::str::contains("Did you mean")),
        );
}

#[test]
fn wrong_order_research_add_suggests_correction() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // "wai research add" should suggest "wai add research"
    wai_cmd(tmp.path())
        .args(["research", "add"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("add research").and(predicate::str::contains("Did you mean")),
        );
}

// ─── Typo Detection ───────────────────────────────────────────────────────────

#[test]
fn typo_suggests_closest_command() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // "statu" is a typo of "status"
    wai_cmd(tmp.path())
        .args(["statu"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Did you mean 'status'")
                .or(predicate::str::contains("did you mean 'status'")),
        );
}

#[test]
fn typo_suggests_doctor_for_doctr() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["doctr"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Did you mean 'doctor'")
                .or(predicate::str::contains("did you mean 'doctor'")),
        );
}

#[test]
fn completely_unknown_command_shows_help_hint() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // "xyz" is not similar to any command
    wai_cmd(tmp.path())
        .args(["xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("wai --help"));
}

// ─── wai why ─────────────────────────────────────────────────────────────────

/// Helper: write `privacy_notice_shown = true` into the [llm] config section
/// without overriding the `llm` setting. Used by tests that exercise the
/// auto-detect backend path so the one-time notice does not interfere with
/// stdout/stderr assertions.
fn set_privacy_notice_shown(dir: &std::path::Path) {
    let config_path = dir.join(".wai").join("config.toml");
    let existing = fs::read_to_string(&config_path).unwrap_or_default();
    // Strip any existing [llm] or legacy [why] section before appending a
    // fresh one to avoid duplicate-section TOML errors.
    let base = existing
        .split("[llm]")
        .next()
        .and_then(|s| {
            // Also strip a trailing [why] that may be present in workspaces
            // initialised by an older version of wai.
            Some(s.split("[why]").next().unwrap_or(s))
        })
        .unwrap_or(&existing)
        .trim_end();
    let updated = format!("{}\n[llm]\nprivacy_notice_shown = true\n", base);
    fs::write(&config_path, updated).unwrap();
}

/// Helper: add an [llm] section to .wai/config.toml forcing a specific backend.
///
/// Setting `llm = "claude"` without an API key guarantees detect_backend()
/// returns None regardless of whether Ollama is installed, so fallback tests
/// are deterministic on any machine.
fn force_why_llm(dir: &std::path::Path, llm: &str) {
    let config_path = dir.join(".wai").join("config.toml");
    let existing = fs::read_to_string(&config_path).unwrap_or_default();
    // Strip any existing [llm] or legacy [why] section before appending to
    // avoid duplicate-section TOML errors.
    let base = existing
        .split("[llm]")
        .next()
        .and_then(|s| Some(s.split("[why]").next().unwrap_or(s)))
        .unwrap_or(&existing)
        .trim_end();
    let updated = format!(
        "{}\n[llm]\nllm = \"{}\"\nprivacy_notice_shown = true\n",
        base, llm
    );
    fs::write(&config_path, updated).unwrap();
}

/// 7.5 — `--no-llm` flag bypasses LLM and falls back to `wai search` output.
#[test]
fn why_no_llm_flag_falls_back_to_search() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable and well-supported.",
    );

    // --no-llm must succeed and produce search-style output
    wai_cmd(tmp.path())
        .args(["why", "--no-llm", "TOML"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TOML"));
}

/// 7.5 — When no LLM backend is configured or available, the command warns
/// and automatically falls back to `wai search`.
#[test]
fn why_auto_fallback_to_search_when_no_llm_available() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable.",
    );

    // Force claude backend (no key) so Ollama auto-detection is skipped
    force_why_llm(tmp.path(), "claude");

    wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .success()
        // warning goes to stderr; fallback search results go to stdout
        // message varies based on whether Ollama/claude is installed
        .stderr(predicate::str::contains("Falling back to"));
}

/// 7.6 — A file-path query in a non-git repository does not crash.
///
/// The temp directory is not a git repo, so gather_git_file_context() must
/// return None gracefully. The command falls back to `wai search`.
#[test]
fn why_file_query_in_non_git_repo_does_not_crash() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "Architecture decision: use Rust for performance.",
    );

    // Force claude backend (no key) → deterministic fallback to search
    force_why_llm(tmp.path(), "claude");

    // Use a path-like query; the temp dir is not a git repo
    wai_cmd(tmp.path())
        .args(["why", "src/main.rs"])
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .success(); // must not crash due to missing git
}

// ─── wai-96y9: Phase 8 — Agent Backend Integration Tests ────────────────────

/// 8.1 — CLAUDECODE=1, no API key → auto-detect selects AgentBackend;
/// wai why prints [AGENT CONTEXT] block to stdout and exits 0.
#[test]
fn why_agent_mode_autodetect_when_claudecode_set() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable and well-supported.",
    );

    // Suppress the one-time privacy notice; no llm override (auto-detect path).
    set_privacy_notice_shown(tmp.path());

    wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .env("CLAUDECODE", "1")
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .success()
        .stdout(predicate::str::contains("[AGENT CONTEXT]"));
}

/// 8.2 — CLAUDECODE unset, no API key, no claude/ollama binaries in PATH →
/// auto-detect finds no backend; wai why falls back to wai search and exits 0.
#[test]
fn why_fallback_to_search_when_no_agent_no_api_key_no_cli() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen for its human-readable syntax.",
    );

    // Create an empty bin directory so that `claude` and `ollama` are not found.
    let empty_bin = tmp.path().join("empty_bin");
    fs::create_dir_all(&empty_bin).unwrap();

    wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("CLAUDECODE")
        .env("PATH", &empty_bin)
        .assert()
        .success()
        .stderr(predicate::str::contains("No LLM available"));
}

/// 8.3 — Explicit `[llm] llm = "agent"` selects AgentBackend regardless of
/// whether CLAUDECODE is set; wai why prints [AGENT CONTEXT] to stdout and
/// exits 0, and emits a warning that no Claude Code session is active.
#[test]
fn why_explicit_agent_backend_ignores_claudecode() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable and well-supported.",
    );

    // Explicitly configure agent backend (also sets privacy_notice_shown = true).
    force_why_llm(tmp.path(), "agent");

    wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .env_remove("CLAUDECODE") // no active Claude Code session
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .success()
        .stdout(predicate::str::contains("[AGENT CONTEXT]"))
        // A warning must be emitted because CLAUDECODE is not set.
        .stderr(predicate::str::contains("agent mode requires"));
}

// ─── wai-44b: Conversational Error Tone ─────────────────────────────────────

#[test]
fn error_not_initialized_is_conversational() {
    let stale_wai = std::path::Path::new("/tmp/.wai");
    if stale_wai.exists() {
        let _ = fs::remove_dir_all(stale_wai);
    }

    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No project initialized"));
}

#[test]
fn error_project_not_found_is_conversational() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["add", "research", "--project", "nonexistent", "notes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// ─── wai-933: Integration tests for resource management ──────────────────────

// Helper: write a projections.yml into an initialized workspace
fn write_projections_yml(dir: &std::path::Path, content: &str) {
    let path = dir.join(".wai/resources/agent-config/.projections.yml");
    fs::write(path, content).unwrap();
}

// ── wai resource add skill ───────────────────────────────────────────────────

#[test]
fn resource_add_skill_creates_directory_and_skill_md() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "my-skill"])
        .assert()
        .success();

    let skill_dir = tmp
        .path()
        .join(".wai/resources/agent-config/skills/my-skill");
    assert!(skill_dir.is_dir(), "skill directory should be created");

    let skill_md = skill_dir.join("SKILL.md");
    assert!(skill_md.is_file(), "SKILL.md should be created");

    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("---"),
        "SKILL.md should have frontmatter delimiters"
    );
    assert!(
        content.contains("name: my-skill"),
        "SKILL.md should include skill name"
    );
    assert!(
        content.contains("description:"),
        "SKILL.md should include description field"
    );
}

#[test]
fn resource_add_skill_fails_on_duplicate() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "dup-skill"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "dup-skill"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn resource_add_skill_fails_on_uppercase_name() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "MySkill"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Invalid character")
                .or(predicate::str::contains("Invalid skill")),
        );
}

#[test]
fn resource_add_skill_fails_on_consecutive_hyphens() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "my--skill"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("consecutive hyphens"));
}

#[test]
fn resource_add_skill_fails_on_too_long_name() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let long_name = "a".repeat(65);
    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", &long_name])
        .assert()
        .failure()
        .stderr(predicate::str::contains("too long"));
}

#[test]
fn resource_add_skill_fails_on_dot_prefix() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", ".hidden"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("cannot start with '.'")
                .or(predicate::str::contains("Invalid skill")),
        );
}

#[test]
fn resource_add_skill_refuses_in_safe_mode() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["--safe", "resource", "add", "skill", "my-skill"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("add skill").or(predicate::str::contains("Safe mode")));
}

// ── wai resource add skill (hierarchical) ────────────────────────────────────

#[test]
fn resource_add_hierarchical_skill_creates_nested_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "issue/gather"])
        .assert()
        .success();

    let skill_dir = tmp
        .path()
        .join(".wai/resources/agent-config/skills/issue/gather");
    assert!(
        skill_dir.is_dir(),
        "nested skill directory should be created"
    );

    let skill_md = skill_dir.join("SKILL.md");
    assert!(
        skill_md.is_file(),
        "SKILL.md should be created in nested directory"
    );

    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("name: issue/gather"),
        "SKILL.md should include full hierarchical name"
    );
}

#[test]
fn resource_list_skills_shows_hierarchical_and_flat() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "issue/gather"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "plain-skill"])
        .assert()
        .success();

    let output = wai_cmd(tmp.path())
        .args(["resource", "list", "skills"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(
        stdout.contains("issue/gather") || stdout.contains("gather"),
        "list should show hierarchical skill; got: {}",
        stdout
    );
    assert!(
        stdout.contains("plain-skill"),
        "list should show flat skill; got: {}",
        stdout
    );
}

#[test]
fn resource_add_hierarchical_skill_conflicts_with_flat_skill() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a flat skill named "issue"
    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "issue"])
        .assert()
        .success();

    // Attempting to create "issue/gather" should fail with a conflict message
    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "issue/gather"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("issue").and(
            predicate::str::contains("flat skill").or(predicate::str::contains("already exists")),
        ));
}

// ── wai resource add skill --template ────────────────────────────────────────

#[test]
fn resource_add_skill_template_gather_contains_wai_search() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args([
            "resource",
            "add",
            "skill",
            "my-gather",
            "--template",
            "gather",
        ])
        .assert()
        .success();

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/my-gather/SKILL.md");
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("wai search"),
        "gather template should contain wai search"
    );
    assert!(
        content.contains("wai add research"),
        "gather template should contain wai add research"
    );
    assert!(
        content.contains("$ARGUMENTS"),
        "gather template should use $ARGUMENTS placeholder"
    );
    assert!(
        content.contains("$PROJECT"),
        "gather template should use $PROJECT placeholder"
    );
}

#[test]
fn resource_add_skill_template_unknown_fails_with_valid_names() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args([
            "resource",
            "add",
            "skill",
            "my-skill",
            "--template",
            "bogus",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("gather")
                .and(predicate::str::contains("tdd"))
                .and(predicate::str::contains("rule-of-5")),
        );
}

#[test]
fn resource_add_skill_no_template_still_creates_bare_stub() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "bare-skill"])
        .assert()
        .success();

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/bare-skill/SKILL.md");
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("name: bare-skill"),
        "bare stub should have skill name"
    );
    assert!(
        !content.contains("$ARGUMENTS"),
        "bare stub should not use $ARGUMENTS"
    );
}

// ── wai resource list skills ─────────────────────────────────────────────────

#[test]
fn resource_list_skills_shows_skill_name() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Add a skill then update SKILL.md with a real description
    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "list-skill"])
        .assert()
        .success();

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/list-skill/SKILL.md");
    fs::write(
        &skill_md,
        "---\nname: list-skill\ndescription: A skill for listing tests\n---\n\n# List Skill\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["resource", "list", "skills"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list-skill"));
}

#[test]
fn resource_list_skills_empty_shows_hint() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // No skills added — directory is empty after init
    wai_cmd(tmp.path())
        .args(["resource", "list", "skills"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No skills found")
                .or(predicate::str::contains("resource add skill")),
        );
}

#[test]
fn resource_list_skills_bad_frontmatter_shows_no_metadata() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a skill directory with a SKILL.md that has no valid frontmatter
    let skill_dir = tmp
        .path()
        .join(".wai/resources/agent-config/skills/broken-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        "# Just a heading, no frontmatter",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["resource", "list", "skills"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no metadata"));
}

#[test]
fn resource_list_skills_json_has_skills_key() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["resource", "list", "skills", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"skills\""));
}

// ── wai resource import skills ───────────────────────────────────────────────

#[test]
fn resource_import_skills_from_custom_path() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a skill in a custom source directory
    let source = tmp.path().join("ext-skills/cool-skill");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: cool-skill\ndescription: An imported skill\n---\n\n# Cool Skill\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args([
            "resource",
            "import",
            "skills",
            "--from",
            tmp.path().join("ext-skills").to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills/cool-skill")
            .is_dir(),
        "imported skill directory should exist"
    );
}

#[test]
fn resource_import_skills_from_default_agents_path() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a skill in the default .agents/skills/ location
    let source = tmp.path().join(".agents/skills/default-skill");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: default-skill\ndescription: From default path\n---\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["resource", "import", "skills"])
        .assert()
        .success();

    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills/default-skill")
            .is_dir()
    );
}

#[test]
fn resource_import_skills_skips_existing_with_warning() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Add a skill that already exists in the workspace
    wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "existing-skill"])
        .assert()
        .success();

    // Create source with the same skill name
    let source = tmp.path().join("source/existing-skill");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: existing-skill\ndescription: Would conflict\n---\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args([
            "resource",
            "import",
            "skills",
            "--from",
            tmp.path().join("source").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Skipped").or(predicate::str::contains("already exists")));
}

#[test]
fn resource_import_skills_copies_full_directory_tree() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a skill with nested files (not just SKILL.md)
    let source = tmp.path().join("source/rich-skill");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: rich-skill\ndescription: A rich skill\n---\n",
    )
    .unwrap();
    fs::write(source.join("examples.md"), "# Examples").unwrap();
    fs::create_dir_all(source.join("templates")).unwrap();
    fs::write(source.join("templates/base.md"), "# Base Template").unwrap();

    wai_cmd(tmp.path())
        .args([
            "resource",
            "import",
            "skills",
            "--from",
            tmp.path().join("source").to_str().unwrap(),
        ])
        .assert()
        .success();

    let target = tmp
        .path()
        .join(".wai/resources/agent-config/skills/rich-skill");
    assert!(target.join("SKILL.md").is_file());
    assert!(
        target.join("examples.md").is_file(),
        "extra files should be copied"
    );
    assert!(
        target.join("templates/base.md").is_file(),
        "subdirectory contents should be copied"
    );
}

// ── Doctor projection consistency ────────────────────────────────────────────

#[test]
fn doctor_warns_on_missing_projection_source_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Configure a projection referencing a source that doesn't exist
    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: AGENTS.md\n    strategy: inline\n    sources: [nonexistent-dir]\n",
    );

    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("\"status\": \"warn\""),
        "doctor should warn about missing source, got: {}",
        stdout
    );
}

#[test]
fn doctor_detects_stale_inline_projection() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a source directory with a file
    let source_dir = tmp.path().join(".wai/resources/agent-config/my-docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("intro.md"), "# Original").unwrap();

    // Configure and sync an inline projection
    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: AGENTS.md\n    strategy: inline\n    sources: [my-docs]\n",
    );
    wai_cmd(tmp.path()).args(["sync"]).assert().success();

    // Modify the source to make the projection stale
    fs::write(source_dir.join("intro.md"), "# Modified content").unwrap();

    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Stale") || stdout.contains("\"status\": \"warn\""),
        "doctor should detect stale inline projection, got: {}",
        stdout
    );
}

#[test]
#[cfg(unix)]
fn doctor_detects_stale_symlink_projection() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a source directory with a file
    let source_dir = tmp.path().join(".wai/resources/agent-config/link-source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("rule.md"), "# A rule").unwrap();

    // Configure and sync a symlink projection
    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: .agents/rules\n    strategy: symlink\n    sources: [link-source]\n",
    );
    wai_cmd(tmp.path()).args(["sync"]).assert().success();

    // Remove the source file to create a broken symlink
    fs::remove_file(source_dir.join("rule.md")).unwrap();

    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("broken") || stdout.contains("\"status\": \"warn\""),
        "doctor should detect broken symlink, got: {}",
        stdout
    );
}

#[test]
fn doctor_passes_for_in_sync_inline_projection() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create source and sync
    let source_dir = tmp.path().join(".wai/resources/agent-config/sync-docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("notes.md"), "# Notes").unwrap();

    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: AGENTS.md\n    strategy: inline\n    sources: [sync-docs]\n",
    );
    wai_cmd(tmp.path()).args(["sync"]).assert().success();

    // Doctor should see the projection as in-sync
    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The projection check should pass (or at least not add a new warn/fail)
    assert!(
        stdout.contains("In sync") || !stdout.contains("Stale"),
        "doctor should see synced projection as passing, got: {}",
        stdout
    );
}

// ─── wai-7gk: Interactive Ambiguity Resolution ───────────────────────────────

#[test]
fn add_research_no_input_fails_when_multiple_projects() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    wai_cmd(tmp.path())
        .args(["--no-input", "add", "research", "some notes"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Multiple projects").or(predicate::str::contains("--project")),
        );
}

#[test]
fn add_research_explicit_project_works_with_multiple() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    wai_cmd(tmp.path())
        .args(["add", "research", "--project", "alpha", "targeting alpha"])
        .assert()
        .success();

    // Verify the artifact landed in the right project
    let research_dir = tmp.path().join(".wai/projects/alpha/research");
    let entries: Vec<_> = fs::read_dir(research_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !entries.is_empty(),
        "research artifact should be in alpha project"
    );
}

#[test]
fn add_research_single_project_still_works_without_flag() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "solo");

    wai_cmd(tmp.path())
        .args(["add", "research", "just one project"])
        .assert()
        .success();
}

// ─── wai-rsp: Context Suggestions Testing ────────────────────────────────────

#[test]
fn add_research_shows_phase_appropriate_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "suggest-proj");

    // Add enough research to trigger the "advance to design" suggestion (≥2 artifacts)
    wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "--project",
            "suggest-proj",
            "first finding",
        ])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "--project",
            "suggest-proj",
            "second finding",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Add more research")
                .or(predicate::str::contains("Move to design"))
                .or(predicate::str::contains("wai phase")),
        );
}

#[test]
fn add_research_in_non_research_phase_shows_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "impl-proj");

    // Advance to implement phase (phase command picks the only project automatically)
    wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .assert()
        .success();

    // In non-research phases, adding research still shows next-step suggestions
    wai_cmd(tmp.path())
        .args(["add", "research", "--project", "impl-proj", "context note"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("wai add research").or(predicate::str::contains("wai search")),
        );
}

// ─── wai close ───────────────────────────────────────────────────────────────

#[test]
fn close_single_project_creates_handoff() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // Create .beads dir so the beads plugin is detected
    fs::create_dir(tmp.path().join(".beads")).unwrap();

    let output = wai_cmd(tmp.path()).args(["close"]).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Handoff file should have been created
    let handoffs_dir = tmp.path().join(".wai/projects/myproject/handoffs");
    let files: Vec<_> = fs::read_dir(&handoffs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);

    // Output should contain the handoff path line
    assert!(
        stdout.contains("Handoff created:"),
        "expected 'Handoff created:' in stdout, got: {stdout}"
    );
    // Output should contain next-steps with bd sync (beads detected)
    assert!(
        stdout.contains("bd sync --from-main"),
        "expected 'bd sync --from-main' in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("→ Next:"),
        "expected '→ Next:' in stdout, got: {stdout}"
    );
}

#[test]
fn close_with_project_flag_skips_prompt() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    wai_cmd(tmp.path())
        .args(["close", "--project", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Handoff created:"));

    // Only alpha should have a handoff
    let alpha_handoffs = tmp.path().join(".wai/projects/alpha/handoffs");
    let beta_handoffs = tmp.path().join(".wai/projects/beta/handoffs");
    let alpha_files: Vec<_> = fs::read_dir(&alpha_handoffs)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let beta_files: Vec<_> = fs::read_dir(&beta_handoffs)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(alpha_files.len(), 1);
    assert_eq!(beta_files.len(), 0);
}

#[test]
fn close_unknown_project_shows_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["close", "--project", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

#[test]
fn close_zero_projects_shows_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["close", "--no-input"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No projects found"));
}

#[test]
fn close_repeated_same_day_increments_suffix() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // First invocation
    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success();

    // Second invocation on same day
    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success();

    let handoffs_dir = tmp.path().join(".wai/projects/myproject/handoffs");
    let mut files: Vec<_> = fs::read_dir(&handoffs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    files.sort();

    assert_eq!(files.len(), 2, "expected 2 handoff files, got: {:?}", files);
    // One file should have no numeric suffix and one should have the -1 suffix
    assert!(
        files.iter().any(|f| f.ends_with("session-end.md")),
        "expected a session-end.md file, got: {:?}",
        files
    );
    assert!(
        files.iter().any(|f| f.ends_with("session-end-1.md")),
        "expected a session-end-1.md file, got: {:?}",
        files
    );
}

#[test]
fn close_writes_pending_resume_signal() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success();

    let proj_dir = tmp.path().join(".wai/projects/myproject");
    let signal = proj_dir.join(".pending-resume");
    assert!(
        signal.exists(),
        ".pending-resume should be written after close"
    );

    let content = fs::read_to_string(&signal).unwrap();
    let relative = content.trim();
    // Should point to a file under handoffs/
    assert!(
        relative.starts_with("handoffs/"),
        ".pending-resume should contain a handoffs/ relative path, got: {relative}"
    );
    // The referenced file should actually exist
    assert!(
        proj_dir.join(relative).exists(),
        "handoff referenced by .pending-resume should exist at {relative}"
    );
}

#[test]
fn close_overwrites_pending_resume_on_second_call() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success();

    let proj_dir = tmp.path().join(".wai/projects/myproject");
    let first = fs::read_to_string(proj_dir.join(".pending-resume")).unwrap();

    // Second close on the same day produces a suffixed file
    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success();

    let second = fs::read_to_string(proj_dir.join(".pending-resume")).unwrap();
    assert_ne!(
        first.trim(),
        second.trim(),
        ".pending-resume should be overwritten with the newest handoff path"
    );
    // The new path should exist
    assert!(
        proj_dir.join(second.trim()).exists(),
        "second .pending-resume path should exist"
    );
}

// ─── wai prime ───────────────────────────────────────────────────────────────

/// Helper: write a handoff file directly into a project's handoffs directory.
fn write_handoff(dir: &std::path::Path, project: &str, filename: &str, content: &str) {
    let handoffs = dir
        .join(".wai")
        .join("projects")
        .join(project)
        .join("handoffs");
    fs::create_dir_all(&handoffs).unwrap();
    fs::write(handoffs.join(filename), content).unwrap();
}

#[test]
fn prime_single_project_with_handoff() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    write_handoff(
        tmp.path(),
        "myproject",
        "2026-02-23-session-end.md",
        "---\ndate: 2026-02-23\nproject: myproject\nphase: research\n---\n\n# Session Handoff\n\nCompleted the initial research phase.\n",
    );

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wai prime"))
        .stdout(predicate::str::contains("Project: myproject"))
        .stdout(predicate::str::contains("Handoff: 2026-02-23"))
        .stdout(predicate::str::contains(
            "Completed the initial research phase.",
        ));
}

#[test]
fn prime_no_handoff_omits_handoff_line() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project: myproject"));

    out.stdout(predicate::str::contains("Handoff:").not());
}

#[test]
fn prime_project_flag_selects_correct_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    wai_cmd(tmp.path())
        .args(["prime", "--project", "alpha", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project: alpha"));
}

#[test]
fn prime_zero_projects_shows_suggestion() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["prime", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wai new project"));
}

#[test]
fn prime_multiple_projects_no_input_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    wai_cmd(tmp.path())
        .args(["prime", "--no-input"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("prime --project"));
}

#[test]
fn prime_unknown_project_fails_with_available() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["prime", "--project", "doesnotexist", "--no-input"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("doesnotexist"))
        .stderr(predicate::str::contains("myproject"));
}

#[test]
fn prime_all_headings_handoff_shows_no_summary_yet() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    write_handoff(
        tmp.path(),
        "myproject",
        "2026-02-23-session-end.md",
        "---\ndate: 2026-02-23\nproject: myproject\nphase: research\n---\n\n# Heading One\n\n## Heading Two\n",
    );

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no summary yet"));
}

#[test]
fn prime_outside_workspace_fails() {
    let tmp = TempDir::new().unwrap();
    // No wai init — no .wai/ directory

    wai_cmd(tmp.path())
        .args(["prime", "--no-input"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("wai init"));
}

/// Helper: write a `.pending-resume` file pointing to a handoff in the project dir.
fn write_pending_resume(dir: &std::path::Path, project: &str, handoff_relative: &str) {
    let proj_dir = dir.join(".wai/projects").join(project);
    fs::write(proj_dir.join(".pending-resume"), handoff_relative).unwrap();
}

#[test]
fn prime_shows_resuming_block_when_pending_resume_present_today() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let today = Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("{today}-session-end.md");
    let content = format!(
        "---\ndate: {today}\nproject: myproject\nphase: implement\n---\n\n\
         Implementing the state machine.\n\n\
         ## Next Steps\n\n\
         1. Finish src/state.rs\n\
         2. Write tests\n"
    );
    write_handoff(tmp.path(), "myproject", &filename, &content);
    write_pending_resume(tmp.path(), "myproject", &format!("handoffs/{filename}"));

    let stdout = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let out = String::from_utf8(stdout).unwrap();

    assert!(
        out.contains("RESUMING"),
        "expected RESUMING in output: {out}"
    );
    assert!(
        out.contains("Next Steps:"),
        "expected Next Steps: in output: {out}"
    );
    assert!(
        out.contains("Finish src/state.rs"),
        "expected step 1 in output: {out}"
    );
    assert!(
        !out.contains("• Handoff:"),
        "normal Handoff: line should be suppressed: {out}"
    );
}

#[test]
fn prime_resuming_signal_not_consumed_on_second_call() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let today = Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("{today}-session-end.md");
    let content = format!(
        "---\ndate: {today}\nproject: myproject\nphase: implement\n---\n\nDoing work.\n\n## Next Steps\n\n1. Next thing\n"
    );
    write_handoff(tmp.path(), "myproject", &filename, &content);
    write_pending_resume(tmp.path(), "myproject", &format!("handoffs/{filename}"));

    // First call
    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("RESUMING"));

    // Second call — signal must not have been deleted
    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("RESUMING"));
}

#[test]
fn prime_ignores_stale_pending_resume_dated_yesterday() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // Handoff with yesterday's date
    let yesterday = (Local::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let filename = format!("{yesterday}-session-end.md");
    let content = format!(
        "---\ndate: {yesterday}\nproject: myproject\nphase: implement\n---\n\nOld work.\n\n## Next Steps\n\n1. Old step\n"
    );
    write_handoff(tmp.path(), "myproject", &filename, &content);
    write_pending_resume(tmp.path(), "myproject", &format!("handoffs/{filename}"));

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("RESUMING").not())
        .stdout(predicate::str::contains("Handoff:"));
}

#[test]
fn prime_renders_normally_without_pending_resume() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // A normal handoff from yesterday — no .pending-resume
    write_handoff(
        tmp.path(),
        "myproject",
        "2026-02-23-session-end.md",
        "---\ndate: 2026-02-23\nproject: myproject\nphase: research\n---\n\nCompleted research.\n",
    );

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("RESUMING").not())
        .stdout(predicate::str::contains("Handoff: 2026-02-23"));
}

#[test]
fn prime_resuming_empty_next_steps_shows_only_header() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let today = Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("{today}-session-end.md");
    // ## Next Steps section exists but only has HTML comments
    let content = format!(
        "---\ndate: {today}\nproject: myproject\nphase: implement\n---\n\nDoing work.\n\n\
         ## Next Steps\n\n<!-- TODO: fill this in -->\n"
    );
    write_handoff(tmp.path(), "myproject", &filename, &content);
    write_pending_resume(tmp.path(), "myproject", &format!("handoffs/{filename}"));

    let stdout = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let out = String::from_utf8(stdout).unwrap();

    assert!(out.contains("RESUMING"), "expected RESUMING header: {out}");
    assert!(
        !out.contains("Next Steps:"),
        "no Next Steps: label when section is empty: {out}"
    );
}

#[test]
fn prime_close_prime_close_prime_end_to_end_resume_loop() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // First close
    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success();

    let proj_dir = tmp.path().join(".wai/projects/myproject");
    let signal1 = fs::read_to_string(proj_dir.join(".pending-resume")).unwrap();
    assert!(
        !signal1.trim().is_empty(),
        ".pending-resume written after first close"
    );

    // prime should show RESUMING (handoff dated today by wai close)
    let out1 = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        String::from_utf8(out1).unwrap().contains("RESUMING"),
        "first prime after close should show RESUMING"
    );

    // Second close
    wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .assert()
        .success();

    let signal2 = fs::read_to_string(proj_dir.join(".pending-resume")).unwrap();
    assert_ne!(
        signal1.trim(),
        signal2.trim(),
        "second close should overwrite .pending-resume"
    );

    // second prime should show RESUMING pointing to the new handoff
    let out2 = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        String::from_utf8(out2).unwrap().contains("RESUMING"),
        "second prime after close should still show RESUMING"
    );
}

// ─── wai ls ──────────────────────────────────────────────────────────────────

/// Helper: write a minimal .wai/config.toml in a directory so it is detected
/// as a wai workspace.
fn make_workspace(dir: &std::path::Path, ws_name: &str) {
    fs::create_dir_all(dir.join(".wai/projects")).unwrap();
    fs::write(
        dir.join(".wai/config.toml"),
        format!("[project]\nname = \"{}\"\n", ws_name),
    )
    .unwrap();
}

/// Helper: create a project directory with an optional phase state file.
fn make_project(workspace: &std::path::Path, project_name: &str, phase: Option<&str>) {
    let proj_dir = workspace.join(".wai/projects").join(project_name);
    fs::create_dir_all(&proj_dir).unwrap();
    if let Some(p) = phase {
        fs::write(
            proj_dir.join(".state"),
            format!("current: {}\nhistory: []\n", p),
        )
        .unwrap();
    }
}

#[test]
fn ls_single_workspace_one_project() {
    let root = TempDir::new().unwrap();
    let ws = root.path().join("my-repo");
    make_workspace(&ws, "my-ws");
    make_project(&ws, "my-proj", Some("implement"));

    wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("my-proj"))
        .stdout(predicate::str::contains("implement"));
}

#[test]
fn ls_multiple_workspaces_sorted_by_name() {
    let root = TempDir::new().unwrap();

    let ws_a = root.path().join("alpha-repo");
    make_workspace(&ws_a, "alpha");
    make_project(&ws_a, "zebra", Some("plan"));

    let ws_b = root.path().join("beta-repo");
    make_workspace(&ws_b, "beta");
    make_project(&ws_b, "apple", Some("research"));

    let out = wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output = String::from_utf8_lossy(&out);
    // Both projects appear
    assert!(output.contains("apple"), "apple missing: {}", output);
    assert!(output.contains("zebra"), "zebra missing: {}", output);

    // apple (a) appears before zebra (z) when sorted
    let pos_apple = output.find("apple").unwrap();
    let pos_zebra = output.find("zebra").unwrap();
    assert!(
        pos_apple < pos_zebra,
        "projects not sorted: apple at {} vs zebra at {}",
        pos_apple,
        pos_zebra
    );
}

#[test]
fn ls_no_workspaces_prints_message() {
    let root = TempDir::new().unwrap();
    // No workspaces — plain empty directory

    wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("No wai workspaces found"));
}

#[test]
fn ls_no_beads_omits_counts_column() {
    let root = TempDir::new().unwrap();
    let ws = root.path().join("proj");
    make_workspace(&ws, "my-ws");
    make_project(&ws, "my-proj", Some("review"));
    // No .beads/ directory

    wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("my-proj"))
        // No "open" or "ready" in output when no beads
        .stdout(predicate::str::contains("open").not())
        .stdout(predicate::str::contains("ready").not());
}

#[test]
fn ls_beads_present_but_unavailable_is_graceful() {
    // When .beads/ exists but `bd` is not available (or returns no data),
    // the command must still succeed without crashing.
    let root = TempDir::new().unwrap();
    let ws = root.path().join("proj");
    make_workspace(&ws, "my-ws");
    make_project(&ws, "my-proj", Some("implement"));
    // Create .beads/ dir to trigger beads detection
    fs::create_dir_all(ws.join(".beads")).unwrap();

    // Use an empty PATH so 'bd' cannot be found — graceful skip
    wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("PATH", "")
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("my-proj"));
}

#[test]
fn ls_depth_limits_recursion() {
    let root = TempDir::new().unwrap();

    // Workspace at depth 1 (root/shallow-repo)
    let shallow = root.path().join("shallow-repo");
    make_workspace(&shallow, "shallow");
    make_project(&shallow, "shallow-proj", Some("plan"));

    // Workspace at depth 2 (root/dir/deep-repo) — only found with depth >= 2
    let deep = root.path().join("dir").join("deep-repo");
    make_workspace(&deep, "deep");
    make_project(&deep, "deep-proj", Some("research"));

    // With --depth 1, only the depth-1 workspace is found
    wai_cmd(root.path())
        .args([
            "ls",
            "--root",
            root.path().to_str().unwrap(),
            "--depth",
            "1",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("shallow-proj"))
        .stdout(predicate::str::contains("deep-proj").not());

    // With --depth 2, both are found
    wai_cmd(root.path())
        .args([
            "ls",
            "--root",
            root.path().to_str().unwrap(),
            "--depth",
            "2",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("shallow-proj"))
        .stdout(predicate::str::contains("deep-proj"));
}

#[test]
fn ls_invalid_root_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["ls", "--root", "/nonexistent-path-that-does-not-exist"])
        .env("NO_COLOR", "1")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "/nonexistent-path-that-does-not-exist",
        ));
}

// ─── wai reflect ─────────────────────────────────────────────────────────────

/// Helper: initialize a workspace with a git repo and CLAUDE.md.
fn reflect_workspace(dir: &std::path::Path) {
    // Initialize git repo (required for wai init which may check git).
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .output()
        .ok();
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init", "--no-gpg-sign"])
        .current_dir(dir)
        .output()
        .ok();

    init_workspace(dir);
    create_project(dir, "test-proj");
    fs::write(dir.join("CLAUDE.md"), "# Claude\n").unwrap();
}

const MOCK_REFLECT_CONTENT: &str = "\
## Project-Specific AI Context\n\
_Last reflected: 2026-02-24 · 1 sessions analyzed_\n\
\n\
### Conventions\n\
- Use TDD always";

#[test]
fn reflect_with_mock_llm_writes_claude_md_and_reflect_meta() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    // Write a research artifact so context gathering has something.
    write_artifact(tmp.path(), "test-proj", "research", "r.md", "some research");

    wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--yes"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("CLAUDE.md"));

    // CLAUDE.md should contain a WAI:REFLECT block.
    let claude_md = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(claude_md.contains("<!-- WAI:REFLECT:START -->"));
    assert!(claude_md.contains("Use TDD always"));

    // .reflect-meta should be created.
    let meta_path = tmp.path().join(".wai/projects/test-proj/.reflect-meta");
    assert!(meta_path.exists(), ".reflect-meta should be created");
}

#[test]
fn reflect_dry_run_does_not_modify_claude_md() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    let original = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();

    wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--dry-run"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));

    let after = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert_eq!(original, after, "CLAUDE.md must not change in dry-run");
}

#[test]
fn reflect_empty_diff_skips_write() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    // Pre-populate CLAUDE.md with exactly what the mock LLM will produce.
    let existing = format!(
        "# Claude\n\
        <!-- WAI:REFLECT:START -->\n{}\n<!-- WAI:REFLECT:END -->\n",
        MOCK_REFLECT_CONTENT
    );
    fs::write(tmp.path().join("CLAUDE.md"), &existing).unwrap();

    // Target only claude.md so AGENTS.md (also created by init) doesn't interfere.
    wai_cmd(tmp.path())
        .args([
            "reflect",
            "--project",
            "test-proj",
            "--output",
            "claude.md",
            "--yes",
        ])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));

    // CLAUDE.md should be unchanged.
    let after = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert_eq!(existing, after);
}

#[test]
fn reflect_diff_shown_for_existing_reflect_block() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    // Pre-populate with old content.
    let old_block = "<!-- WAI:REFLECT:START -->\n## Old content\n<!-- WAI:REFLECT:END -->\n";
    fs::write(
        tmp.path().join("CLAUDE.md"),
        format!("# Claude\n{}", old_block),
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--dry-run"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));
}

#[test]
fn close_nudge_fires_at_five_plus_handoffs() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    let handoffs_dir = tmp.path().join(".wai/projects/test-proj/handoffs");
    fs::create_dir_all(&handoffs_dir).unwrap();

    // Write 5 handoff files.
    for i in 1..=5 {
        fs::write(
            handoffs_dir.join(format!("2026-02-{:02}-session.md", i)),
            "# Session Handoff\n## What Was Done\nWork.",
        )
        .unwrap();
    }

    // wai close should produce the nudge.
    wai_cmd(tmp.path())
        .args(["close", "--project", "test-proj"])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("sessions since last reflect"));
}

#[test]
fn close_nudge_does_not_fire_for_fewer_than_five_handoffs() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    let handoffs_dir = tmp.path().join(".wai/projects/test-proj/handoffs");
    fs::create_dir_all(&handoffs_dir).unwrap();

    // Write only 3 handoff files — below the threshold.
    for i in 1..=3 {
        fs::write(
            handoffs_dir.join(format!("2026-02-{:02}-session.md", i)),
            "# Session Handoff\n",
        )
        .unwrap();
    }

    let output = wai_cmd(tmp.path())
        .args(["close", "--project", "test-proj"])
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("sessions since last reflect"),
        "nudge should not appear with only 3 handoffs"
    );
}

// ── wai sync --dry-run ────────────────────────────────────────────────────────

#[test]
fn sync_dry_run_does_not_create_files() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Set up a source file to sync.
    let source_dir = tmp.path().join(".wai/resources/agent-config/docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("guide.md"), "# Guide").unwrap();

    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: GUIDE.md\n    strategy: inline\n    sources: [docs]\n",
    );

    wai_cmd(tmp.path())
        .args(["sync", "--dry-run"])
        .assert()
        .success();

    // The target must not have been created.
    assert!(
        !tmp.path().join("GUIDE.md").exists(),
        "dry-run must not create any files"
    );
}

// ─── wai pipeline ─────────────────────────────────────────────────────────────

#[test]
fn pipeline_list_empty() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["pipeline", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No pipelines defined"));
}

#[test]
fn pipeline_add_auto_tags_with_pipeline_run_env() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // Add research with WAI_PIPELINE_RUN set
    wai_cmd(tmp.path())
        .env("WAI_PIPELINE_RUN", "my-pipe-2026-02-25-feature")
        .args(["add", "research", "test content for pipeline tagging"])
        .assert()
        .success();

    // Find the created research file and verify it has the pipeline-run tag
    let research_dir = tmp.path().join(".wai/projects/myproject/research");
    let entries: Vec<_> = fs::read_dir(&research_dir).unwrap().flatten().collect();
    assert_eq!(entries.len(), 1, "Expected exactly one research file");

    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.contains("pipeline-run:my-pipe-2026-02-25-feature"),
        "Expected pipeline-run tag in content: {}",
        content
    );
}

#[test]
fn pipeline_add_merges_user_tags_and_pipeline_run_tag() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .env("WAI_PIPELINE_RUN", "test-run-id")
        .args([
            "add",
            "research",
            "--tags=manual-tag",
            "content with both tags",
        ])
        .assert()
        .success();

    let research_dir = tmp.path().join(".wai/projects/myproject/research");
    let entries: Vec<_> = fs::read_dir(&research_dir).unwrap().flatten().collect();
    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.contains("manual-tag"),
        "Expected manual-tag in: {}",
        content
    );
    assert!(
        content.contains("pipeline-run:test-run-id"),
        "Expected pipeline-run tag in: {}",
        content
    );
}

// ─── wai pipeline init ────────────────────────────────────────────────────────

#[test]
fn pipeline_init_creates_toml_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["pipeline", "init", "my-workflow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-workflow"));

    let toml_path = tmp
        .path()
        .join(".wai/resources/pipelines/my-workflow.toml");
    assert!(toml_path.exists(), "Expected TOML file at {:?}", toml_path);

    let content = fs::read_to_string(&toml_path).unwrap();
    // Check the pipeline name is set correctly
    assert!(
        content.contains("name = \"my-workflow\""),
        "Expected pipeline name in TOML: {}",
        content
    );
    // Check both steps are present
    assert!(
        content.contains("id = \"step-one\""),
        "Expected step-one in TOML: {}",
        content
    );
    assert!(
        content.contains("id = \"step-two\""),
        "Expected step-two in TOML: {}",
        content
    );
    // Check the {topic} placeholder is present (not substituted)
    assert!(
        content.contains("{topic}"),
        "Expected {{topic}} placeholder in TOML: {}",
        content
    );
    // Check the convention comment is present
    assert!(
        content.contains("navigation hints"),
        "Expected convention comment in TOML: {}",
        content
    );
}

#[test]
fn pipeline_init_creates_pipelines_dir_if_absent() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let pipelines_dir = tmp.path().join(".wai/resources/pipelines");
    assert!(!pipelines_dir.exists(), "Pipelines dir should not exist yet");

    wai_cmd(tmp.path())
        .args(["pipeline", "init", "new-pipe"])
        .assert()
        .success();

    assert!(pipelines_dir.exists(), "Pipelines dir should have been created");
}

#[test]
fn pipeline_init_fails_if_file_already_exists() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create the pipeline the first time — should succeed
    wai_cmd(tmp.path())
        .args(["pipeline", "init", "duplicate"])
        .assert()
        .success();

    // Try again — should fail with a clear error
    wai_cmd(tmp.path())
        .args(["pipeline", "init", "duplicate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn pipeline_init_rejects_invalid_name() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["pipeline", "init", "Bad Name!"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid character"));
}

#[test]
fn pipeline_init_template_is_valid_toml() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["pipeline", "init", "valid-check"])
        .assert()
        .success();

    let toml_path = tmp
        .path()
        .join(".wai/resources/pipelines/valid-check.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Verify the generated file parses as valid TOML
    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Generated TOML should be valid, but got error: {:?}",
        parsed.err()
    );
}

// ─── wai pipeline start ───────────────────────────────────────────────────────

/// Helper: write a minimal two-step TOML pipeline for start tests.
fn write_pipeline_toml(dir: &std::path::Path, name: &str) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let content = format!(
        r#"[pipeline]
name = "{name}"
description = "Test pipeline"

[[steps]]
id = "step-one"
prompt = "{{topic}}: do step one research."

[[steps]]
id = "step-two"
prompt = "{{topic}}: do step two implementation."
"#,
        name = name
    );
    fs::write(pipelines_dir.join(format!("{}.toml", name)), content).unwrap();
}

#[test]
fn pipeline_start_creates_run_state() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=auth-refactor"])
        .assert()
        .success();

    // A run state file should exist under .wai/pipeline-runs/
    let runs_dir = tmp.path().join(".wai/pipeline-runs");
    assert!(runs_dir.exists(), ".wai/pipeline-runs/ should be created");

    let entries: Vec<_> = fs::read_dir(&runs_dir)
        .unwrap()
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                == Some("yml")
        })
        .collect();
    assert_eq!(entries.len(), 1, "Exactly one run state file should be written");

    // The run state file name should contain the pipeline name and topic slug
    let run_file = &entries[0].path();
    let stem = run_file.file_stem().unwrap().to_string_lossy().to_string();
    assert!(stem.contains("my-pipe"), "Run ID should contain pipeline name, got: {stem}");
    assert!(stem.contains("auth-refactor"), "Run ID should contain topic slug, got: {stem}");

    // The run state YAML should parse and have correct fields
    let content = fs::read_to_string(run_file).unwrap();
    assert!(content.contains("pipeline: my-pipe"), "Run state should record pipeline name");
    assert!(content.contains("auth-refactor"), "Run state should record topic");
    assert!(content.contains("current_step: 0"), "New run should start at step 0");
}

#[test]
fn pipeline_start_writes_last_run_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=feature-x"])
        .assert()
        .success();

    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    assert!(last_run_path.exists(), ".last-run pointer file should be written");

    let run_id = fs::read_to_string(&last_run_path).unwrap();
    let run_id = run_id.trim();
    assert!(!run_id.is_empty(), ".last-run should contain a non-empty run ID");
    assert!(run_id.starts_with("my-pipe"), "Run ID should start with pipeline name, got: {run_id}");
    assert!(run_id.contains("feature-x"), "Run ID should contain topic slug, got: {run_id}");
}

#[test]
fn pipeline_start_prints_first_step_prompt() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=my-topic"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should contain env export line
    assert!(
        stripped.contains("WAI_PIPELINE_RUN="),
        "Output should contain WAI_PIPELINE_RUN export line, got:\n{stripped}"
    );

    // Should contain step 1/N header
    assert!(
        stripped.contains("step 1/2"),
        "Output should contain 'step 1/2' header, got:\n{stripped}"
    );

    // Should contain the rendered prompt with topic substituted
    assert!(
        stripped.contains("my-topic"),
        "Output should contain topic in rendered prompt, got:\n{stripped}"
    );

    // Should NOT contain the literal {topic} placeholder
    assert!(
        !stripped.contains("{topic}"),
        "Output should not contain literal {{topic}} placeholder, got:\n{stripped}"
    );
}

#[test]
fn pipeline_start_fails_for_unknown_pipeline() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["pipeline", "start", "nonexistent", "--topic=foo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent").or(predicate::str::contains("not found")));
}

// ─── wai pipeline next ────────────────────────────────────────────────────────

#[test]
fn pipeline_next_advances_to_second_step() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run first
    wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=test-topic"])
        .assert()
        .success();

    // Read .last-run to get run ID
    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_id = fs::read_to_string(&last_run_path).unwrap();
    let run_id = run_id.trim().to_string();

    // Advance to next step
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should show step 2/2
    assert!(
        stripped.contains("step 2/2"),
        "Output should contain 'step 2/2', got:\n{stripped}"
    );

    // The run state file should have current_step incremented to 1
    let run_file = tmp.path().join(".wai/pipeline-runs").join(format!("{}.yml", run_id));
    let content = fs::read_to_string(&run_file).unwrap();
    assert!(
        content.contains("current_step: 1"),
        "Run state should show current_step: 1 after advancing, got:\n{content}"
    );
}

#[test]
fn pipeline_next_on_last_step_shows_completion() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run
    wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=test-topic"])
        .assert()
        .success();

    // Advance once (step 1 → step 2)
    wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .assert()
        .success();

    // Advance again (step 2 → completion)
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should show completion block
    assert!(
        stripped.contains("complete"),
        "Output should contain 'complete', got:\n{stripped}"
    );

    // Should contain wai close suggestion
    assert!(
        stripped.contains("wai close"),
        "Output should contain 'wai close' suggestion, got:\n{stripped}"
    );
}

#[test]
fn pipeline_next_errors_when_no_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // No pipeline started, no .last-run file, no WAI_PIPELINE_RUN env
    wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No active pipeline run")
                .or(predicate::str::contains("pipeline start")),
        );
}

#[test]
fn pipeline_next_errors_when_run_already_complete() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run
    wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=test-topic"])
        .assert()
        .success();

    // Advance through all steps (2-step pipeline)
    wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .assert()
        .success();

    // Third call should fail — run is already complete
    wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("already complete")
                .or(predicate::str::contains("complete")),
        );
}

// ─── wai pipeline current ─────────────────────────────────────────────────────

#[test]
fn pipeline_current_reprints_step_prompt() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a pipeline run
    wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=my-topic"])
        .assert()
        .success();

    // Call pipeline current — should reprint step 1 without advancing
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "current"])
        .env_remove("WAI_PIPELINE_RUN")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should contain step 1/2 header
    assert!(
        stripped.contains("step 1/2"),
        "Output should contain 'step 1/2', got:\n{stripped}"
    );

    // Should contain the rendered prompt with topic substituted
    assert!(
        stripped.contains("my-topic"),
        "Output should contain topic in rendered prompt, got:\n{stripped}"
    );

    // Should NOT contain the literal {topic} placeholder
    assert!(
        !stripped.contains("{topic}"),
        "Output should not contain literal {{topic}} placeholder, got:\n{stripped}"
    );

    // Verify state was NOT advanced — current_step should still be 0
    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_id = fs::read_to_string(&last_run_path).unwrap();
    let run_id = run_id.trim().to_string();
    let run_file = tmp.path().join(".wai/pipeline-runs").join(format!("{}.yml", run_id));
    let content = fs::read_to_string(&run_file).unwrap();
    assert!(
        content.contains("current_step: 0"),
        "Run state must NOT have advanced after 'current'; got:\n{content}"
    );
}

#[test]
fn pipeline_current_after_next_shows_step_2() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run
    wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=my-topic"])
        .assert()
        .success();

    // Advance once
    wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .assert()
        .success();

    // Now pipeline current should show step 2
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "current"])
        .env_remove("WAI_PIPELINE_RUN")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("step 2/2"),
        "Output should contain 'step 2/2' after one advance, got:\n{stripped}"
    );
}

#[test]
fn pipeline_current_errors_when_no_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // No pipeline started, no .last-run file, no WAI_PIPELINE_RUN env
    wai_cmd(tmp.path())
        .args(["pipeline", "current"])
        .env_remove("WAI_PIPELINE_RUN")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No active pipeline run")
                .or(predicate::str::contains("pipeline start")),
        );
}

#[test]
fn pipeline_current_on_complete_run_prints_done() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run
    wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=test-topic"])
        .assert()
        .success();

    // Advance through all steps (2-step pipeline)
    wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .assert()
        .success();

    // pipeline current on a completed run should indicate done (not error)
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "current"])
        .env_remove("WAI_PIPELINE_RUN")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("complete") || stripped.contains("done") || stripped.contains("wai close"),
        "Output should indicate the pipeline is complete, got:\n{stripped}"
    );
}

// ─── wai pipeline suggest ─────────────────────────────────────────────────────

/// Helper: write a pipeline TOML with a specific name and description.
fn write_pipeline_toml_with_desc(dir: &std::path::Path, name: &str, description: &str) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let content = format!(
        "[pipeline]\nname = \"{name}\"\ndescription = \"{description}\"\n\n[[steps]]\nid = \"step-one\"\nprompt = \"{{topic}}: do the thing.\"\n",
        name = name,
        description = description
    );
    fs::write(pipelines_dir.join(format!("{}.toml", name)), content).unwrap();
}

#[test]
fn pipeline_suggest_lists_all_pipelines() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_desc(tmp.path(), "auth", "Authentication workflow");
    write_pipeline_toml_with_desc(tmp.path(), "database", "Database migration workflow");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("auth"),
        "Output should contain 'auth' pipeline, got:\n{stripped}"
    );
    assert!(
        stripped.contains("database"),
        "Output should contain 'database' pipeline, got:\n{stripped}"
    );

    // Both pipelines listed, should show start hint
    assert!(
        stripped.contains("wai pipeline start"),
        "Output should contain a start hint, got:\n{stripped}"
    );
}

#[test]
fn pipeline_suggest_with_description_ranks_keyword_match_first() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_desc(tmp.path(), "auth", "Authentication and login workflow");
    write_pipeline_toml_with_desc(tmp.path(), "database", "Database migration and schema changes");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest", "auth login"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // "auth" should appear before "database" in the output
    let auth_pos = stripped.find("auth").expect("'auth' should be in output");
    let db_pos = stripped.find("database").expect("'database' should be in output");
    assert!(
        auth_pos < db_pos,
        "'auth' should appear before 'database' when query matches 'auth login', got:\n{stripped}"
    );

    // The start hint should point to the top result (auth)
    assert!(
        stripped.contains("wai pipeline start auth"),
        "Start hint should point to 'auth', got:\n{stripped}"
    );
}

#[test]
fn pipeline_suggest_no_match_still_shows_all_alphabetically() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_desc(tmp.path(), "zebra", "Zebra workflow");
    write_pipeline_toml_with_desc(tmp.path(), "apple", "Apple workflow");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest", "xyzzy-nomatch-token"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Both should appear
    assert!(stripped.contains("apple"), "Output should contain 'apple', got:\n{stripped}");
    assert!(stripped.contains("zebra"), "Output should contain 'zebra', got:\n{stripped}");

    // Alphabetically "apple" comes before "zebra" when scores are tied at 0
    let apple_pos = stripped.find("apple").unwrap();
    let zebra_pos = stripped.find("zebra").unwrap();
    assert!(
        apple_pos < zebra_pos,
        "'apple' should appear before 'zebra' (alphabetical tie-break), got:\n{stripped}"
    );
}

#[test]
fn pipeline_suggest_no_pipelines_prints_hint() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    // Ensure the pipelines dir exists but is empty (no TOML files)
    fs::create_dir_all(tmp.path().join(".wai/resources/pipelines")).unwrap();

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("No pipelines"),
        "Output should say 'No pipelines', got:\n{stripped}"
    );
    assert!(
        stripped.contains("wai pipeline init"),
        "Output should hint 'wai pipeline init', got:\n{stripped}"
    );
}

#[test]
fn pipeline_suggest_empty_string_treated_as_no_description() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_desc(tmp.path(), "beta", "Beta workflow");
    write_pipeline_toml_with_desc(tmp.path(), "alpha", "Alpha workflow");

    // With empty string — should behave same as no description: alphabetical order
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest", ""])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should show both pipelines
    assert!(stripped.contains("alpha"), "Output should contain 'alpha', got:\n{stripped}");
    assert!(stripped.contains("beta"), "Output should contain 'beta', got:\n{stripped}");

    // Alphabetically: "alpha" before "beta"
    let alpha_pos = stripped.find("alpha").unwrap();
    let beta_pos = stripped.find("beta").unwrap();
    assert!(
        alpha_pos < beta_pos,
        "'alpha' should appear before 'beta' (alphabetical, empty string = no scoring), got:\n{stripped}"
    );
}

// ─── wai status: pipeline integration ────────────────────────────────────────

/// Helper: write a run state YAML file directly, simulating a started pipeline run.
fn write_pipeline_run(
    dir: &std::path::Path,
    pipeline_name: &str,
    run_id: &str,
    current_step: usize,
) {
    let runs_dir = dir.join(".wai/pipeline-runs");
    fs::create_dir_all(&runs_dir).unwrap();
    let yaml = format!(
        "run_id: {run_id}\npipeline: {pipeline_name}\ntopic: test-topic\ncreated_at: \"2026-01-01T00:00:00Z\"\ncurrent_step: {current_step}\n"
    );
    fs::write(runs_dir.join(format!("{run_id}.yml")), yaml).unwrap();

    // Write .last-run pointer
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(pipelines_dir.join(".last-run"), run_id).unwrap();
}

#[test]
fn status_shows_pipeline_active_when_run_exists() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Create the pipeline TOML definition
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Simulate a started run at step 0 (step 1/2)
    write_pipeline_run(tmp.path(), "my-pipe", "my-pipe-2026-01-01-test-topic", 0);

    let output = wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("PIPELINE ACTIVE"),
        "Output should contain 'PIPELINE ACTIVE', got:\n{stripped}"
    );
    assert!(
        stripped.contains("my-pipe"),
        "Output should contain pipeline name 'my-pipe', got:\n{stripped}"
    );
    assert!(
        stripped.contains("step 1/"),
        "Output should contain 'step 1/', got:\n{stripped}"
    );
}

#[test]
fn status_shows_pipeline_current_suggestion_when_active() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_pipeline_toml(tmp.path(), "my-pipe");
    write_pipeline_run(tmp.path(), "my-pipe", "my-pipe-2026-01-01-test-topic", 0);

    let output = wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("wai pipeline current"),
        "Output should suggest 'wai pipeline current', got:\n{stripped}"
    );
}

#[test]
fn status_shows_available_pipelines_when_no_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Create a pipeline TOML but no active run
    write_pipeline_toml(tmp.path(), "my-pipe");

    let output = wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("Available pipelines"),
        "Output should contain 'Available pipelines', got:\n{stripped}"
    );
    assert!(
        stripped.contains("my-pipe"),
        "Output should contain pipeline name 'my-pipe', got:\n{stripped}"
    );
}

#[test]
fn status_shows_pipeline_suggest_when_pipelines_exist() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Create a pipeline TOML but no active run
    write_pipeline_toml(tmp.path(), "my-pipe");

    let output = wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("wai pipeline suggest"),
        "Output should suggest 'wai pipeline suggest', got:\n{stripped}"
    );
}

#[test]
fn status_ignores_stale_last_run_pointer() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Create a pipeline TOML
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Write .last-run pointing to a nonexistent run file (stale pointer)
    let pipelines_dir = tmp.path().join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(pipelines_dir.join(".last-run"), "nonexistent-run-id").unwrap();

    // Should NOT crash and should show "Available pipelines" instead
    let output = wai_cmd(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        !stripped.contains("PIPELINE ACTIVE"),
        "Should not show PIPELINE ACTIVE for stale pointer, got:\n{stripped}"
    );
    assert!(
        stripped.contains("Available pipelines"),
        "Should show Available pipelines when pointer is stale, got:\n{stripped}"
    );
}
