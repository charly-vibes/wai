use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    // Update notice off in tests: existing assertions expect quiet stderr and
    // must not depend on real ~/.cache state (wai-r3p0 tidy).
    cmd.env("GENESIS_NO_UPDATE_CHECK", "1");
    cmd
}

#[test]
fn way_minimal_repo_reports_partial_adoption() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test Project").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stdout.contains("ℹ"));
    assert!(stderr.contains("best practices adopted"));
}

#[test]
fn way_json_output_includes_checks_and_summary() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"checks\""));
    assert!(stdout.contains("\"summary\""));
    assert!(stdout.contains("\"recommendations\""));
}

#[test]
fn way_partial_repo_emits_fix_suggestions() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("→"));
    assert!(stdout.contains("https://"));
}

#[test]
fn way_pretender_check_recommends_when_no_config() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("pretender"));
    assert!(stdout.contains("No pretender.toml"));
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

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("pretender"));
    assert!(stdout.contains("pretender.toml detected"));
}

#[test]
fn way_code_quality_topic_prints_guide() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "code-quality"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Code Quality Guide"));
    assert!(stdout.contains("pretender"));
}
