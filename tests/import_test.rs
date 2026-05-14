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

#[test]
fn import_single_rule_file_copies_into_rules_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let source = tmp.path().join(".cursorrules");
    fs::write(&source, "be explicit").unwrap();

    wai_cmd(tmp.path())
        .args(["import", source.to_str().unwrap()])
        .assert()
        .success();

    let imported = tmp
        .path()
        .join(".wai/resources/agent-config/rules/.cursorrules");
    assert!(imported.is_file());
    assert_eq!(fs::read_to_string(imported).unwrap(), "be explicit");
}

#[test]
fn import_directory_routes_files_by_category() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let source = tmp.path().join("external-config");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("team-rule.md"), "rule").unwrap();
    fs::write(source.join("review-command.md"), "command").unwrap();
    fs::write(source.join("notes.md"), "context").unwrap();

    wai_cmd(tmp.path())
        .args(["import", source.to_str().unwrap()])
        .assert()
        .success();

    let agent_config = tmp.path().join(".wai/resources/agent-config");
    assert!(agent_config.join("rules/team-rule.md").is_file());
    assert!(agent_config.join("skills/review-command.md").is_file());
    assert!(agent_config.join("context/notes.md").is_file());
}

#[test]
fn repeated_import_overwrites_existing_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let source = tmp.path().join("notes.md");
    fs::write(&source, "first version").unwrap();

    wai_cmd(tmp.path())
        .args(["import", source.to_str().unwrap()])
        .assert()
        .success();

    fs::write(&source, "second version").unwrap();

    wai_cmd(tmp.path())
        .args(["import", source.to_str().unwrap()])
        .assert()
        .success();

    let imported = tmp
        .path()
        .join(".wai/resources/agent-config/context/notes.md");
    assert_eq!(fs::read_to_string(imported).unwrap(), "second version");
}

#[test]
fn import_missing_path_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["import", "missing-path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Path not found"));
}
