#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn close_zero_projects_shows_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["close", "--no-input"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("No projects found"));
}

#[test]
fn close_repeated_same_day_updates_existing() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // First invocation
    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Second invocation on same day
    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let handoffs_dir = tmp.path().join(".wai/projects/myproject/handoffs");
    let files: Vec<_> = fs::read_dir(&handoffs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    assert_eq!(
        files.len(),
        1,
        "expected only 1 handoff file after two same-day runs, got: {:?}",
        files
    );
    assert!(
        files[0].ends_with("session-end.md"),
        "expected a session-end.md file, got: {:?}",
        files
    );
}

#[test]
fn close_writes_pending_resume_signal() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let proj_dir = tmp.path().join(".wai/projects/myproject");
    let signal = proj_dir.join(".pending-resume");
    assert!(
        signal.exists(),
        ".pending-resume should be written after close"
    );

    let content = fs::read_to_string(&signal).unwrap();
    let relative = content.trim();
    // Should point to a file under handoffs/
    assert!(
        relative.starts_with("handoffs/"),
        ".pending-resume should contain a handoffs/ relative path, got: {relative}"
    );
    // The referenced file should actually exist
    assert!(
        proj_dir.join(relative).exists(),
        "handoff referenced by .pending-resume should exist at {relative}"
    );
}

#[test]
fn close_overwrites_pending_resume_on_second_call() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let proj_dir = tmp.path().join(".wai/projects/myproject");
    let first = fs::read_to_string(proj_dir.join(".pending-resume")).unwrap();

    // Second close on the same day updates the same file in place.
    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "myproject"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let second = fs::read_to_string(proj_dir.join(".pending-resume")).unwrap();
    // Both runs point to the same (only) handoff file for this day.
    assert_eq!(
        first.trim(),
        second.trim(),
        ".pending-resume should point to the same (updated) handoff file"
    );
    // The path should exist.
    assert!(
        proj_dir.join(second.trim()).exists(),
        ".pending-resume path should exist"
    );
}

#[test]
fn prime_single_project_with_handoff() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    write_handoff(
        tmp.path(),
        "myproject",
        "2026-02-23-session-end.md",
        "---\ndate: 2026-02-23\nproject: myproject\nphase: research\n---\n\n# Session Handoff\n\nCompleted the initial research phase.\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("wai prime"));
    assert!(stdout.contains("Project: myproject"));
    assert!(stdout.contains("Handoff: 2026-02-23"));
    assert!(stdout.contains("Completed the initial research phase."));
}

#[test]
fn prime_no_handoff_omits_handoff_line() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project: myproject"));

    assert!(!stdout.contains("Handoff:"));
}

#[test]
fn search_with_project_filter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "app-a");
    create_project(tmp.path(), "app-b");
    write_artifact(
        tmp.path(),
        "app-a",
        "research",
        "2026-01-15-a.md",
        "unique_a\n",
    );
    write_artifact(
        tmp.path(),
        "app-b",
        "research",
        "2026-01-15-b.md",
        "unique_b\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "unique_a", "--in", "app-a"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("unique_a"));

    let out = wai_cmd(tmp.path())
        .args(["search", "unique_b", "--in", "app-a"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No results found"));
}

#[test]
fn search_with_regex() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "error_code: 404\nerror_code: 500\nsuccess_code: 200\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "error_code: \\d+", "--regex"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("2 matches"));
    assert!(stdout.contains("404"));
    assert!(stdout.contains("500"));
}

#[test]
fn search_with_limit() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "line one match\nline two match\nline three match\nline four match\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "match", "-n", "2"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stdout.contains("4 matches"));
    assert!(stderr.contains("Showing first 2 of 4"));
}

#[test]
fn search_invalid_regex_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["search", "[invalid", "--regex"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("Invalid regex"));
}

#[test]
fn search_include_memories_appends_memories_section() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-15-notes.md",
        "shared-term in artifact\n",
    );
    fs::create_dir_all(tmp.path().join(".beads")).unwrap();
    let fake_bin = install_fake_bd(
        tmp.path(),
        "shared-term",
        &["memory-key: remembered detail from bd memories"],
    );
    let path = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "shared-term", "--include-memories"])
        .env("PATH", path)
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Search results"));
    assert!(stdout.contains("Memories"));
    assert!(stdout.contains("[mem]"));
    assert!(stdout.contains("memory-key"));
}

#[test]
fn search_include_memories_shows_memory_only_results() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    fs::create_dir_all(tmp.path().join(".beads")).unwrap();
    let fake_bin = install_fake_bd(
        tmp.path(),
        "memory-only",
        &["memory-only-key: remembered detail with no artifact hit"],
    );
    let path = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = wai_cmd(tmp.path())
        .args(["search", "memory-only", "--include-memories"])
        .env("PATH", path)
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No artifact results found"));
    assert!(stdout.contains("Memories"));
    assert!(stdout.contains("memory-only-key"));
}

#[test]
fn search_include_memories_verbose_shows_full_value() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    fs::create_dir_all(tmp.path().join(".beads")).unwrap();
    let long_value = "This memory value is intentionally long so search should truncate it by default but show the full value when verbose is enabled.";
    let fake_bin = install_fake_bd(
        tmp.path(),
        "verbose-memory",
        &[&format!("verbose-key: {long_value}")],
    );
    let path = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = wai_cmd(tmp.path())
        .args(["-v", "search", "verbose-memory", "--include-memories"])
        .env("PATH", path)
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains(long_value));
}

// ─── add plan/design with --tags ─────────────────────────────────────────────

#[test]
fn add_plan_with_tags_includes_frontmatter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "plan",
            "My plan content",
            "--project",
            "my-app",
            "--tags",
            "backend,api",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let plans_dir = tmp.path().join(".wai/projects/my-app/plans");
    let files: Vec<_> = fs::read_dir(&plans_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.contains("---"), "plan should have frontmatter");
    assert!(content.contains("tags:"), "plan should have tags key");
    assert!(content.contains("backend"), "plan should have backend tag");
    assert!(content.contains("api"), "plan should have api tag");
    assert!(
        content.contains("My plan content"),
        "plan body should be present"
    );
}

#[test]
fn add_design_with_tags_includes_frontmatter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "design",
            "My design content",
            "--project",
            "my-app",
            "--tags",
            "ux",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let designs_dir = tmp.path().join(".wai/projects/my-app/designs");
    let files: Vec<_> = fs::read_dir(&designs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.contains("---"), "design should have frontmatter");
    assert!(content.contains("ux"), "design should have ux tag");
}

// ─── search --tag and --latest ────────────────────────────────────────────────

#[test]
fn search_tag_filter_returns_only_tagged_files() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Write one tagged and one untagged research file.
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-tagged.md",
        "---\ntags: [rust, perf]\n---\n\nfoo bar baz",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-11-untagged.md",
        "foo bar baz",
    );

    let output = wai_cmd(tmp.path())
        .args(["search", "foo", "--tag", "rust"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("tagged"), "should match the tagged file");
    assert!(
        !stdout.contains("untagged"),
        "should not match the untagged file"
    );
}

#[test]
fn search_tag_filter_no_match_returns_no_results() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-file.md",
        "---\ntags: [rust]\n---\n\ncontent",
    );

    let output = wai_cmd(tmp.path())
        .args(["search", "content", "--tag", "nonexistent-tag"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No results"), "should report no results");
}

#[test]
fn search_malformed_frontmatter_does_not_abort_tag_search() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-bad.md",
        "---\n[[ invalid\n---\ncontent",
    );

    // Should succeed (not panic / not fail), even with malformed frontmatter.
    let out = wai_cmd(tmp.path())
        .args(["search", "content", "--tag", "anything"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

#[test]
fn search_latest_returns_only_most_recent_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-older.md",
        "needle content",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-20-newer.md",
        "needle content",
    );

    let output = wai_cmd(tmp.path())
        .args(["search", "needle", "--latest"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("newer"), "should contain the newer file");
    assert!(
        !stdout.contains("older"),
        "should not contain the older file"
    );
}

#[test]
fn search_tag_and_type_and_latest_combined() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-01-r1.md",
        "---\ntags: [perf]\n---\n\ndata",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-01-r2.md",
        "---\ntags: [perf]\n---\n\ndata",
    );
    // plan with same tag — should be excluded by --type research
    write_artifact(
        tmp.path(),
        "my-app",
        "plans",
        "2026-03-01-p1.md",
        "---\ntags: [perf]\n---\n\ndata",
    );

    let output = wai_cmd(tmp.path())
        .args([
            "search", "data", "--tag", "perf", "--type", "research", "--latest",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("r2"),
        "should match the latest research file"
    );
    assert!(!stdout.contains("r1"), "should exclude older research file");
    assert!(!stdout.contains("p1"), "should exclude the plan file");
}

// ─── wai timeline ───────────────────────────────────────────────────────────

#[test]
fn timeline_shows_dated_artifacts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-first.md",
        "First\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "plans",
        "2026-01-20-second.md",
        "Second\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["timeline", "my-app"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Timeline for"));
    assert!(stdout.contains("2026-01-20"));
    assert!(stdout.contains("2026-01-10"));
}

#[test]
fn timeline_empty_project_shows_message() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    let out = wai_cmd(tmp.path())
        .args(["timeline", "my-app"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No dated artifacts found"));
}

#[test]
fn timeline_nonexistent_project_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["timeline", "ghost"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}

#[test]
fn timeline_from_filter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-05-old.md",
        "Old\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-15-new.md",
        "New\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--from", "2026-02-01"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("2026-02-15"));
    assert!(!(stdout.contains("2026-01-05")));
}

#[test]
fn timeline_to_filter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-05-old.md",
        "Old\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-15-new.md",
        "New\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--to", "2026-01-31"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("2026-01-05"));
    assert!(!(stdout.contains("2026-02-15")));
}

#[test]
fn timeline_from_to_range() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-01-jan.md",
        "Jan\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-02-01-feb.md",
        "Feb\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-03-01-mar.md",
        "Mar\n",
    );

    let out = wai_cmd(tmp.path())
        .args([
            "timeline",
            "my-app",
            "--from",
            "2026-01-15",
            "--to",
            "2026-02-15",
        ])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("2026-02-01"));
    assert!(!(stdout.contains("2026-01-01")));
    assert!(!(stdout.contains("2026-03-01")));
}

#[test]
fn timeline_reverse_shows_oldest_first() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-first.md",
        "First\n",
    );
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-03-20-third.md",
        "Third\n",
    );

    let output = wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--reverse"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let pos_jan = stdout.find("2026-01-10").unwrap();
    let pos_mar = stdout.find("2026-03-20").unwrap();
    assert!(
        pos_jan < pos_mar,
        "In reverse mode, oldest date should appear first"
    );
}

#[test]
fn timeline_json_outputs_entries() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    write_artifact(
        tmp.path(),
        "my-app",
        "research",
        "2026-01-10-first.md",
        "First\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["timeline", "my-app", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"entries\""));
}
