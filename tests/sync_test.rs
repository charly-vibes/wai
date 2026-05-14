use assert_cmd::Command;
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

    wai_cmd(tmp.path()).args(["sync"]).assert().success();

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

    wai_cmd(tmp.path())
        .args(["sync", "--dry-run"])
        .assert()
        .success();

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

    wai_cmd(tmp.path()).args(["sync"]).assert().success();
}
