#![allow(clippy::too_many_lines)]

mod common;

use chrono::Local;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn no_args_in_initialized_dir_shows_commands() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path()).output().expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("wai status"));
}

// ─── wai doctor ─────────────────────────────────────────────────────────────

#[test]
fn doctor_healthy_workspace_all_pass() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"fail\": 0"));
    assert!(stdout.contains("\"summary\""));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn doctor_missing_directories_fails_with_fix() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::remove_dir_all(tmp.path().join(".wai/archives")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout.contains("\"status\": \"fail\""));
    assert!(stdout.contains("archives"));
    assert!(stdout.contains("doctor --fix"));
}

#[test]
fn doctor_invalid_config_toml_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    fs::write(tmp.path().join(".wai/config.toml"), "{{invalid toml!!!").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout.contains("\"status\": \"fail\""));
    assert!(stdout.contains("Configuration"));
}

#[test]
fn doctor_missing_plugin_tool_warns() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create .git dir so the git plugin is detected
    fs::create_dir(tmp.path().join(".git")).unwrap();

    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\": \"warn\""));
    assert!(stdout.contains("not found in PATH"));
}

#[test]
fn prime_project_flag_selects_correct_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "alpha", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project: alpha"));
}

#[test]
fn prime_zero_projects_shows_suggestion() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["prime", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("wai new project"));
}

#[test]
fn prime_multiple_projects_no_input_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--no-input"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("--project <name>"));
}

#[test]
fn prime_unknown_project_fails_with_available() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "doesnotexist", "--no-input"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("doesnotexist"));
    assert!(stderr.contains("myproject"));
}

#[test]
fn prime_all_headings_handoff_shows_no_summary_yet() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    write_handoff(
        tmp.path(),
        "myproject",
        "2026-02-23-session-end.md",
        "---\ndate: 2026-02-23\nproject: myproject\nphase: research\n---\n\n# Heading One\n\n## Heading Two\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("no summary yet"));
}

#[test]
fn prime_handoff_code_fence_content_not_used_as_snippet() {
    // Regression: find_first_paragraph used to leak code-fence content (e.g. git
    // status lines) as the handoff snippet when all real sections were empty.
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    write_handoff(
        tmp.path(),
        "myproject",
        "2026-02-23-session-end.md",
        "---\ndate: 2026-02-23\nproject: myproject\nphase: implement\n---\n\n\
         ## What Was Done\n\n<!-- placeholder -->\n\n\
         ## Context\n\n### git_status\n\n```\nM  src/main.rs\n```\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("no summary yet"));
    assert!(regex::Regex::new("(?s).*").unwrap().is_match(&stdout));
    // Must NOT show the raw git-status line as the snippet
    assert!(!stdout.contains("M  src/main.rs"));
}

#[test]
fn prime_outside_workspace_fails() {
    let tmp = TempDir::new().unwrap();
    // No wai init — no .wai/ directory

    let out = wai_cmd(tmp.path())
        .args(["prime", "--no-input"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("wai init"));
}

#[test]
fn prime_shows_resuming_block_when_pending_resume_present_today() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let today = Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("{today}-session-end.md");
    let content = format!(
        "---\ndate: {today}\nproject: myproject\nphase: implement\n---\n\n\
         Implementing the state machine.\n\n\
         ## Next Steps\n\n\
         1. Finish src/state.rs\n\
         2. Write tests\n"
    );
    write_handoff(tmp.path(), "myproject", &filename, &content);
    write_pending_resume(tmp.path(), "myproject", &format!("handoffs/{filename}"));

    let stdout = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    assert!(stdout.status.success());
    let stdout = stdout.stdout.clone();
    let out = String::from_utf8(stdout).unwrap();

    assert!(
        out.contains("RESUMING"),
        "expected RESUMING in output: {out}"
    );
    assert!(
        out.contains("Next Steps:"),
        "expected Next Steps: in output: {out}"
    );
    assert!(
        out.contains("Finish src/state.rs"),
        "expected step 1 in output: {out}"
    );
    assert!(
        !out.contains("• Handoff:"),
        "normal Handoff: line should be suppressed: {out}"
    );
}

#[test]
fn prime_resuming_signal_not_consumed_on_second_call() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let today = Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("{today}-session-end.md");
    let content = format!(
        "---\ndate: {today}\nproject: myproject\nphase: implement\n---\n\nDoing work.\n\n## Next Steps\n\n1. Next thing\n"
    );
    write_handoff(tmp.path(), "myproject", &filename, &content);
    write_pending_resume(tmp.path(), "myproject", &format!("handoffs/{filename}"));

    // First call
    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("RESUMING"));

    // Second call — signal must not have been deleted
    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("RESUMING"));
}

#[test]
fn prime_ignores_stale_pending_resume_dated_yesterday() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // Handoff with yesterday's date
    let yesterday = (Local::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let filename = format!("{yesterday}-session-end.md");
    let content = format!(
        "---\ndate: {yesterday}\nproject: myproject\nphase: implement\n---\n\nOld work.\n\n## Next Steps\n\n1. Old step\n"
    );
    write_handoff(tmp.path(), "myproject", &filename, &content);
    write_pending_resume(tmp.path(), "myproject", &format!("handoffs/{filename}"));

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("RESUMING")));
    assert!(stdout.contains("Handoff:"));
}

#[test]
fn prime_renders_normally_without_pending_resume() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // A normal handoff from yesterday — no .pending-resume
    write_handoff(
        tmp.path(),
        "myproject",
        "2026-02-23-session-end.md",
        "---\ndate: 2026-02-23\nproject: myproject\nphase: research\n---\n\nCompleted research.\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("RESUMING")));
    assert!(stdout.contains("Handoff: 2026-02-23"));
}

#[test]
fn prime_resuming_empty_next_steps_shows_only_header() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let today = Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("{today}-session-end.md");
    // ## Next Steps section exists but only has HTML comments
    let content = format!(
        "---\ndate: {today}\nproject: myproject\nphase: implement\n---\n\nDoing work.\n\n\
         ## Next Steps\n\n<!-- TODO: fill this in -->\n"
    );
    write_handoff(tmp.path(), "myproject", &filename, &content);
    write_pending_resume(tmp.path(), "myproject", &format!("handoffs/{filename}"));

    let stdout = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    assert!(stdout.status.success());
    let stdout = stdout.stdout.clone();
    let out = String::from_utf8(stdout).unwrap();

    assert!(out.contains("RESUMING"), "expected RESUMING header: {out}");
    assert!(
        !out.contains("Next Steps:"),
        "no Next Steps: label when section is empty: {out}"
    );
}

#[test]
fn prime_close_prime_close_prime_end_to_end_resume_loop() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // First close
    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let proj_dir = tmp.path().join(".wai/projects/myproject");
    let signal1 = fs::read_to_string(proj_dir.join(".pending-resume")).unwrap();
    assert!(
        !signal1.trim().is_empty(),
        ".pending-resume written after first close"
    );

    // prime should show RESUMING (handoff dated today by wai close)
    let out1 = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    assert!(out1.status.success());
    let out1 = out1.stdout.clone();
    assert!(
        String::from_utf8(out1).unwrap().contains("RESUMING"),
        "first prime after close should show RESUMING"
    );

    // Second close
    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let signal2 = fs::read_to_string(proj_dir.join(".pending-resume")).unwrap();
    // Second close updates the same file in place, so .pending-resume still
    // points to the same (only) handoff for this day.
    assert_eq!(
        signal1.trim(),
        signal2.trim(),
        "second close should keep .pending-resume pointing to the same handoff"
    );
    assert!(
        !signal2.trim().is_empty(),
        ".pending-resume should still be non-empty after second close"
    );

    // second prime should show RESUMING pointing to the updated handoff
    let out2 = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    assert!(out2.status.success());
    let out2 = out2.stdout.clone();
    assert!(
        String::from_utf8(out2).unwrap().contains("RESUMING"),
        "second prime after close should still show RESUMING"
    );
}

#[test]
fn ls_single_workspace_one_project() {
    let root = TempDir::new().unwrap();
    let ws = root.path().join("my-repo");
    make_workspace(&ws, "my-ws");
    make_project(&ws, "my-proj", Some("implement"));

    let out = wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-proj"));
    assert!(stdout.contains("implement"));
}

#[test]
fn ls_multiple_workspaces_sorted_by_name() {
    let root = TempDir::new().unwrap();

    let ws_a = root.path().join("alpha-repo");
    make_workspace(&ws_a, "alpha");
    make_project(&ws_a, "zebra", Some("plan"));

    let ws_b = root.path().join("beta-repo");
    make_workspace(&ws_b, "beta");
    make_project(&ws_b, "apple", Some("research"));

    let out = wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();

    let output = String::from_utf8_lossy(&out);
    // Both projects appear
    assert!(output.contains("apple"), "apple missing: {}", output);
    assert!(output.contains("zebra"), "zebra missing: {}", output);

    // apple (a) appears before zebra (z) when sorted
    let pos_apple = output.find("apple").unwrap();
    let pos_zebra = output.find("zebra").unwrap();
    assert!(
        pos_apple < pos_zebra,
        "projects not sorted: apple at {} vs zebra at {}",
        pos_apple,
        pos_zebra
    );
}

#[test]
fn ls_no_workspaces_prints_message() {
    let root = TempDir::new().unwrap();
    // No workspaces — plain empty directory

    let out = wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No wai workspaces found"));
}

#[test]
fn ls_no_beads_omits_counts_column() {
    let root = TempDir::new().unwrap();
    let ws = root.path().join("proj");
    make_workspace(&ws, "my-ws");
    make_project(&ws, "my-proj", Some("review"));
    // No .beads/ directory

    let out = wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-proj"));
    assert!(!(stdout.contains("open")));
    assert!(!(stdout.contains("ready")));
}

#[test]
fn ls_beads_present_but_unavailable_is_graceful() {
    // When .beads/ exists but `bd` is not available (or returns no data),
    // the command must still succeed without crashing.
    let root = TempDir::new().unwrap();
    let ws = root.path().join("proj");
    make_workspace(&ws, "my-ws");
    make_project(&ws, "my-proj", Some("implement"));
    // Create .beads/ dir to trigger beads detection
    fs::create_dir_all(ws.join(".beads")).unwrap();

    // Use an empty PATH so 'bd' cannot be found — graceful skip
    let out = wai_cmd(root.path())
        .args(["ls", "--root", root.path().to_str().unwrap()])
        .env("PATH", "")
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-proj"));
}

#[test]
fn ls_depth_limits_recursion() {
    let root = TempDir::new().unwrap();

    // Workspace at depth 1 (root/shallow-repo)
    let shallow = root.path().join("shallow-repo");
    make_workspace(&shallow, "shallow");
    make_project(&shallow, "shallow-proj", Some("plan"));

    // Workspace at depth 2 (root/dir/deep-repo) — only found with depth >= 2
    let deep = root.path().join("dir").join("deep-repo");
    make_workspace(&deep, "deep");
    make_project(&deep, "deep-proj", Some("research"));

    // With --depth 1, only the depth-1 workspace is found
    let out = wai_cmd(root.path())
        .args([
            "ls",
            "--root",
            root.path().to_str().unwrap(),
            "--depth",
            "1",
        ])
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("shallow-proj"));
    assert!(!(stdout.contains("deep-proj")));

    // With --depth 2, both are found
    let out = wai_cmd(root.path())
        .args([
            "ls",
            "--root",
            root.path().to_str().unwrap(),
            "--depth",
            "2",
        ])
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("shallow-proj"));
    assert!(stdout.contains("deep-proj"));
}

#[test]
fn ls_invalid_root_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["ls", "--root", "/nonexistent-path-that-does-not-exist"])
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("/nonexistent-path-that-does-not-exist"));
}

#[test]
fn reflect_with_mock_llm_writes_claude_md_and_reflect_meta() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    // Write a research artifact so context gathering has something.
    write_artifact(tmp.path(), "test-proj", "research", "r.md", "some research");

    let out = wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--yes"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Wrote"));

    // A reflection resource file should be created in .wai/resources/reflections/.
    let refl_dir = tmp.path().join(".wai/resources/reflections");
    assert!(refl_dir.exists(), "reflections dir should exist");
    let entries: Vec<_> = fs::read_dir(&refl_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "expected exactly one reflection resource file"
    );
    let resource_content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        resource_content.contains("Use TDD always"),
        "resource file should contain reflect content"
    );

    // .reflect-meta should be created.
    let meta_path = tmp.path().join(".wai/projects/test-proj/.reflect-meta");
    assert!(meta_path.exists(), ".reflect-meta should be created");
}

#[test]
fn reflect_dry_run_does_not_modify_claude_md() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    let original = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--dry-run"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Dry run"));

    let after = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert_eq!(original, after, "CLAUDE.md must not change in dry-run");
}

#[test]
fn reflect_repeated_call_updates_existing_resource_file() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    // Run reflect twice with the same mock content.
    for _ in 0..2 {
        let out = wai_cmd(tmp.path())
            .args([
                "reflect",
                "--project",
                "test-proj",
                "--output",
                "claude.md",
                "--yes",
            ])
            .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
            .env("NO_COLOR", "1")
            .output()
            .expect("command should run");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success());
        assert!(stdout.contains("Wrote"));
    }

    // Only one resource file should exist (idempotent — second call updates in place).
    let refl_dir = tmp.path().join(".wai/resources/reflections");
    let entries: Vec<_> = fs::read_dir(&refl_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "expected exactly one reflection file after two reflect calls, got: {:?}",
        entries.iter().map(|e| e.file_name()).collect::<Vec<_>>()
    );
}

#[test]
fn reflect_diff_shown_for_existing_reflect_block() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    // Pre-populate with old content.
    let old_block = "<!-- WAI:REFLECT:START -->\n## Old content\n<!-- WAI:REFLECT:END -->\n";
    fs::write(
        tmp.path().join("CLAUDE.md"),
        format!("# Claude\n{}", old_block),
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["reflect", "--project", "test-proj", "--dry-run"])
        .env("WAI_REFLECT_MOCK_RESPONSE", MOCK_REFLECT_CONTENT)
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Dry run"));
}

#[test]
fn close_nudge_fires_at_five_plus_handoffs() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    let handoffs_dir = tmp.path().join(".wai/projects/test-proj/handoffs");
    fs::create_dir_all(&handoffs_dir).unwrap();

    // Write 5 handoff files.
    for i in 1..=5 {
        fs::write(
            handoffs_dir.join(format!("2026-02-{:02}-session.md", i)),
            "# Session Handoff\n## What Was Done\nWork.",
        )
        .unwrap();
    }

    // wai close should produce the nudge.
    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "test-proj"])
        .env("NO_COLOR", "1")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("sessions since last reflect"));
}

#[test]
fn close_nudge_does_not_fire_for_fewer_than_five_handoffs() {
    let tmp = TempDir::new().unwrap();
    reflect_workspace(tmp.path());

    let handoffs_dir = tmp.path().join(".wai/projects/test-proj/handoffs");
    fs::create_dir_all(&handoffs_dir).unwrap();

    // Write only 3 handoff files — below the threshold.
    for i in 1..=3 {
        fs::write(
            handoffs_dir.join(format!("2026-02-{:02}-session.md", i)),
            "# Session Handoff\n",
        )
        .unwrap();
    }

    let output = wai_cmd(tmp.path())
        .args(["close", "--project", "test-proj"])
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("sessions since last reflect"),
        "nudge should not appear with only 3 handoffs"
    );
}
