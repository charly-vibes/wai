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

fn init_workspace(dir: &std::path::Path) {
    let out = wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

fn create_project(dir: &std::path::Path, name: &str) {
    let out = wai_cmd(dir)
        .args(["new", "project", name])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

// ── healthy workspace ────────────────────────────────────────────────────────

#[test]
fn doctor_healthy_workspace_reports_zero_failures() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"fail\": 0"));
    assert!(stdout.contains("\"summary\""));
    assert!(stdout.contains("\"pass\""));
}

// ── broken workspace detection ───────────────────────────────────────────────

#[test]
fn doctor_missing_required_directory_reports_fail() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout.contains("\"status\": \"fail\""));
    assert!(stdout.contains("archives"));
}

#[test]
fn doctor_invalid_config_toml_reports_fail() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::write(tmp.path().join(".wai/config.toml"), "{{invalid toml!!!").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout.contains("\"status\": \"fail\""));
    assert!(stdout.contains("Configuration"));
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

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout.contains("\"status\": \"fail\""));
    assert!(stdout.contains("project-state"));
}

// ── fix paths (non-interactive) ──────────────────────────────────────────────

#[test]
fn doctor_fix_with_yes_repairs_missing_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--yes"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Fixed"));

    assert!(tmp.path().join(".wai/archives").is_dir());
}

#[test]
fn doctor_fix_with_safe_flag_refuses_to_apply_fixes() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--fix", "--safe"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("apply doctor fixes") || stderr.contains("--safe")));
}

// ── pi session hook (wai-hfgz) ───────────────────────────────────────────────

#[test]
fn doctor_pi_hook_omitted_when_no_pi_dir() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // No `.pi/` → the pi session hook check must not appear at all.
    let out = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("Pi session hook")));
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

    let out = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pi session hook"));
    assert!(stdout.contains("session_start"));
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

    let out = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pi session hook"));
    assert!(stdout.contains("wai prime"));
}
