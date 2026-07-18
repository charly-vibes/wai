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

#[test]
fn way_minimal_repo_reports_partial_adoption() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test Project").unwrap();

    wai_cmd(tmp.path())
        .args(["way"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ℹ"))
        .stderr(predicate::str::contains("best practices adopted"));
}

#[test]
fn way_json_output_includes_checks_and_summary() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"checks\"")
                .and(predicate::str::contains("\"summary\""))
                .and(predicate::str::contains("\"recommendations\"")),
        );
}

#[test]
fn way_partial_repo_emits_fix_suggestions() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    wai_cmd(tmp.path())
        .args(["way"])
        .assert()
        .success()
        .stdout(predicate::str::contains("→").and(predicate::str::contains("https://")));
}

#[test]
fn way_pretender_check_recommends_when_no_config() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("pretender")
                .and(predicate::str::contains("No pretender.toml")),
        );
}

#[test]
fn way_pretender_check_passes_when_config_present() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(
        tmp.path().join("pretender.toml"),
        "[pretender]\nmode = \"tiered\"\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["way", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("pretender")
                .and(predicate::str::contains("pretender.toml detected")),
        );
}

#[test]
fn way_code_quality_topic_prints_guide() {
    let tmp = TempDir::new().unwrap();

    wai_cmd(tmp.path())
        .args(["way", "code-quality"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Code Quality Guide")
                .and(predicate::str::contains("pretender")),
        );
}
