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

// ── fresh workspace initialization ───────────────────────────────────────────

#[test]
fn init_creates_wai_directory_structure() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["init", "--name", "my-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(tmp.path().join(".wai/projects").is_dir());
    assert!(tmp.path().join(".wai/areas").is_dir());
    assert!(tmp.path().join(".wai/resources").is_dir());
    assert!(tmp.path().join(".wai/archives").is_dir());

    let config = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert!(
        config.contains("my-ws"),
        "config.toml should contain workspace name"
    );
}

// ── re-init behavior ──────────────────────────────────────────────────────────

#[test]
fn init_reinit_warns_already_initialized() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["init", "--name", "my-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["init", "--name", "my-ws"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("already initialized"));
}

// ── corrupt config on re-init (evallerina-00n) ────────────────────────────────

/// Re-init must not hand a hint-blind agent a green envelope when
/// .wai/config.toml exists but is unparseable: init fails with an error
/// envelope whose remediation points at `wai doctor` (repair is doctor's
/// channel; init never silently overwrites the config).
#[test]
fn init_reinit_with_corrupt_config_fails_pointing_at_doctor() {
    let tmp = TempDir::new().unwrap();
    wai_cmd(tmp.path())
        .args(["init", "--name", "my-ws"])
        .assert()
        .success();
    fs::write(tmp.path().join(".wai/config.toml"), "not valid toml = [[[").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["--json", "init", "--name", "my-ws"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let payload: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("error envelope is JSON");

    assert_eq!(payload["ok"], false, "corrupt config must not be ok:true");
    assert!(!out.status.success(), "error envelope must exit nonzero");
    let rendered = serde_json::to_string(&payload).unwrap();
    assert!(
        rendered.contains("wai doctor"),
        "remediation should point at `wai doctor`, got: {rendered}"
    );
    // The corrupt file is untouched — no silent overwrite.
    let config = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert_eq!(config, "not valid toml = [[[", "init must not overwrite");
}

// ── non-default flag: --json ──────────────────────────────────────────────────

#[test]
fn init_json_flag_emits_structured_output() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["--json", "init", "--name", "my-ws", "--yes"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&out);
    let payload: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

    // Wrapped in genesis envelope
    assert_eq!(payload["ok"], true, "envelope should have ok=true");
    assert_eq!(
        payload["data"]["already_initialized"], false,
        "fresh init should report already_initialized: false"
    );
    assert_eq!(
        payload["data"]["project_name"], "my-ws",
        "JSON should include the workspace name"
    );
}
