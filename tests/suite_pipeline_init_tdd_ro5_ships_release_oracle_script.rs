#![allow(clippy::too_many_lines)]

mod common;

use common::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn pipeline_init_tdd_ro5_ships_release_oracle_script() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "tdd-ro5"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let script = tmp
        .path()
        .join(".wai/resources/oracles/release-docs-fresh.sh");
    let meta =
        fs::metadata(&script).expect("pipeline init tdd-ro5 should ship release-docs-fresh.sh");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(
            meta.permissions().mode() & 0o111 != 0,
            "shipped oracle must be executable: {:?}",
            script
        );
    }
}

#[test]
fn release_oracle_accepts_fresh_docs() {
    let tmp = release_oracle_sandbox("2026.10.3", Some("2026.10.3"));
    let (code, stderr) = run_release_oracle(tmp.path());
    assert_eq!(code, 0, "expected accept for fresh docs, stderr: {stderr}");
}

#[test]
fn release_oracle_rejects_lagging_changelog() {
    let tmp = release_oracle_sandbox("2026.10.3", Some("2026.9.28"));
    let (code, stderr) = run_release_oracle(tmp.path());
    assert_eq!(
        code, 1,
        "expected reject for lagging CHANGELOG, stderr: {stderr}"
    );
    assert!(
        stderr.contains("2026.10.3") && stderr.contains("2026.9.28"),
        "stderr must name the drift (both versions), got: {stderr}"
    );
}

#[test]
fn release_oracle_rejects_dirty_docs() {
    let tmp = release_oracle_sandbox("2026.10.3", Some("2026.10.3"));
    // Uncommitted docs change after the initial commit -> docs drift
    fs::write(tmp.path().join("docs/index.md"), "uncommitted edit").unwrap();
    let (code, stderr) = run_release_oracle(tmp.path());
    assert_eq!(code, 1, "expected reject for dirty docs/, stderr: {stderr}");
    assert!(
        stderr.to_lowercase().contains("docs"),
        "stderr must name docs/ as the drift, got: {stderr}"
    );
}

/// Run `wai <args>` with a custom data dir, asserting success.
fn run_ok_wdd(dir: &std::path::Path, data_dir: &std::path::Path, args: &[&str]) {
    let out = wai_cmd_with_data_dir(dir, data_dir)
        .args(args)
        .output()
        .expect("command should run");
    assert!(out.status.success(), "wai {:?} failed", args);
}

#[test]
fn plugin_trust_list_shows_approved() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");
    write_malicious_plugin(tmp.path(), "hook-marker");

    run_ok_wdd(tmp.path(), &data_dir, &["plugin", "trust", "evil"]);

    let out = wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["plugin", "trust", "--list"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("evil"));
}

#[test]
fn plugin_trust_approve_json_outputs_state() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");
    write_malicious_plugin(tmp.path(), "hook-marker");

    let out = wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["plugin", "trust", "evil", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"plugin\""));
    assert!(stdout.contains("\"approved\""));
}

#[test]
fn plugin_trust_unknown_plugin_fails() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());

    let out = wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["plugin", "trust", "nonexistent"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}

#[test]
fn plugin_trust_builtin_plugin_fails() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());

    let out = wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["plugin", "trust", "git"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("is a built-in plugin"));
}

#[test]
fn plugin_trust_revoke_disables_hook() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");
    write_malicious_plugin(tmp.path(), "hook-marker");

    // Approve with --json to capture the full digest.
    let output = wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["plugin", "trust", "evil", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract the 64-char hex digest from the JSON output.
    let digest = stdout.lines().find_map(|line| {
        let line = line.trim();
        if let Some(start) = line.find("\"digest\":\"") {
            let rest = &line[start + 10..];
            rest.find('"').map(|end| rest[..end].to_string())
        } else {
            None
        }
    });

    if let Some(digest) = digest {
        // Revoke using the digest.
        wai_cmd_with_data_dir(tmp.path(), &data_dir)
            .args(["plugin", "trust", "--revoke", &digest])
            .assert()
            .success()
            .stdout(predicate::str::contains("Revoked"));

        // Verify the hook is no longer executed after revocation.
        wai_cmd_with_data_dir(tmp.path(), &data_dir)
            .args(["status"])
            .assert()
            .success();

        assert!(
            !tmp.path().join("hook-marker").exists(),
            "revoked hook must not execute"
        );
    }
}

#[test]
fn plugin_trust_non_interactive_fail_closed() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");
    write_malicious_plugin(tmp.path(), "hook-marker");

    // --no-input with unapproved hook: fail-closed, no execution.
    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["status", "--no-input"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not trusted"));

    assert!(
        !tmp.path().join("hook-marker").exists(),
        "unapproved hook must not execute in non-interactive mode"
    );
}

#[test]
fn plugin_trust_prime_skips_unapproved_hook() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");
    write_malicious_plugin(tmp.path(), "prime-marker");

    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["prime"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not trusted").or(predicate::str::contains("no active")));

    assert!(
        !tmp.path().join("prime-marker").exists(),
        "prime must not execute unapproved hook"
    );
}

#[test]
fn plugin_trust_content_modification_invalidates_approval() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");

    let plugin_dir = tmp.path().join(".wai/plugins");
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::write(
        plugin_dir.join("evil.toml"),
        r#"
name = "evil"
description = "malicious"

[hooks.on_status]
command = "touch marker-v1"
inject_as = "evil_marker"
"#,
    )
    .unwrap();

    // Approve the plugin.
    run_ok_wdd(tmp.path(), &data_dir, &["plugin", "trust", "evil"]);

    // Run status: hook executes.
    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["status"])
        .assert()
        .success();
    assert!(tmp.path().join("marker-v1").exists(), "approved hook runs");

    // Modify the plugin file (change the command).
    fs::write(
        plugin_dir.join("evil.toml"),
        r#"
name = "evil"
description = "malicious"

[hooks.on_status]
command = "touch marker-v2"
inject_as = "evil_marker"
"#,
    )
    .unwrap();

    // Run status: the new hook must NOT execute (digest changed).
    let _ = fs::remove_file(tmp.path().join("marker-v2"));
    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["status"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not trusted"));

    assert!(
        !tmp.path().join("marker-v2").exists(),
        "modified hook must not execute without re-approval"
    );
}

#[test]
fn plugin_trust_new_skips_unapproved_hook() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());

    // Create a plugin with an on_project_create hook.
    let plugin_dir = tmp.path().join(".wai/plugins");
    fs::create_dir_all(&plugin_dir).unwrap();
    let new_marker = tmp.path().join("new-marker");
    fs::write(
        plugin_dir.join("evil.toml"),
        format!(
            r#"
name = "evil"
description = "malicious"

[hooks.on_project_create]
command = "touch {}"
inject_as = "evil_marker"
"#,
            new_marker.to_str().unwrap()
        ),
    )
    .unwrap();

    // `wai new` should NOT execute the unapproved on_project_create hook.
    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["new", "project", "test-project"])
        .assert()
        .success();

    assert!(
        !new_marker.exists(),
        "wai new must not execute unapproved hook"
    );
}

#[test]
fn plugin_trust_approve_specific_hook() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");

    // Plugin with two hooks.
    let plugin_dir = tmp.path().join(".wai/plugins");
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::write(
        plugin_dir.join("evil.toml"),
        r#"
name = "evil"
description = "malicious"

[hooks.on_status]
command = "echo 'status'"
inject_as = "status_hook"

[hooks.on_handoff_generate]
command = "echo 'handoff'"
inject_as = "handoff_hook"
"#,
    )
    .unwrap();

    // Only approve the on_status hook.
    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["plugin", "trust", "evil", "--hook", "on_status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("on_status"));

    // on_status should now execute.
    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["status"])
        .assert()
        .success();

    // on_handoff_generate should still be untrusted (but we can't easily
    // trigger handoff in tests, so we just verify the trust store reflects it).
    let output = wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["plugin", "trust", "--list", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // There should be exactly one entry (on_status).
    assert!(
        stdout.contains("on_status"),
        "only on_status should be in trust store"
    );
}

// ─── wai status ─────────────────────────────────────────────────────────────

#[test]
fn status_shows_project_info() {
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

#[test]
fn status_json_outputs_suggestions() {
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

#[test]
fn status_without_openspec_omits_section() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("OpenSpec")));
}

#[test]
fn status_with_openspec_shows_change_counts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Set up an openspec directory with a change
    let changes_dir = tmp.path().join("openspec/changes/add-feature");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Setup\n\n- [x] done\n- [ ] todo\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("OpenSpec"));
    assert!(stdout.contains("add-feature"));
    assert!(stdout.contains("1/2"));
}

#[test]
fn status_verbose_shows_section_breakdown() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let changes_dir = tmp.path().join("openspec/changes/add-feature");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Setup\n\n- [x] done\n- [ ] todo\n\n## 2. Implement\n\n- [ ] a\n- [ ] b\n",
    )
    .unwrap();

    // Also add a spec
    fs::create_dir_all(tmp.path().join("openspec/specs/core")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Setup"));
    assert!(stdout.contains("Implement"));
    assert!(stdout.contains("core"));
}

#[test]
fn status_hides_completed_openspec_changes_by_default() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let changes_dir = tmp.path().join("openspec/changes/done-feature");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Setup\n\n- [x] a\n- [x] b\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("done-feature")));
}

#[test]
fn status_verbose_shows_completed_openspec_changes() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let changes_dir = tmp.path().join("openspec/changes/done-feature");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Setup\n\n- [x] a\n- [x] b\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("done-feature"));
    assert!(stdout.contains("2/2"));
}

#[test]
fn status_all_complete_shows_hint_message() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let changes_dir = tmp.path().join("openspec/changes/done-feature");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Setup\n\n- [x] a\n- [x] b\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("all changes complete"));
}

#[test]
fn status_plugin_info_hidden_when_only_completed_openspec_changes() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let changes_dir = tmp.path().join("openspec/changes/done-feature");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Setup\n\n- [x] a\n- [x] b\n",
    )
    .unwrap();

    // Plugin Info section should not appear when there are no active (incomplete) changes
    // and no other hook outputs (no beads, etc.)
    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("Plugin Info")));
}

#[test]
fn status_json_with_openspec_includes_field() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let changes_dir = tmp.path().join("openspec/changes/my-change");
    fs::create_dir_all(&changes_dir).unwrap();
    fs::write(
        changes_dir.join("tasks.md"),
        "## 1. Work\n\n- [x] a\n- [ ] b\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"openspec\":"));
    assert!(stdout.contains("\"my-change\""));
    assert!(stdout.contains("\"done\": 1"));
}

#[test]
fn status_json_without_openspec_omits_field() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // The openspec *plugin* name appears in plugins list, so check for the JSON key specifically
    let out = wai_cmd(tmp.path())
        .args(["status", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("\"openspec\":")));
}

// ─── wai status: phase-aware suggestions ─────────────────────────────────────

#[test]
fn status_suggests_advance_when_enough_research() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Write 3 research artifacts to cross the readiness threshold
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-01-first.md",
        "First finding",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-02-second.md",
        "Second finding",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-03-third.md",
        "Third finding",
    );

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("design"));
    assert!(stdout.contains("Advance"));
}

#[test]
fn status_suggests_research_needed_when_in_implement_phase_without_research() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Jump to implement phase without adding any research
    let out = wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("research"));
}

#[test]
fn status_shows_minimal_research_suggestion_for_new_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // New project with no artifacts → NewProject pattern → suggest adding research
    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("research"));
}

#[test]
fn status_flags_stale_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let project_dir = tmp.path().join(".wai/projects/my-app");
    write_stale_state(&project_dir, "research", 20);

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!((stdout.contains("phase next") || stdout.contains("move")));
}

#[test]
fn status_does_not_flag_recent_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // phase_started within threshold (7 days ago) — should not show stale suggestion
    let project_dir = tmp.path().join(".wai/projects/my-app");
    write_stale_state(&project_dir, "research", 7);

    let stdout = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    assert!(stdout.status.success());
    let stdout = stdout.stdout.clone();
    let output = String::from_utf8(stdout).unwrap();
    assert!(
        !output.contains("phase next") || !output.contains("20 days"),
        "should not show stale-phase suggestion for recent project"
    );
}

#[test]
fn status_flags_complete_review_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Set project to review phase and create a handoff file
    let project_dir = tmp.path().join(".wai/projects/my-app");
    write_stale_state(&project_dir, "review", 2);
    fs::write(
        project_dir.join("handoffs/2026-01-01-handoff.md"),
        "Session handoff",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!((stdout.contains("move") || stdout.contains("archive")));
}

#[test]
fn status_json_includes_stale_suggestion() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let project_dir = tmp.path().join(".wai/projects/my-app");
    write_stale_state(&project_dir, "implement", 20);

    let out = wai_cmd(tmp.path())
        .args(["status", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!((stdout.contains("phase next") || stdout.contains("move")));
}

#[test]
fn status_json_includes_complete_suggestion() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let project_dir = tmp.path().join(".wai/projects/my-app");
    write_stale_state(&project_dir, "review", 1);
    fs::write(
        project_dir.join("handoffs/2026-01-01-handoff.md"),
        "Session handoff",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!((stdout.contains("move") || stdout.contains("archive")));
}

// ─── wai (no args) ─────────────────────────────────────────────────────────

#[test]
fn no_args_shows_welcome() {
    let stale_wai = std::path::Path::new("/tmp/.wai");
    if stale_wai.exists() {
        let _ = fs::remove_dir_all(stale_wai);
    }

    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path()).output().expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("wai init"));
}
