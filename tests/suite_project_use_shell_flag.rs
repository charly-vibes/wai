//! Contract suite for `wai project use --shell` explicit override
//! (epic wai-fvhv / ticket wai-sib1; RO5U pass 1 in
//! .wai/projects/qa-round-execution/designs/).
//!
//! Contract:
//! - `--shell fish` must produce fish syntax even when SHELL is a posix shell
//!   (explicit flag beats env fallback).
//! - `--shell posix` must produce posix syntax even when SHELL is fish
//!   (env fallback must not override an explicit flag).
//! - No flag keeps the SHELL-detection fallback: posix default (tested here,
//!   hermetic against FISH_VERSION) and fish detection (covered by
//!   tests/suite_project_use_fish_shell_syntax.rs).

mod common;
use common::*;
use tempfile::TempDir;

#[test]
fn project_use_default_is_posix_export_line() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .env("SHELL", "/bin/bash")
        // Hermetic: a host with fish installed leaks FISH_VERSION through
        // env inheritance, which detect_shell honors (RO5U pass 1, L5).
        .env_remove("FISH_VERSION")
        .args(["project", "use", "myproj"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&out);
    assert!(
        stdout.contains("export WAI_PROJECT='myproj'"),
        "expected posix default export line in: {}",
        stdout
    );
}

#[test]
fn project_use_shell_flag_fish_overrides_posix_default() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .env("SHELL", "/bin/bash")
        .args(["project", "use", "myproj", "--shell", "fish"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&out);
    assert!(
        stdout.contains("set -gx WAI_PROJECT 'myproj'"),
        "expected fish syntax from --shell fish with posix SHELL in: {}",
        stdout
    );
}

#[test]
fn project_use_shell_flag_posix_overrides_fish_shell_env() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .env("SHELL", "/usr/bin/fish")
        .args(["project", "use", "myproj", "--shell", "posix"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&out);
    assert!(
        stdout.contains("export WAI_PROJECT='myproj'"),
        "expected posix syntax from --shell posix with fish SHELL in: {}",
        stdout
    );
}

#[test]
fn project_use_shell_flag_rejects_unknown_shell() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .env("SHELL", "/bin/bash")
        .args(["project", "use", "myproj", "--shell", "nushell"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "unknown shell must fail the command");
    assert!(
        stderr.contains("Unknown shell"),
        "expected 'Unknown shell' guidance in stderr: {}",
        stderr
    );
}
