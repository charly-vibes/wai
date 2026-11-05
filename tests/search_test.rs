use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    // Update notice off in tests: existing assertions expect quiet stderr and
    // must not depend on real ~/.cache state (wai-r3p0 tidy).
    cmd.env("GENESIS_NO_UPDATE_CHECK", "1");
    // Isolate HOME: global ~/.wai/resources is now part of the search scope
    // (wai-gkk3) and must not leak real-user content into assertions.
    cmd.env("HOME", dir);
    cmd
}

fn init_workspace(dir: &std::path::Path) {
    let out = wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    assert!(out.status.success());
}

fn create_project(dir: &std::path::Path, name: &str) {
    let out = wai_cmd(dir)
        .args(["new", "project", name])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    assert!(out.status.success());
}

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
    fs::write(path, content).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
}

// ── plain-text query ──────────────────────────────────────────────────────────

#[test]
fn search_finds_content_by_plain_query() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "# Notes\nThe REST API uses GraphQL.\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "GraphQL"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Search results"));
    assert!(stdout.contains("GraphQL"));
}

// ── type filter ───────────────────────────────────────────────────────────────

#[test]
fn search_type_filter_limits_to_matching_artifact_type() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "keyword_hit in research\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "plans",
        "2026-01-15-plan.md",
        "keyword_hit in plans\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "keyword_hit", "--type", "research"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("research/"));
    assert!(!(stdout.contains("plans/")));
}

// ── invalid regex / no-results ────────────────────────────────────────────────

#[test]
fn search_no_results_reports_empty_message() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["search", "zzz_no_match_zzz"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No results found"));
}

#[test]
fn search_invalid_regex_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["search", "[invalid", "--regex"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("Invalid regex"));
}

// ── global resources scope (wai-gkk3) ─────────────────────────────────────────

/// Content in ~/.wai/resources is discoverable from any repo workspace,
/// with hits tagged by their global path (e.g. global:patterns/x.md).
#[test]
fn search_returns_global_resource_hits_tagged() {
    let ws = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    let home = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(ws.path());
    create_project(ws.path(), "my-app");
    let patterns = home.path().join(".wai/resources/patterns");
    fs::create_dir_all(&patterns).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(
        patterns.join("orchestrator-subagents.md"),
        "# Orchestrator\n\nthe pokeable canon lives here\n",
    )
    .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(ws.path())
        .env("HOME", home.path())
        .args(["search", "pokeable"])
        .output()
        .unwrap_or_else(|e| panic!("search should succeed: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("global:patterns/orchestrator-subagents.md"));
}

/// --in scopes the search to a project; global resources are not included.
#[test]
fn search_with_project_scope_excludes_global_resources() {
    let ws = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    let home = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(ws.path());
    create_project(ws.path(), "my-app");
    let patterns = home.path().join(".wai/resources/patterns");
    fs::create_dir_all(&patterns).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(
        patterns.join("orchestrator-subagents.md"),
        "pokeable canon\n",
    )
    .unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(ws.path())
        .env("HOME", home.path())
        .args(["search", "pokeable", "--in", "my-app"])
        .output()
        .unwrap_or_else(|e| panic!("search should succeed: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No results found"));
}
