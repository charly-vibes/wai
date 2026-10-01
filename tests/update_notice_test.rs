//! Update-notice wiring (genesis-2ex phase 3, wai-r3p0).
//!
//! The binary checks crates.io for a newer `wai-cli` release via
//! `genesis::update_check` (feature `update-check`) and prints a single-line
//! notice AFTER command output on success. These tests are hermetic: the
//! crates.io response never leaves the process because the cache is seeded
//! fresh (TTL not expired → zero HTTP calls by design contract).

use assert_cmd::Command;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Seed a fresh update-check cache saying `9999.1.1` is available.
fn seed_fresh_cache(cache_root: &std::path::Path) {
    let dir = cache_root.join("genesis").join("update-check");
    fs::create_dir_all(&dir).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let entry = format!(
        r#"{{"checked_at":{now},"latest":"9999.1.1","published_at":null,"ttl_secs":604800}}"#
    );
    // Seed BOTH cache roots: genesis-39r (XDG_CACHE_HOME support in
    // cache_path) is unreleased — pinned genesis-vibes 0.10.0 from crates.io
    // always reads $HOME/.cache, while 0.10.1+ will read XDG_CACHE_HOME.
    // Seeding both keeps this hermetic across the pin bump. Drop the
    // HOME/.cache copy once genesis-vibes >= 0.10.1 ships the fix.
    let home_dir = cache_root
        .join(".cache")
        .join("genesis")
        .join("update-check");
    fs::create_dir_all(&home_dir).unwrap();
    fs::write(dir.join("wai-cli.json"), &entry).unwrap();
    fs::write(home_dir.join("wai-cli.json"), &entry).unwrap();
}

/// A wai command pointed at a hermetic cache dir, with CI opt-outs cleared.
fn wai_cmd(cache_root: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.env("XDG_CACHE_HOME", cache_root)
        .env("NO_COLOR", "1")
        .env_remove("GENESIS_NO_UPDATE_CHECK")
        .env_remove("CI");
    cmd
}

// ── Acceptance: notice fires after output, exactly one line ────────────

#[test]
fn notice_prints_after_command_output_as_one_line() {
    let tmp = tempfile::TempDir::new().unwrap();
    seed_fresh_cache(tmp.path());

    // `completions` is a cheap, side-effect-free, pure-output command.
    let output = wai_cmd(tmp.path())
        .args(["completions", "bash"])
        .env("HOME", tmp.path()) // isolate real cache dir too
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Notice lives on stderr so piped/JSON stdout stays machine-parseable,
    // and it appears only after the command's own output completed.
    assert!(
        stderr.contains("wai-cli 9999.1.1 available — you have"),
        "expected update notice on stderr, got: {stderr:?}"
    );
    assert!(
        stderr.contains("cargo install wai-cli"),
        "notice must include the fix command, got: {stderr:?}"
    );
    assert!(
        !stdout.contains("9999.1.1"),
        "notice must not pollute stdout: {stdout:?}"
    );
    // Exactly one notice line.
    let notice_lines: Vec<&str> = stderr
        .lines()
        .filter(|l| l.contains("available — you have"))
        .collect();
    assert_eq!(notice_lines.len(), 1, "notice must be a single line");
    // No ANSI escapes when not a TTY.
    assert!(!stderr.contains('\u{1b}'), "no color codes when not a TTY");
}

// ── Acceptance: opt-out env vars suppress the notice ───────────────────

#[test]
fn genesis_no_update_check_suppresses_notice() {
    let tmp = tempfile::TempDir::new().unwrap();
    seed_fresh_cache(tmp.path());

    let output = wai_cmd(tmp.path())
        .args(["completions", "bash"])
        .env("HOME", tmp.path())
        .env("GENESIS_NO_UPDATE_CHECK", "1")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("available — you have"),
        "opt-out must suppress the notice, got: {stderr:?}"
    );
}

#[test]
fn ci_env_suppresses_notice() {
    let tmp = tempfile::TempDir::new().unwrap();
    seed_fresh_cache(tmp.path());

    let output = wai_cmd(tmp.path())
        .args(["completions", "bash"])
        .env("HOME", tmp.path())
        .env("CI", "true")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("available — you have"),
        "CI must suppress the notice, got: {stderr:?}"
    );
}

// ── Acceptance: no update → silent ─────────────────────────────────────

#[test]
fn no_update_available_means_no_notice() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("genesis").join("update-check");
    fs::create_dir_all(&dir).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Cache says installed version IS the latest.
    let entry =
        format!(r#"{{"checked_at":{now},"latest":null,"published_at":null,"ttl_secs":604800}}"#);
    fs::write(dir.join("wai-cli.json"), entry).unwrap();

    let output = wai_cmd(tmp.path())
        .args(["completions", "bash"])
        .env("HOME", tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("available — you have"),
        "up-to-date install must stay silent, got: {stderr:?}"
    );
}
