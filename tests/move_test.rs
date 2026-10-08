use assert_cmd::Command;
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

// ── move between PARA categories ──────────────────────────────────────────────

#[test]
fn move_project_to_archives() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "old-proj");

    let out = wai_cmd(tmp.path())
        .args(["move", "old-proj", "archives"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Moved"));

    assert!(
        !tmp.path().join(".wai/projects/old-proj").exists(),
        "project dir should no longer exist in projects/"
    );
    assert!(
        tmp.path().join(".wai/archives/old-proj").exists(),
        "project dir should exist in archives/"
    );
}

#[test]
fn move_project_to_areas() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");

    let out = wai_cmd(tmp.path())
        .args(["move", "my-proj", "areas"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Moved"));

    assert!(
        !tmp.path().join(".wai/projects/my-proj").exists(),
        "project dir should no longer exist in projects/"
    );
    assert!(
        tmp.path().join(".wai/areas/my-proj").exists(),
        "project dir should exist in areas/"
    );
}

// ── failure: unknown item ─────────────────────────────────────────────────────

#[test]
fn move_nonexistent_item_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["move", "ghost", "archives"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}
