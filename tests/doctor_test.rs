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

// ── healthy workspace ────────────────────────────────────────────────────────

#[test]
fn doctor_healthy_workspace_reports_zero_failures() {
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

// ── broken workspace detection ───────────────────────────────────────────────

#[test]
fn doctor_missing_required_directory_reports_fail() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .assert()
        .code(1)
        .stdout(
            predicate::str::contains("\"status\": \"fail\"")
                .and(predicate::str::contains("archives")),
        );
}

#[test]
fn doctor_invalid_config_toml_reports_fail() {
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
fn doctor_corrupted_project_state_reports_fail() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "broken");

    fs::write(
        tmp.path().join(".wai/projects/broken/.state"),
        "{{not valid yaml",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .assert()
        .code(1)
        .stdout(
            predicate::str::contains("\"status\": \"fail\"")
                .and(predicate::str::contains("Project state: broken")),
        );
}

// ── fix paths (non-interactive) ──────────────────────────────────────────────

#[test]
fn doctor_fix_with_yes_repairs_missing_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Fixed"));

    assert!(tmp.path().join(".wai/archives").is_dir());
}

#[test]
fn doctor_fix_with_safe_flag_refuses_to_apply_fixes() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--safe"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("apply doctor fixes").or(predicate::str::contains("--safe")),
        );
}

// ── pi session hook (wai-hfgz) ───────────────────────────────────────────────

#[test]
fn doctor_pi_hook_omitted_when_no_pi_dir() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // No `.pi/` → the pi session hook check must not appear at all.
    wai_cmd(tmp.path())
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pi session hook").not());
}

#[test]
fn doctor_pi_hook_warns_when_pi_present_without_wai_extension() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    fs::create_dir_all(tmp.path().join(".pi/extensions")).unwrap();
    fs::write(
        tmp.path().join(".pi/extensions/other.ts"),
        "pi.on(\"session_start\", async (_e, ctx) => { ctx.ui.notify(\"hi\"); });\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pi session hook"))
        .stdout(predicate::str::contains("session_start"));
}

#[test]
fn doctor_pi_hook_passes_when_wai_prime_extension_present() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    fs::create_dir_all(tmp.path().join(".pi/extensions")).unwrap();
    fs::write(
        tmp.path().join(".pi/extensions/wai-prime.ts"),
        "pi.on(\"session_start\", async () => { await pi.exec(\"wai\", [\"prime\"]); });\n",
    )
    .unwrap();

    wai_cmd(tmp.path())
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pi session hook"))
        .stdout(predicate::str::contains("wai prime"));
}
