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

    // Second init should warn, not fail (cliclack outputs to stderr)
    wai_cmd(tmp.path())
        .args(["init", "--name", "test-ws"])
        .assert()
        .success()
        .stderr(predicate::str::contains("already initialized"));
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

// ─── wai (no args) ─────────────────────────────────────────────────────────

#[test]
fn no_args_shows_welcome() {
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

// ─── error cases ────────────────────────────────────────────────────────────

#[test]
fn commands_fail_without_init() {
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
