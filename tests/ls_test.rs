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

fn make_workspace(dir: &std::path::Path, name: &str) {
    fs::create_dir_all(dir.join(".wai/projects")).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(
        dir.join(".wai/config.toml"),
        format!("[project]\nname = \"{}\"\n", name),
    )
    .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
}

fn make_project(workspace: &std::path::Path, project_name: &str, phase: &str) {
    let proj_dir = workspace.join(".wai/projects").join(project_name);
    fs::create_dir_all(&proj_dir).unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    fs::write(
        proj_dir.join(".state"),
        format!("current: {}\nhistory: []\n", phase),
    )
    .unwrap_or_else(|e| panic!("unwrap failed: {e}"));
}

// ── workspace discovery ───────────────────────────────────────────────────────

#[test]
fn ls_discovers_nested_workspace_and_project() {
    let root = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));
    let ws = root.path().join("my-repo");
    make_workspace(&ws, "my-ws");
    make_project(&ws, "my-proj", "implement");

    let out = wai_cmd(root.path())
        .args([
            "ls",
            "--root",
            root.path()
                .to_str()
                .unwrap_or_else(|| panic!("unwrap failed")),
        ])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-proj"));
    assert!(stdout.contains("implement"));
}

// ── flag path: --depth ────────────────────────────────────────────────────────

#[test]
fn ls_depth_flag_limits_recursion() {
    let root = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let shallow = root.path().join("shallow-repo");
    make_workspace(&shallow, "shallow");
    make_project(&shallow, "shallow-proj", "plan");

    let deep = root.path().join("dir").join("deep-repo");
    make_workspace(&deep, "deep");
    make_project(&deep, "deep-proj", "research");

    let out = wai_cmd(root.path())
        .args([
            "ls",
            "--root",
            root.path()
                .to_str()
                .unwrap_or_else(|| panic!("unwrap failed")),
            "--depth",
            "1",
        ])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("shallow-proj"));
    assert!(!(stdout.contains("deep-proj")));
}

// ── no-results path ───────────────────────────────────────────────────────────

#[test]
fn ls_empty_root_reports_no_workspaces() {
    let root = TempDir::new().unwrap_or_else(|e| panic!("unwrap failed: {e}"));

    let out = wai_cmd(root.path())
        .args([
            "ls",
            "--root",
            root.path()
                .to_str()
                .unwrap_or_else(|| panic!("unwrap failed")),
        ])
        .output()
        .unwrap_or_else(|e| panic!("command should run: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No wai workspaces found"));
}
