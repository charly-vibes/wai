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
fn add_research_inline_content_creates_dated_artifact() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["add", "research", "inline findings", "--project", "my-app"])
        .assert()
        .success();

    let research_dir = tmp.path().join(".wai/projects/my-app/research");
    let files: Vec<_> = fs::read_dir(&research_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(files.len(), 1);
    let name = files[0].file_name().into_string().unwrap();
    assert_eq!(&name[4..5], "-");
    assert_eq!(&name[7..8], "-");

    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.contains("inline findings"));
}

#[test]
fn add_research_file_input_with_explicit_project_and_tags() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let source = tmp.path().join("notes.md");
    fs::write(&source, "file-backed research").unwrap();

    wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "--project",
            "beta",
            "--file",
            source.to_str().unwrap(),
            "--tags",
            "api,design",
        ])
        .assert()
        .success();

    let beta_research = tmp.path().join(".wai/projects/beta/research");
    let beta_files: Vec<_> = fs::read_dir(&beta_research)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(beta_files.len(), 1);

    let content = fs::read_to_string(beta_files[0].path()).unwrap();
    assert!(content.contains("file-backed research"));
    assert!(content.contains("tags: [api, design]"));

    let alpha_research = tmp.path().join(".wai/projects/alpha/research");
    let alpha_files: Vec<_> = fs::read_dir(&alpha_research)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(alpha_files.is_empty(), "artifact should not land in alpha");
}

#[test]
fn add_research_fails_outside_workspace() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["add", "research", "notes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No project initialized"));
}
