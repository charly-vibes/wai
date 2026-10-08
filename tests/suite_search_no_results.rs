#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn search_no_results() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["search", "zzz_nonexistent_zzz"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No results found"));
}

#[test]
fn search_json_outputs_results() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-api-notes.md",
        "JSON output is required.\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "JSON", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"results\""));
}

#[test]
fn search_with_type_filter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "keyword_match here\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "plans",
        "2026-01-15-plan.md",
        "keyword_match in plan\n",
    );

    // Filter to research only
    let out = wai_cmd(tmp.path())
        .args(["search", "keyword_match", "--type", "research"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("research/"));
    assert!(stdout.contains("keyword_match"));
    assert!(!(stdout.contains("plans/")));
}

#[test]
fn phase_show_displays_via_wai_project_indicator() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .env("WAI_PROJECT", "alpha")
        .args(["phase", "show"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = strip_ansi(&String::from_utf8_lossy(&out));
    assert!(
        stdout.contains("[via WAI_PROJECT]"),
        "expected [via WAI_PROJECT] in: {}",
        stdout
    );
}

#[test]
fn phase_show_displays_via_project_flag_indicator() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .args(["phase", "--project", "alpha", "show"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let stdout = strip_ansi(&String::from_utf8_lossy(&out));
    assert!(
        stdout.contains("[via --project]"),
        "expected [via --project] in: {}",
        stdout
    );
}

#[test]
fn phase_show_no_indicator_for_auto_detect() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "only");

    let out = wai_cmd(tmp.path())
        .args(["phase", "show"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let stdout = strip_ansi(&String::from_utf8_lossy(&out));
    assert!(
        !stdout.contains("[via"),
        "expected no [via] indicator for auto-detect in: {}",
        stdout
    );
}

// ─── Doctor WAI_PROJECT checks ───────────────────────────────────────────────

#[test]
fn doctor_warns_wai_project_nonexistent() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .env("WAI_PROJECT", "nonexistent")
        .args(["doctor"])
        .assert()
        .get_output()
        .stdout
        .clone();
    let stdout = strip_ansi(&String::from_utf8_lossy(&out));
    assert!(
        stdout.contains("WAI_PROJECT"),
        "expected WAI_PROJECT warning in: {}",
        stdout
    );
}

#[test]
fn doctor_warns_wai_project_empty() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .env("WAI_PROJECT", "")
        .args(["doctor"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = strip_ansi(&String::from_utf8_lossy(&out));
    // Empty WAI_PROJECT should not trigger a project-not-found error
    assert!(
        !stdout.contains("project not found"),
        "empty WAI_PROJECT should not trigger project-not-found in: {}",
        stdout
    );
}

// ─── Genesis CLI ergonomics (wai-4guc) ────────────────────────────────────────

#[test]
fn cli_format_human_flag_is_accepted() {
    // `--human` is provided by genesis CliFormat; it must parse and behave as
    // the default human output (no JSON envelope) in an initialized workspace.
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["--human", "status"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let stdout = strip_ansi(&String::from_utf8_lossy(&out));
    assert!(
        !stdout.trim_start().starts_with('{'),
        "--human should emit human text, not a JSON envelope: {}",
        stdout
    );
}

#[test]
fn cli_format_json_and_human_conflict() {
    // CliFormat declares --json/--human as mutually exclusive; clap must reject
    // passing both together.
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["--json", "--human", "status"])
        .output()
        .expect("command should run");
    assert!(!out.status.success());
}

#[test]
fn cli_verbosity_counts_with_format_flags() {
    // CliVerbosity (-v/-vv) and CliFormat (--json) must coexist as global
    // flags: a verbose JSON status call still emits a parseable envelope.
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["-vv", "--json", "status"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let stdout = String::from_utf8_lossy(&out);
    let payload: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(
        payload["ok"], true,
        "verbose --json status should be valid envelope"
    );
}

#[test]
fn cli_verbosity_quiet_and_verbose_coexist() {
    // CliVerbosity accepts -q and -v together; --quiet takes precedence over
    // the -v count, but both must parse as valid global flags.
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["-q", "-v", "status"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

#[test]
fn stale_run_doctor_flags_old_midflight_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace_with_project(tmp.path(), "my-app");
    write_stale_run_fixture(tmp.path(), "my-pipe", "my-pipe-old-run", 0, 30);

    let output = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(
        stdout.contains("stale") && stdout.contains("my-pipe-old-run"),
        "doctor must flag a 30-day-old mid-flight run as stale, got: {stdout}"
    );
    assert!(
        stdout.contains("wai pipeline gc --yes"),
        "stale warning must name the quarantine command, got: {stdout}"
    );
}

#[test]
fn stale_run_doctor_ignores_fresh_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace_with_project(tmp.path(), "my-app");
    write_stale_run_fixture(tmp.path(), "my-pipe", "my-pipe-fresh-run", 0, 2);

    let output = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(
        !stdout.contains("my-pipe-fresh-run"),
        "doctor must not flag a 2-day-old run as stale, got: {stdout}"
    );
}

#[test]
fn stale_run_doctor_ignores_old_complete_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace_with_project(tmp.path(), "my-app");
    // current_step == 2 (total) → complete, not mid-flight → not GC material.
    write_stale_run_fixture(tmp.path(), "my-pipe", "my-pipe-done-run", 2, 30);

    let output = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(
        !stdout.contains("my-pipe-done-run"),
        "doctor must not flag an old COMPLETE run as stale, got: {stdout}"
    );
}

#[test]
fn stale_run_gc_dry_run_lists_without_moving() {
    let tmp = TempDir::new().unwrap();
    init_workspace_with_project(tmp.path(), "my-app");
    write_stale_run_fixture(tmp.path(), "my-pipe", "my-pipe-old-run", 1, 30);
    let run_path = tmp.path().join(".wai/pipeline-runs/my-pipe-old-run.yml");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "gc"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-pipe-old-run"));
    assert!(stdout.contains("--yes"));

    assert!(run_path.exists(), "dry run must not move the run file");
    assert!(
        tmp.path()
            .join(".wai/resources/pipelines/.last-run")
            .exists(),
        "dry run must not remove the pointer"
    );
}

#[test]
fn stale_run_gc_yes_quarantines_under_stale_dir() {
    let tmp = TempDir::new().unwrap();
    init_workspace_with_project(tmp.path(), "my-app");
    write_stale_run_fixture(tmp.path(), "my-pipe", "my-pipe-old-run", 1, 30);
    let run_path = tmp.path().join(".wai/pipeline-runs/my-pipe-old-run.yml");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "gc", "--yes"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-pipe-old-run"));

    assert!(
        !run_path.exists(),
        "quarantined run must be moved out of pipeline-runs/"
    );
    let stale_dir = tmp.path().join(".wai/pipeline-runs/stale");
    let quarantined: Vec<_> = fs::read_dir(&stale_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        quarantined
            .iter()
            .any(|f| f.starts_with("my-pipe-old-run") && f.ends_with(".yml")),
        "run file must be preserved under stale/ with original name + timestamp suffix, got: {quarantined:?}"
    );
    assert!(
        !tmp.path()
            .join(".wai/resources/pipelines/.last-run")
            .exists(),
        "pointer to a quarantined run must be removed"
    );

    // prime no longer shows the run as current.
    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "my-app", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("ADOPT/RESUME")));
}

#[test]
fn stale_run_gc_leaves_fresh_runs_untouched() {
    let tmp = TempDir::new().unwrap();
    init_workspace_with_project(tmp.path(), "my-app");
    write_stale_run_fixture(tmp.path(), "my-pipe", "my-pipe-fresh-run", 1, 2);
    let run_path = tmp.path().join(".wai/pipeline-runs/my-pipe-fresh-run.yml");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "gc", "--yes"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No stale runs"));

    assert!(run_path.exists(), "fresh run must not be quarantined");
    assert!(
        tmp.path()
            .join(".wai/resources/pipelines/.last-run")
            .exists(),
        "pointer to a fresh run must survive gc"
    );
}

#[test]
fn stale_run_config_stale_days_override() {
    let tmp = TempDir::new().unwrap();
    init_workspace_with_project(tmp.path(), "my-app");
    write_stale_run_fixture(tmp.path(), "my-pipe", "my-pipe-old-run", 1, 30);
    // Raise the threshold: a 30-day-old run is no longer stale at staleDays=60.
    fs::write(
        tmp.path().join(".wai/config.toml"),
        "[project]\nname = \"test-ws\"\n\n[pipeline]\nstaleDays = 60\n",
    )
    .unwrap();

    let output = wai_cmd(tmp.path())
        .args(["doctor"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(
        !stdout.contains("my-pipe-old-run"),
        "staleDays=60 must keep a 30-day-old run under threshold, got: {stdout}"
    );
}

#[test]
fn pipeline_start_epic_discovers_children_creates_parent_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");
    let fake_bin = install_fake_bd_ready_json(
        tmp.path(),
        r#"[{"id":"child-a","parent":"epic-1"},{"id":"child-b","parent":"epic-1"},{"id":"other","parent":"epic-2"}]"#,
    );
    let path = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--epic=epic-1"])
        .env("PATH", path.clone())
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert_parent_run_state(tmp.path());

    // Idempotent: a second start for the same epic reuses the parent run.
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--epic=epic-1"])
        .env("PATH", path)
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let runs_dir = tmp.path().join(".wai/pipeline-runs");
    let ymls2: Vec<_> = fs::read_dir(&runs_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yml"))
        .collect();
    assert_eq!(
        ymls2.len(),
        1,
        "second start must not create a duplicate parent run"
    );
}

#[test]
fn pipeline_start_epic_skips_when_no_ready_children() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");
    let fake_bin = install_fake_bd_ready_json(tmp.path(), r#"[{"id":"other","parent":"epic-2"}]"#);
    let path = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--epic=epic-1"])
        .env("PATH", path)
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("no ready children"));

    let runs_dir = tmp.path().join(".wai/pipeline-runs");
    if runs_dir.exists() {
        let ymls: Vec<_> = fs::read_dir(&runs_dir)
            .unwrap()
            .flatten()
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yml"))
            .collect();
        assert!(
            ymls.is_empty(),
            "no parent run must be created without ready children"
        );
    }
    assert!(
        !tmp.path()
            .join(".wai/resources/pipelines/.last-run")
            .exists(),
        ".last-run must not be written when no parent run is created"
    );
}

#[test]
fn pipeline_start_child_appends_run_to_epic_parent() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");
    write_epic_run_fixture(
        tmp.path(),
        "my-pipe-parent",
        "my-pipe",
        "epic-1",
        &["child-a"],
        &[],
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=child-a"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Find the child run file just created (the one that is not the parent).
    let runs_dir = tmp.path().join(".wai/pipeline-runs");
    let child_run: String = fs::read_dir(&runs_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yml"))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().to_string())
        .find(|stem| stem != "my-pipe-parent")
        .expect("child run state must exist");

    let parent = fs::read_to_string(runs_dir.join("my-pipe-parent.yml")).unwrap();
    assert!(
        parent.contains(&child_run),
        "epic parent state must record the child run id '{child_run}', got: {parent}"
    );
}

#[test]
fn pipeline_next_epic_parent_blocked_while_children_midflight() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");
    write_epic_run_fixture(
        tmp.path(),
        "my-pipe-parent",
        "my-pipe",
        "epic-1",
        &["child-a"],
        &["child-run"],
    );
    // Mid-flight child run: step 0 of 2.
    write_child_run_fixture(tmp.path(), "child-run", "my-pipe", "child-a", 0, None);

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("child-run"));

    let parent =
        fs::read_to_string(tmp.path().join(".wai/pipeline-runs/my-pipe-parent.yml")).unwrap();
    assert!(
        parent.contains("current_step: 0"),
        "refused parent advance must not modify parent state, got: {parent}"
    );
}

#[test]
fn pipeline_current_json_renders_epic_tree() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");
    write_epic_run_fixture(
        tmp.path(),
        "my-pipe-parent",
        "my-pipe",
        "epic-1",
        &["child-a"],
        &["child-run"],
    );
    write_child_run_fixture(tmp.path(), "child-run", "my-pipe", "child-a", 0, None);

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "current", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("epic-1"));
    assert!(stdout.contains("\"run_id\": \"child-run\""));
    assert!(stdout.contains("\"mid_flight\": true"));
}

// ─── inter-child handoff artifacts (wai-vx02.5) ────────────────────────────
// A child run that reaches its terminal step with a handoff artifact recorded
// (via `wai handoff create` while the run is active) is handoff-ready: the
// epic tree marks it and links the artifact path, and starting the next child
// surfaces the prior child's handoff artifact path in `pipeline current`.

#[test]
fn epic_handoff_terminal_child_marks_handoff_ready() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");
    write_epic_run_fixture(
        tmp.path(),
        "my-pipe-parent",
        "my-pipe",
        "epic-1",
        &["child-a"],
        &["child-run-a"],
    );
    // Terminal child run (step 2 of 2) with a recorded handoff artifact.
    write_child_run_fixture(
        tmp.path(),
        "child-run-a",
        "my-pipe",
        "child-a",
        2,
        Some("handoffs/2026-10-05-session-end.md"),
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "current", "--json"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"handoff_ready\": true"));
    assert!(stdout.contains("handoffs/2026-10-05-session-end.md"));
}

#[test]
fn epic_handoff_midflight_child_not_handoff_ready() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");
    write_epic_run_fixture(
        tmp.path(),
        "my-pipe-parent",
        "my-pipe",
        "epic-1",
        &["child-a"],
        &["child-run-a"],
    );
    // Mid-flight child run (step 0 of 2) that already recorded an artifact:
    // not handoff-ready until the run reaches its terminal step.
    write_child_run_fixture(
        tmp.path(),
        "child-run-a",
        "my-pipe",
        "child-a",
        0,
        Some("handoffs/2026-10-05-session-end.md"),
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "current", "--json"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"handoff_ready\": false"));
    assert!(stdout.contains("\"mid_flight\": true"));
}

#[test]
fn epic_handoff_create_records_artifact_on_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_pipeline_toml(tmp.path(), "my-pipe");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=child-a"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["handoff", "create", "my-app"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // The active (child) run state must record the created handoff artifact.
    let runs_dir = tmp.path().join(".wai/pipeline-runs");
    let child_run: String = fs::read_dir(&runs_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yml"))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().to_string())
        .find(|stem| stem != "my-pipe-parent")
        .expect("child run state must exist");
    let state = fs::read_to_string(runs_dir.join(format!("{child_run}.yml"))).unwrap();
    assert!(
        state.contains(".wai/projects/my-app/handoffs/2026-"),
        "child run state must record the handoff artifact path, got: {state}"
    );
}

#[test]
fn epic_handoff_next_child_surfaces_prior_sibling_handoff() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");
    write_epic_run_fixture(
        tmp.path(),
        "my-pipe-parent",
        "my-pipe",
        "epic-1",
        &["child-a", "child-b"],
        &["child-run-a"],
    );
    // Prior child completed (terminal) with a recorded handoff artifact.
    write_child_run_fixture(
        tmp.path(),
        "child-run-a",
        "my-pipe",
        "child-a",
        2,
        Some("handoffs/2026-10-05-session-end.md"),
    );

    // Starting the NEXT child makes it the active run...
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=child-b"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // ...and `pipeline current` must surface the prior child's handoff path.
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "current", "--json"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("handoffs/2026-10-05-session-end.md"));
}

/// Assert the epic start created exactly one `-parent` run state recording
/// the epic id and only its ready children.
fn assert_parent_run_state(tmp: &std::path::Path) {
    let runs_dir = tmp.join(".wai/pipeline-runs");
    let ymls: Vec<_> = fs::read_dir(&runs_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yml"))
        .collect();
    assert_eq!(ymls.len(), 1, "exactly one parent run state expected");
    let stem = ymls[0]
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert!(
        stem.ends_with("-parent"),
        "parent run id must end in -parent, got: {stem}"
    );

    let content = fs::read_to_string(ymls[0].path()).unwrap();
    assert!(
        content.contains("epic: epic-1"),
        "parent run must record the epic id, got: {content}"
    );
    assert!(
        content.contains("- child-a"),
        "ready child must be discovered: {content}"
    );
    assert!(
        content.contains("- child-b"),
        "ready child must be discovered: {content}"
    );
    assert!(
        !content.contains("- other"),
        "children of other epics must not be recorded: {content}"
    );
}
