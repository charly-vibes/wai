use assert_cmd::Command;
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

// ── PARA overview ─────────────────────────────────────────────────────────────

#[test]
fn show_overview_lists_para_categories() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["show"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Projects"));
    assert!(stdout.contains("my-app"));
    assert!(stdout.contains("Areas"));
    assert!(stdout.contains("Resources"));
}

// ── specific item detail ──────────────────────────────────────────────────────

#[test]
fn show_specific_project_displays_details() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["show", "my-app"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-app"));
    assert!(stdout.contains("research"));
    assert!(stdout.contains("plans"));
}

// ── failure: missing item ─────────────────────────────────────────────────────

#[test]
fn show_nonexistent_item_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["show", "nonexistent"])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}
