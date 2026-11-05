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

// ── phase show ────────────────────────────────────────────────────────────────

#[test]
fn phase_show_displays_current_phase() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["phase", "show"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("research"));
}

// ── phase set (explicit advancement) ─────────────────────────────────────────

#[test]
fn phase_set_jumps_to_target_phase() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("implement"));

    let state = fs::read_to_string(tmp.path().join(".wai/projects/my-app/.state"))
        .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    assert!(
        state.contains("current: implement"),
        "state file should reflect the new phase"
    );
}

// ── failure: invalid phase ────────────────────────────────────────────────────

#[test]
fn phase_set_invalid_phase_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["phase", "set", "banana"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("Unknown phase"));
}
