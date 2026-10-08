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

// ── project summary and suggestions ──────────────────────────────────────────

#[test]
fn status_shows_workspace_when_no_projects() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Workspace:"));
}

#[test]
fn status_shows_project_name_in_header() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Project:"));
    assert!(!(stderr.contains("test-ws")));
}

#[test]
fn status_shows_project_name_in_summary() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-app"));
}

// ── JSON output ───────────────────────────────────────────────────────────────

#[test]
fn status_json_flag_emits_suggestions_field() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["status", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"suggestions\""));
}

// ── empty-state path ──────────────────────────────────────────────────────────

#[test]
fn status_no_projects_succeeds_with_empty_workspace() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

// ── doctor health summary (wai-j56n) ─────────────────────────────────────────

use std::fs;

/// Write a pipeline TOML WITHOUT a `[pipeline.metadata]` section. This triggers
/// a `wai doctor` warning ("Missing [pipeline.metadata]") — used to force a
/// non-clean health summary in status tests.
fn write_pipeline_no_metadata(dir: &std::path::Path, name: &str) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let toml = format!(
        "[pipeline]\nname = \"{name}\"\ndescription = \"no metadata\"\n[[steps]]\nid = \"one\"\nprompt = \"do {{topic}}\"\n"
    );
    fs::write(pipelines_dir.join(format!("{name}.toml")), toml).unwrap();
}

#[test]
fn status_surfaces_doctor_warning_when_not_clean() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    // A pipeline without [pipeline.metadata] triggers a doctor Warn.
    write_pipeline_no_metadata(tmp.path(), "orphan");

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("warning"));
    assert!(stdout.contains("wai doctor"));
}

#[test]
fn status_silent_on_health_when_doctor_is_clean() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("wai doctor")));
}
