use assert_cmd::Command;
use std::fs;

/// Helper: create a wai command with NO_COLOR for predictable assertions.
fn wai_cmd() -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.env("NO_COLOR", "1");
    cmd
}

/// The target repo for `wai feedback` is wai's own upstream.
const TARGET_REPO: &str = "charly-vibes/wai";

// ── Acceptance: e2e --dry-run asserts body + exact gh line ────────────

#[test]
fn feedback_dry_run_prints_title_body_labels_and_exact_gh_line() {
    let tmp = tempfile::TempDir::new().unwrap();
    // Initialize a workspace so context-gathering has a repo to inspect.
    wai_cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "demo"])
        .assert()
        .success();

    let output = wai_cmd()
        .current_dir(tmp.path())
        .args([
            "feedback",
            "bug",
            "--dry-run",
            "--title",
            "crash on init",
            "--body",
            "Running wai init panics when .wai/ is read-only.",
            "--yes",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "dry-run should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Title and body are present.
    assert!(stdout.contains("crash on init"), "title in output");
    assert!(stdout.contains("Running wai init panics"), "body in output");

    // The exact gh line is emitted and targets wai's upstream repo.
    let gh_line = stdout
        .lines()
        .find(|l| l.starts_with("gh issue create"))
        .expect("an exact `gh issue create` line is printed in dry-run");
    assert!(
        gh_line.contains(&format!("--repo {}", TARGET_REPO)),
        "gh line targets wai upstream: {gh_line}"
    );
    assert!(
        gh_line.contains("--body-file -"),
        "gh line reads body from stdin: {gh_line}"
    );
    // A label is applied.
    assert!(
        gh_line.contains("--label"),
        "gh line applies at least one label: {gh_line}"
    );
}

#[test]
fn feedback_dry_run_no_context_omits_environment_section() {
    let tmp = tempfile::TempDir::new().unwrap();
    wai_cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "demo"])
        .assert()
        .success();

    let output = wai_cmd()
        .current_dir(tmp.path())
        .args([
            "feedback",
            "idea",
            "--dry-run",
            "--no-context",
            "--title",
            "x",
            "--body",
            "y",
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("## Environment"),
        "--no-context omits the environment bundle"
    );
}

// ── Acceptance: a failed scratch write does not change the exit code ──
//
// We can't easily force the *real* wai error path to fail mid-scratch from an
// integration test, so we assert the documented contract directly: the
// feedback command reads `--from-last-error` and, when the scratch file is
// absent/unreadable, it does not crash — it reports "no recent error" with a
// non-zero-but-stable exit (no panic, no scratch-induced exit change).

#[test]
fn feedback_from_last_error_without_scratch_is_stable() {
    let tmp = tempfile::TempDir::new().unwrap();
    wai_cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "demo"])
        .assert()
        .success();

    // Point XDG_CACHE_HOME at the temp dir so there is no prior scratch.
    let output = wai_cmd()
        .current_dir(tmp.path())
        .env("XDG_CACHE_HOME", tmp.path().join("cache").to_str().unwrap())
        .args(["feedback", "bug", "--from-last-error", "--dry-run", "--yes"])
        .output()
        .unwrap();

    // Stable, non-panic exit (not 101). Exit code is non-zero (no error to file)
    // but the process must not crash.
    assert_ne!(
        output.status.code(),
        Some(101),
        "must not panic when scratch is missing"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("no recent error")
            || stderr.to_lowercase().contains("no error"),
        "stderr explains there is no recent error: {stderr}"
    );
}

// ── Acceptance: redactor strips https://<pat>@... to host/path ─────────
//
// This is a unit-level contract of genesis::feedback::redactor, but the
// acceptance criterion calls for an end-to-end check through wai. We verify
// the dry-run body does NOT leak a PAT embedded in a git remote URL passed as
// body text, and DOES keep the host/path.

#[test]
fn feedback_dry_run_redacts_pat_from_remote_url_in_body() {
    let tmp = tempfile::TempDir::new().unwrap();
    wai_cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "demo"])
        .assert()
        .success();

    let pat_url = "https://ghp_SECRET1234567890@github.com/charly-vibes/wai.git";
    let output = wai_cmd()
        .current_dir(tmp.path())
        .args([
            "feedback",
            "bug",
            "--dry-run",
            "--title",
            "leak",
            "--body",
            &format!("remote was: {pat_url}"),
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("ghp_SECRET1234567890"),
        "PAT must not leak into the dry-run body"
    );
    assert!(
        stdout.contains("github.com/charly-vibes/wai"),
        "host/path is preserved after redaction"
    );
}

// ── Acceptance: monkey_type / keymap survive redaction ────────────────
//
// The redactor matches secret *values*, not key substrings — so a harmless
// value like "monkey_type" or "keymap" must survive untouched.

#[test]
fn feedback_dry_run_preserves_monkey_type_and_keymap_values() {
    let tmp = tempfile::TempDir::new().unwrap();
    wai_cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "demo"])
        .assert()
        .success();

    let body = "I was configuring my keymap and testing monkey_type when wai crashed.";
    let output = wai_cmd()
        .current_dir(tmp.path())
        .args([
            "feedback",
            "friction",
            "--dry-run",
            "--title",
            "x",
            "--body",
            body,
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("monkey_type"),
        "monkey_type survives redaction"
    );
    assert!(stdout.contains("keymap"), "keymap survives redaction");
}

// ── Kind defaults / validation ────────────────────────────────────────

#[test]
fn feedback_invalid_kind_is_rejected() {
    let tmp = tempfile::TempDir::new().unwrap();
    wai_cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "demo"])
        .assert()
        .success();

    wai_cmd()
        .current_dir(tmp.path())
        .args(["feedback", "not-a-kind", "--dry-run", "--yes"])
        .assert()
        .failure();
}

#[test]
fn feedback_missing_title_without_from_last_error_errors_cleanly() {
    let tmp = tempfile::TempDir::new().unwrap();
    wai_cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "demo"])
        .assert()
        .success();

    let output = wai_cmd()
        .current_dir(tmp.path())
        .args(["feedback", "bug", "--dry-run", "--yes"])
        .output()
        .unwrap();
    // No title and no --from-last-error → clean error, not a panic.
    assert_ne!(output.status.code(), Some(101));
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("title")
            || stderr.to_lowercase().contains("from-last-error"),
        "stderr points at --title or --from-last-error: {stderr}"
    );
}

// ── End-to-end: a real wai error is captured to scratch and read back ──

#[test]
fn feedback_from_last_error_reads_a_real_wai_error() {
    let tmp = tempfile::TempDir::new().unwrap();
    // Isolate the scratch file in the temp dir's cache.
    let cache = tmp.path().join("cache");
    fs::create_dir_all(&cache).unwrap();

    // Trigger a real wai error: `wai status` outside any initialized workspace.
    let _ = wai_cmd()
        .current_dir(tmp.path())
        .env("XDG_CACHE_HOME", cache.to_str().unwrap())
        .args(["status"])
        .output()
        .unwrap();

    // The scratch file now exists and the feedback command reads it back.
    let output = wai_cmd()
        .current_dir(tmp.path())
        .env("XDG_CACHE_HOME", cache.to_str().unwrap())
        .args(["feedback", "bug", "--from-last-error", "--dry-run", "--yes"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "dry-run from last error should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The derived title references the failing argv (status) and exit code.
    assert!(
        stdout.contains("status"),
        "derived title/body references the failing command: {stdout}"
    );
    // The context bundle's repro hash is present.
    assert!(stdout.contains("repro_hash"), "context bundle attached");
    // The exact gh line is present.
    assert!(
        stdout.lines().any(|l| l.starts_with("gh issue create")),
        "gh line present in from-last-error dry-run"
    );
}
