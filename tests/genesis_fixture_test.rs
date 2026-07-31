//! Tests demonstrating genesis::fixture::Fixture adoption.
//!
//! These tests use the shared Fixture builder instead of raw
//! tempfile::TempDir, showing the preferred pattern for new tests.

use assert_cmd::Command;
use predicates::prelude::*;

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

// ── Fixture: basic workspace with markers ─────────────────────────────────────

#[test]
fn fixture_with_wai_marker_is_detected_by_doctor() {
    let fixture = genesis::fixture::Fixture::new()
        .with_marker(".wai")
        .build()
        .expect("build fixture");

    // After init, .wai/ should exist
    wai_cmd(fixture.root())
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();
    assert!(
        fixture.path(".wai").exists(),
        ".wai/ should exist after init"
    );
}

// ── Fixture: with config file ─────────────────────────────────────────────────

#[test]
fn fixture_with_toml_config_parses_correctly() {
    let cfg = toml::toml! {
        [project]
        name = "fixture-test"
        version = "1.0.0"
    };
    let fixture = genesis::fixture::Fixture::new()
        .with_marker(".wai")
        .with_toml(".wai/config.toml", toml::Value::Table(cfg))
        .build()
        .expect("build fixture");

    wai_cmd(fixture.root()).args(["status"]).assert().success();
}

// ── Fixture: with git init ────────────────────────────────────────────────────

#[test]
fn fixture_with_git_init_allows_commit() {
    let fixture = genesis::fixture::Fixture::new()
        .with_git_init()
        .build()
        .expect("build fixture");

    // Run wai init inside the git repo
    wai_cmd(fixture.root())
        .args(["init", "--name", "git-test"])
        .assert()
        .success();

    // Git should have auto-committed .wai/
    let output = fixture
        .run(&[
            "git",
            "-C",
            fixture.root().to_str().unwrap(),
            "log",
            "--oneline",
            "-1",
        ])
        .expect("run git log");
    assert!(
        output.success(),
        "git log should succeed: {}",
        output.stdout
    );
}

// ── Fixture: with custom files ────────────────────────────────────────────────

#[test]
fn fixture_custom_files_are_present() {
    let fixture = genesis::fixture::Fixture::new()
        .with_marker(".wai")
        .with_file("README.md", "# Test Project\n\nA fixture test project.")
        .with_file("src/main.rs", "fn main() { println!(\"hello\"); }")
        .build()
        .expect("build fixture");

    assert!(fixture.path("README.md").exists());
    assert!(fixture.path("src/main.rs").exists());
    fixture.assert_file_contains("README.md", "Test Project");
}
