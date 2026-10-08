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

fn write_projections_yml(dir: &std::path::Path, content: &str) {
    let path = dir.join(".wai/resources/agent-config/.projections.yml");
    fs::write(path, content).unwrap();
}

// ── successful sync ───────────────────────────────────────────────────────────

#[test]
fn sync_projects_inline_source_to_target_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let source_dir = tmp.path().join(".wai/resources/agent-config/docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("guide.md"), "# Guide").unwrap();

    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: GUIDE.md\n    strategy: inline\n    sources: [docs]\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["sync"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        tmp.path().join("GUIDE.md").exists(),
        "sync should create the projected target file"
    );
}

// ── dry-run: no files written ─────────────────────────────────────────────────

#[test]
fn sync_dry_run_does_not_create_files() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let source_dir = tmp.path().join(".wai/resources/agent-config/docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("guide.md"), "# Guide").unwrap();

    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: GUIDE.md\n    strategy: inline\n    sources: [docs]\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["sync", "--dry-run"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        !tmp.path().join("GUIDE.md").exists(),
        "dry-run must not create any files"
    );
}

// ── empty workspace: sync is a no-op ─────────────────────────────────────────

#[test]
fn sync_empty_workspace_succeeds_without_projections() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["sync"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}
