use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd
}

fn init_workspace(dir: &std::path::Path) {
    wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();
}

fn create_project(dir: &std::path::Path, name: &str) {
    wai_cmd(dir)
        .args(["new", "project", name])
        .assert()
        .success();
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
    fs::write(path, content).unwrap();
}

// ── plain-text query ──────────────────────────────────────────────────────────

#[test]
fn search_finds_content_by_plain_query() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "# Notes\nThe REST API uses GraphQL.\n",
    );

    wai_cmd(tmp.path())
        .args(["search", "GraphQL"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search results"))
        .stdout(predicate::str::contains("GraphQL"));
}

// ── type filter ───────────────────────────────────────────────────────────────

#[test]
fn search_type_filter_limits_to_matching_artifact_type() {
    let tmp = TempDir::new().unwrap();
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

    wai_cmd(tmp.path())
        .args(["search", "keyword_hit", "--type", "research"])
        .assert()
        .success()
        .stdout(predicate::str::contains("research/"))
        .stdout(predicate::str::contains("plans/").not());
}

// ── invalid regex / no-results ────────────────────────────────────────────────

#[test]
fn search_no_results_reports_empty_message() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["search", "zzz_no_match_zzz"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results found"));
}

#[test]
fn search_invalid_regex_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    wai_cmd(tmp.path())
        .args(["search", "[invalid", "--regex"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid regex"));
}
