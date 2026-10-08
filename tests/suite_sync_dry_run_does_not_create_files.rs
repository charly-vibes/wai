#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use flate2::{Compression, write::GzEncoder};
use std::fs;
use tar::Header;
use tempfile::TempDir;

// ── wai sync --dry-run ────────────────────────────────────────────────────────

#[test]
fn sync_dry_run_does_not_create_files() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Set up a source file to sync.
    let source_dir = tmp.path().join(".wai/resources/agent-config/docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("guide.md"), "# Guide").unwrap();

    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: GUIDE.md\n    strategy: inline\n    sources: [docs]\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["sync", "--dry-run"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // The target must not have been created.
    assert!(
        !tmp.path().join("GUIDE.md").exists(),
        "dry-run must not create any files"
    );
}

// ─── wai pipeline ─────────────────────────────────────────────────────────────

#[test]
fn pipeline_list_empty() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "list"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No pipelines defined"));
}

#[test]
fn pipeline_add_auto_tags_with_pipeline_run_env() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // Add research with WAI_PIPELINE_RUN set
    let out = wai_cmd(tmp.path())
        .env("WAI_PIPELINE_RUN", "my-pipe-2026-02-25-feature")
        .args(["add", "research", "test content for pipeline tagging"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Find the created research file and verify it has the pipeline-run tag
    let research_dir = tmp.path().join(".wai/projects/myproject/research");
    let entries: Vec<_> = fs::read_dir(&research_dir).unwrap().flatten().collect();
    assert_eq!(entries.len(), 1, "Expected exactly one research file");

    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.contains("pipeline-run:my-pipe-2026-02-25-feature"),
        "Expected pipeline-run tag in content: {}",
        content
    );
}

#[test]
fn pipeline_add_merges_user_tags_and_pipeline_run_tag() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .env("WAI_PIPELINE_RUN", "test-run-id")
        .args([
            "add",
            "research",
            "--tags=manual-tag",
            "content with both tags",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let research_dir = tmp.path().join(".wai/projects/myproject/research");
    let entries: Vec<_> = fs::read_dir(&research_dir).unwrap().flatten().collect();
    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.contains("manual-tag"),
        "Expected manual-tag in: {}",
        content
    );
    assert!(
        content.contains("pipeline-run:test-run-id"),
        "Expected pipeline-run tag in: {}",
        content
    );
}

// ─── wai pipeline init ────────────────────────────────────────────────────────

#[test]
fn pipeline_help_lists_builtin_templates() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "--help", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Built-in templates"));
    assert!(stdout.contains("scientific-research"));
    assert!(stdout.contains("tdd-ro5"));
}

#[test]
fn pipeline_init_help_lists_builtin_templates() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "--help", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Built-in templates"));
    assert!(stdout.contains("scientific-research"));
    assert!(stdout.contains("tdd-ro5"));
}

#[test]
fn resource_list_skills_bad_frontmatter_shows_no_metadata() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a skill directory with a SKILL.md that has no valid frontmatter
    let skill_dir = tmp
        .path()
        .join(".wai/resources/agent-config/skills/broken-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        "# Just a heading, no frontmatter",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["resource", "list", "skills"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("no metadata"));
}

#[test]
fn resource_list_skills_json_has_skills_key() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "list", "skills", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"skills\""));
}

// ── wai resource install ─────────────────────────────────────────────────────

#[test]
fn resource_install_global_copies_skill_to_home_library() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "global-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/global-skill/SKILL.md");
    fs::write(
        &skill_md,
        "---\nname: global-skill\ndescription: Install me globally\n---\n\n# Global Skill\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["resource", "install", "global-skill", "--global"])
        .env("HOME", fake_home.path())
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Installed 'global-skill' globally"));

    assert!(
        fake_home
            .path()
            .join(".wai/resources/skills/global-skill/SKILL.md")
            .is_file()
    );
}

#[test]
fn resource_install_from_repo_copies_skill_into_current_project() {
    let dst = TempDir::new().unwrap();
    let src = TempDir::new().unwrap();
    init_workspace(dst.path());
    init_workspace(src.path());

    let out = wai_cmd(src.path())
        .args(["resource", "add", "skill", "repo-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let src_skill = src
        .path()
        .join(".wai/resources/agent-config/skills/repo-skill/SKILL.md");
    fs::write(
        &src_skill,
        "---\nname: repo-skill\ndescription: Install from repo\n---\n\n# Repo Skill\n",
    )
    .unwrap();

    let out = wai_cmd(dst.path())
        .args([
            "resource",
            "install",
            "repo-skill",
            "--from-repo",
            src.path().to_str().unwrap(),
        ])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Installed 'repo-skill' from"));

    assert!(
        dst.path()
            .join(".wai/resources/agent-config/skills/repo-skill/SKILL.md")
            .is_file()
    );
}

#[test]
fn resource_install_global_fails_when_skill_missing() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "install", "missing-skill", "--global"])
        .env("HOME", fake_home.path())
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found in current project"));
}

// ── wai resource import skills ───────────────────────────────────────────────

#[test]
fn resource_import_skills_from_custom_path() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a skill in a custom source directory
    let source = tmp.path().join("ext-skills/cool-skill");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: cool-skill\ndescription: An imported skill\n---\n\n# Cool Skill\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "import",
            "skills",
            "--from",
            tmp.path().join("ext-skills").to_str().unwrap(),
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills/cool-skill")
            .is_dir(),
        "imported skill directory should exist"
    );
}

#[test]
fn resource_import_skills_from_default_agents_path() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a skill in the default .agents/skills/ location
    let source = tmp.path().join(".agents/skills/default-skill");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: default-skill\ndescription: From default path\n---\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["resource", "import", "skills"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills/default-skill")
            .is_dir()
    );
}

#[test]
fn resource_import_skills_skips_existing_with_warning() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Add a skill that already exists in the workspace
    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "existing-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Create source with the same skill name
    let source = tmp.path().join("source/existing-skill");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: existing-skill\ndescription: Would conflict\n---\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "import",
            "skills",
            "--from",
            tmp.path().join("source").to_str().unwrap(),
        ])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!((stderr.contains("Skipped") || stderr.contains("already exists")));
}

#[test]
fn resource_import_skills_copies_full_directory_tree() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a skill with nested files (not just SKILL.md)
    let source = tmp.path().join("source/rich-skill");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: rich-skill\ndescription: A rich skill\n---\n",
    )
    .unwrap();
    fs::write(source.join("examples.md"), "# Examples").unwrap();
    fs::create_dir_all(source.join("templates")).unwrap();
    fs::write(source.join("templates/base.md"), "# Base Template").unwrap();

    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "import",
            "skills",
            "--from",
            tmp.path().join("source").to_str().unwrap(),
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let target = tmp
        .path()
        .join(".wai/resources/agent-config/skills/rich-skill");
    assert!(target.join("SKILL.md").is_file());
    assert!(
        target.join("examples.md").is_file(),
        "extra files should be copied"
    );
    assert!(
        target.join("templates/base.md").is_file(),
        "subdirectory contents should be copied"
    );
}

#[test]
fn resource_export_then_import_archive_round_trips_skill() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "round-trip-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/round-trip-skill/SKILL.md");
    let content = "---\nname: round-trip-skill\ndescription: Round trip\n---\n\n# Round Trip\n";
    fs::write(&skill_md, content).unwrap();

    let archive = tmp.path().join("skills.tar.gz");
    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "export",
            "round-trip-skill",
            "--output",
            archive.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    fs::remove_dir_all(
        tmp.path()
            .join(".wai/resources/agent-config/skills/round-trip-skill"),
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "import",
            "archive",
            archive.to_str().unwrap(),
            "--yes",
        ])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Imported 1 new skill"));

    assert_eq!(fs::read_to_string(skill_md).unwrap(), content);
}

#[test]
fn resource_import_archive_rejects_malformed_entry_path() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let archive = tmp.path().join("bad-skills.tar.gz");
    let out = fs::File::create(&archive).unwrap();
    let gz = GzEncoder::new(out, Compression::default());
    let mut builder = tar::Builder::new(gz);

    let payload = b"bad skill";
    let mut header = Header::new_gnu();
    header.set_size(payload.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, "evil.txt", &payload[..])
        .unwrap();
    builder.finish().unwrap();
    let gz = builder.into_inner().unwrap();
    gz.finish().unwrap();

    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "import",
            "archive",
            archive.to_str().unwrap(),
            "--yes",
        ])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(
        (stderr.contains("Invalid archive entry")
            || stderr.contains("entries must end with '/SKILL.md'"))
    );
}

// ── Doctor projection consistency ────────────────────────────────────────────

#[test]
fn doctor_warns_on_missing_projection_source_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Configure a projection referencing a source that doesn't exist
    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: AGENTS.md\n    strategy: inline\n    sources: [nonexistent-dir]\n",
    );

    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("\"status\": \"warn\""),
        "doctor should warn about missing source, got: {}",
        stdout
    );
}

#[test]
fn doctor_detects_stale_inline_projection() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a source directory with a file
    let source_dir = tmp.path().join(".wai/resources/agent-config/my-docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("intro.md"), "# Original").unwrap();

    // Configure and sync an inline projection
    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: AGENTS.md\n    strategy: inline\n    sources: [my-docs]\n",
    );
    let out = wai_cmd(tmp.path())
        .args(["sync"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Modify the source to make the projection stale
    fs::write(source_dir.join("intro.md"), "# Modified content").unwrap();

    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Stale") || stdout.contains("\"status\": \"warn\""),
        "doctor should detect stale inline projection, got: {}",
        stdout
    );
}

#[cfg(unix)]
#[test]
fn doctor_passes_for_in_sync_inline_projection() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create source and sync
    let source_dir = tmp.path().join(".wai/resources/agent-config/sync-docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("notes.md"), "# Notes").unwrap();

    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: AGENTS.md\n    strategy: inline\n    sources: [sync-docs]\n",
    );
    let out = wai_cmd(tmp.path())
        .args(["sync"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Doctor should see the projection as in-sync
    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The projection check should pass (or at least not add a new warn/fail)
    assert!(
        stdout.contains("In sync") || !stdout.contains("Stale"),
        "doctor should see synced projection as passing, got: {}",
        stdout
    );
}

// ─── wai-7gk: Interactive Ambiguity Resolution ───────────────────────────────

#[test]
fn add_research_no_input_fails_when_multiple_projects() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .args(["--no-input", "add", "research", "some notes"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("Multiple projects") || stderr.contains("--project")));
}

#[test]
fn add_research_explicit_project_works_with_multiple() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .args(["add", "research", "--project", "alpha", "targeting alpha"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Verify the artifact landed in the right project
    let research_dir = tmp.path().join(".wai/projects/alpha/research");
    let entries: Vec<_> = fs::read_dir(research_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !entries.is_empty(),
        "research artifact should be in alpha project"
    );
}

#[test]
fn add_research_single_project_still_works_without_flag() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "solo");

    let out = wai_cmd(tmp.path())
        .args(["add", "research", "just one project"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

// ─── wai-rsp: Context Suggestions Testing ────────────────────────────────────

#[test]
fn add_research_shows_phase_appropriate_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "suggest-proj");

    // Add enough research to trigger the "advance to design" suggestion (≥2 artifacts)
    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "--project",
            "suggest-proj",
            "first finding",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "--project",
            "suggest-proj",
            "second finding",
        ])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(
        ((stdout.contains("Add more research") || stdout.contains("Move to design"))
            || stdout.contains("wai phase"))
    );
}

#[test]
fn add_research_in_non_research_phase_shows_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "impl-proj");

    // Advance to implement phase (phase command picks the only project automatically)
    let out = wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // In non-research phases, adding research still shows next-step suggestions
    let out = wai_cmd(tmp.path())
        .args(["add", "research", "--project", "impl-proj", "context note"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!((stdout.contains("wai add research") || stdout.contains("wai search")));
}

// ─── wai close ───────────────────────────────────────────────────────────────

#[test]
fn close_single_project_creates_handoff() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // Create .beads dir so the beads plugin is detected
    fs::create_dir(tmp.path().join(".beads")).unwrap();

    let output = wai_cmd(tmp.path())
        .args(["close"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout.clone()).unwrap();

    // Handoff file should have been created
    let handoffs_dir = tmp.path().join(".wai/projects/myproject/handoffs");
    let files: Vec<_> = fs::read_dir(&handoffs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);

    // Output should contain the handoff path line
    assert!(
        stdout.contains("Handoff created:"),
        "expected 'Handoff created:' in stdout, got: {stdout}"
    );
    // Output should contain next-steps plus a generic beads hint.
    assert!(
        stdout.contains("→ Next:"),
        "expected '→ Next:' in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("→ Beads:"),
        "expected '→ Beads:' in stdout, got: {stdout}"
    );
    assert!(
        !stdout.contains("bd sync --from-main"),
        "did not expect stale 'bd sync --from-main' in stdout, got: {stdout}"
    );
}

#[test]
fn close_with_project_flag_skips_prompt() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "alpha"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Handoff created:"));

    // Only alpha should have a handoff
    let alpha_handoffs = tmp.path().join(".wai/projects/alpha/handoffs");
    let beta_handoffs = tmp.path().join(".wai/projects/beta/handoffs");
    let alpha_files: Vec<_> = fs::read_dir(&alpha_handoffs)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let beta_files: Vec<_> = fs::read_dir(&beta_handoffs)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(alpha_files.len(), 1);
    assert_eq!(beta_files.len(), 0);
}

#[test]
fn close_unknown_project_shows_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["close", "--project", "nonexistent"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("nonexistent"));
}
