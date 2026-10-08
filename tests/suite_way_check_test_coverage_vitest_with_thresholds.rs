#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn way_check_test_coverage_vitest_with_thresholds() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("vitest.config.ts"),
        "export default { test: { coverage: { thresholds: { lines: 80 } } } }\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Test coverage"));
    assert!(stdout.contains("vitest coverage configured (threshold enforced)"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_test_coverage_not_configured() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Test coverage"));
    assert!(stdout.contains("No coverage tool configured"));
    assert!(stdout.contains("\"warn\""));
}

// ─── way: check_beads ─────────────────────────────────────────────────────────

#[test]
fn way_check_beads_present() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".beads")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Issue tracking"));
    assert!(stdout.contains("beads detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_beads_absent() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Issue tracking"));
    assert!(stdout.contains("\"warn\""));
}

// ─── way: check_openspec ──────────────────────────────────────────────────────

#[test]
fn way_check_openspec_present() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("openspec")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Change proposals"));
    assert!(stdout.contains("openspec detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_openspec_absent() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Change proposals"));
    assert!(stdout.contains("\"warn\""));
}

// ─── Post-Command Suggestions ────────────────────────────────────────────────

#[test]
fn new_project_shows_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["new", "project", "test-proj"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Add research: wai add research"));
    assert!(stdout.contains("Check project phase: wai phase"));
    assert!(stdout.contains("Check status: wai status"));
}

#[test]
fn add_research_shows_context_aware_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-proj");

    // First research artifact - should suggest adding more research
    let out = wai_cmd(tmp.path())
        .args(["add", "research", "Initial research"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Add more research: wai add research"));
    assert!(stdout.contains("Check phase: wai phase"));

    // Second research artifact - should suggest moving to design phase
    let out = wai_cmd(tmp.path())
        .args(["add", "research", "More detailed findings"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Add more research: wai add research"));
    assert!(stdout.contains("Move to design phase: wai phase set design"));
    assert!(stdout.contains("Review research: wai search"));
}

#[test]
fn phase_next_shows_phase_specific_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-proj");

    // Advance from research to design - should show design suggestions
    let out = wai_cmd(tmp.path())
        .args(["phase", "next"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Add design: wai add design"));
    assert!(stdout.contains("Review research: wai search"));
    assert!(stdout.contains("Show project details: wai show"));
}

#[test]
fn phase_set_shows_phase_specific_suggestions() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-proj");

    // Set to implement phase - should show implement suggestions
    let out = wai_cmd(tmp.path())
        .args(["phase", "set", "implement"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Show project details: wai show"));
    assert!(stdout.contains("Add implementation notes: wai add plan"));
    assert!(stdout.contains("Check status: wai status"));
}

// ─── Context Inference ───────────────────────────────────────────────────────

#[test]
fn typo_suggestion_shown_outside_workspace() {
    // Outside any workspace, a typo should still show "Did you mean?" rather
    // than just "No project initialized".
    let tmp = TempDir::new().unwrap();
    // NOT calling init_workspace — this dir has no .wai/

    let out = wai_cmd(tmp.path())
        .args(["statu"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("Did you mean 'status'") || stderr.contains("did you mean 'status'")));
}

#[test]
fn wrong_order_shown_outside_workspace() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["project", "new"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("new project"));
    assert!(stderr.contains("Did you mean"));
}

#[test]
fn wrong_order_project_new_suggests_correction() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // "wai project new" should suggest "wai new project"
    let out = wai_cmd(tmp.path())
        .args(["project", "new"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("new project"));
    assert!(stderr.contains("Did you mean"));
}

#[test]
fn wrong_order_research_add_suggests_correction() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // "wai research add" should suggest "wai add research"
    let out = wai_cmd(tmp.path())
        .args(["research", "add"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("add research"));
    assert!(stderr.contains("Did you mean"));
}

// ─── Typo Detection ───────────────────────────────────────────────────────────

#[test]
fn typo_suggests_closest_command() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // "statu" is a typo of "status"
    let out = wai_cmd(tmp.path())
        .args(["statu"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("Did you mean 'status'") || stderr.contains("did you mean 'status'")));
}

#[test]
fn typo_suggests_doctor_for_doctr() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["doctr"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("Did you mean 'doctor'") || stderr.contains("did you mean 'doctor'")));
}

#[test]
fn completely_unknown_command_shows_help_hint() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // "xyz" is not similar to any command
    let out = wai_cmd(tmp.path())
        .args(["xyz"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("wai --help"));
}

/// 7.5 — `--no-llm` flag bypasses LLM and falls back to `wai search` output.
#[test]
fn why_no_llm_flag_falls_back_to_search() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable and well-supported.",
    );

    // --no-llm must succeed and produce search-style output
    let out = wai_cmd(tmp.path())
        .args(["why", "--no-llm", "TOML"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("TOML"));
}

#[test]
fn why_no_llm_ignores_missing_llm_config_and_error_fallback() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable and well-supported.",
    );

    let config_path = tmp.path().join(".wai").join("config.toml");
    let existing = fs::read_to_string(&config_path).unwrap();
    let updated = format!(
        "{}\n[llm]\nllm = \"claude\"\nfallback = \"error\"\n",
        existing.trim_end()
    );
    fs::write(&config_path, updated).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["why", "--no-llm", "TOML"])
        .env_remove("ANTHROPIC_API_KEY")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stdout.contains("TOML"));
    assert!(stderr.is_empty());
}

#[test]
fn why_with_no_artifacts_prints_guidance_and_exits_cleanly() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("No artifacts found in .wai/"));
}

/// 7.5 — When no LLM backend is configured or available, the command warns
/// and automatically falls back to `wai search`.
#[test]
fn why_auto_fallback_to_search_when_no_llm_available() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable.",
    );

    // Force claude backend (no key) so Ollama auto-detection is skipped
    force_why_llm(tmp.path(), "claude");

    let out = wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .env_remove("ANTHROPIC_API_KEY")
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("Falling back to"));
}

/// 7.6 — A file-path query in a non-git repository does not crash.
///
/// The temp directory is not a git repo, so gather_git_file_context() must
/// return None gracefully. The command falls back to `wai search`.
#[test]
fn why_file_query_in_non_git_repo_does_not_crash() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "Architecture decision: use Rust for performance.",
    );

    // Force claude backend (no key) → deterministic fallback to search
    force_why_llm(tmp.path(), "claude");

    // Use a path-like query; the temp dir is not a git repo
    let out = wai_cmd(tmp.path())
        .args(["why", "src/main.rs"])
        .env_remove("ANTHROPIC_API_KEY")
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

// ─── wai-96y9: Phase 8 — Agent Backend Integration Tests ────────────────────

/// 8.1 — CLAUDECODE=1, no API key → auto-detect selects AgentBackend;
/// wai why prints [AGENT CONTEXT] block to stdout and exits 0.
#[test]
fn why_agent_mode_autodetect_when_claudecode_set() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable and well-supported.",
    );

    // Suppress the one-time privacy notice; no llm override (auto-detect path).
    set_privacy_notice_shown(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .env("CLAUDECODE", "1")
        .env_remove("ANTHROPIC_API_KEY")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("[AGENT CONTEXT]"));
}

/// 8.2 — CLAUDECODE unset, no API key, no claude/ollama binaries in PATH →
/// auto-detect finds no backend; wai why falls back to wai search and exits 0.
#[test]
fn why_fallback_to_search_when_no_agent_no_api_key_no_cli() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen for its human-readable syntax.",
    );

    // Create an empty bin directory so that `claude` and `ollama` are not found.
    let empty_bin = tmp.path().join("empty_bin");
    fs::create_dir_all(&empty_bin).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("CLAUDECODE")
        .env("PATH", &empty_bin)
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("No LLM available"));
}

/// 8.3 — Explicit `[llm] llm = "agent"` selects AgentBackend regardless of
/// whether CLAUDECODE is set; wai why prints [AGENT CONTEXT] to stdout and
/// exits 0, and emits a warning that no Claude Code session is active.
#[test]
fn why_explicit_agent_backend_ignores_claudecode() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");
    write_artifact(
        tmp.path(),
        "myproj",
        "research",
        "2024-01-01-notes.md",
        "TOML was chosen because it is human-readable and well-supported.",
    );

    // Explicitly configure agent backend (also sets privacy_notice_shown = true).
    force_why_llm(tmp.path(), "agent");

    let out = wai_cmd(tmp.path())
        .args(["why", "TOML"])
        .env_remove("CLAUDECODE") // no active Claude Code session
        .env_remove("ANTHROPIC_API_KEY")
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stdout.contains("[AGENT CONTEXT]"));
    assert!(stderr.contains("agent mode requires"));
}

// ─── wai-44b: Conversational Error Tone ─────────────────────────────────────

#[test]
fn error_not_initialized_is_conversational() {
    let stale_wai = std::path::Path::new("/tmp/.wai");
    if stale_wai.exists() {
        let _ = fs::remove_dir_all(stale_wai);
    }

    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("No project initialized"));
}

#[test]
fn error_project_not_found_is_conversational() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["add", "research", "--project", "nonexistent", "notes"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}

// ── wai resource add skill ───────────────────────────────────────────────────

#[test]
fn resource_add_skill_creates_directory_and_skill_md() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "my-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_dir = tmp
        .path()
        .join(".wai/resources/agent-config/skills/my-skill");
    assert!(skill_dir.is_dir(), "skill directory should be created");

    let skill_md = skill_dir.join("SKILL.md");
    assert!(skill_md.is_file(), "SKILL.md should be created");

    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("---"),
        "SKILL.md should have frontmatter delimiters"
    );
    assert!(
        content.contains("name: my-skill"),
        "SKILL.md should include skill name"
    );
    assert!(
        content.contains("description:"),
        "SKILL.md should include description field"
    );
}

#[test]
fn resource_add_skill_fails_on_duplicate() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "dup-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Deprecated path: succeeds with informational message instead of raw error
    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "dup-skill"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("already exists"));
}

#[test]
fn resource_add_skill_fails_on_uppercase_name() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "MySkill"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("Invalid character") || stderr.contains("Invalid skill")));
}

#[test]
fn resource_add_skill_fails_on_consecutive_hyphens() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "my--skill"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("consecutive hyphens"));
}

#[test]
fn resource_add_skill_fails_on_too_long_name() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let long_name = "a".repeat(65);
    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", &long_name])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("too long"));
}

#[test]
fn resource_add_skill_fails_on_dot_prefix() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", ".hidden"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("cannot start with '.'") || stderr.contains("Invalid skill")));
}

#[test]
fn resource_add_skill_refuses_in_safe_mode() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["--safe", "resource", "add", "skill", "my-skill"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("add skill") || stderr.contains("Safe mode")));
}

// ── wai resource add skill (hierarchical) ────────────────────────────────────

#[test]
fn resource_add_hierarchical_skill_creates_nested_directory() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "issue/gather"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_dir = tmp
        .path()
        .join(".wai/resources/agent-config/skills/issue/gather");
    assert!(
        skill_dir.is_dir(),
        "nested skill directory should be created"
    );

    let skill_md = skill_dir.join("SKILL.md");
    assert!(
        skill_md.is_file(),
        "SKILL.md should be created in nested directory"
    );

    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("name: issue/gather"),
        "SKILL.md should include full hierarchical name"
    );
}

#[test]
fn resource_list_skills_shows_hierarchical_and_flat() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "issue/gather"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "plain-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let output = wai_cmd(tmp.path())
        .args(["resource", "list", "skills"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(
        stdout.contains("issue/gather") || stdout.contains("gather"),
        "list should show hierarchical skill; got: {}",
        stdout
    );
    assert!(
        stdout.contains("plain-skill"),
        "list should show flat skill; got: {}",
        stdout
    );
}

#[test]
fn resource_add_hierarchical_skill_conflicts_with_flat_skill() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a flat skill named "issue"
    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "issue"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Deprecated path: succeeds with informational message
    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "issue/gather"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("already exists"));
}

// ── wai resource add skill --template ────────────────────────────────────────

#[test]
fn resource_add_skill_template_gather_contains_wai_search() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "add",
            "skill",
            "my-gather",
            "--template",
            "gather",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/my-gather/SKILL.md");
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("wai search"),
        "gather template should contain wai search"
    );
    assert!(
        content.contains("wai add research"),
        "gather template should contain wai add research"
    );
    assert!(
        content.contains("$ARGUMENTS"),
        "gather template should use $ARGUMENTS placeholder"
    );
    assert!(
        content.contains("$PROJECT"),
        "gather template should use $PROJECT placeholder"
    );
}
