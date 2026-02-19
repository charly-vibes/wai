use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

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
    assert!(tmp
        .path()
        .join(".wai/resources/agent-config/skills")
        .is_dir());
    assert!(tmp
        .path()
        .join(".wai/resources/agent-config/rules")
        .is_dir());
    assert!(tmp
        .path()
        .join(".wai/resources/agent-config/context")
        .is_dir());
    assert!(tmp
        .path()
        .join(".wai/resources/agent-config/.projections.yml")
        .is_file());

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

    assert!(tmp
        .path()
        .join(".wai/areas/dev-standards/research")
        .is_dir());
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
        .stdout(predicate::str::contains("2+ matches"));
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
                .and(predicate::str::contains("mkdir")),
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
