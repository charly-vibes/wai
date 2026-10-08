#![allow(clippy::too_many_lines)]

mod common;

use common::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── wai plugin list ────────────────────────────────────────────────────────

#[test]
fn plugin_list_shows_builtin_plugins() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["plugin", "list"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(((stdout.contains("git") || stdout.contains("beads")) || stdout.contains("openspec")));
}

#[test]
fn plugin_list_json_outputs_plugins() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["plugin", "list", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"name\""));
}

// ─── wai plugin enable / disable ────────────────────────────────────────────

#[test]
fn plugin_enable_persists_to_config() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["plugin", "enable", "git"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("enabled"));

    let config = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert!(config.contains("git"));
    assert!(config.contains("enabled = true"));
}

#[test]
fn plugin_disable_persists_to_config() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Enable first, then disable
    let out = wai_cmd(tmp.path())
        .args(["plugin", "enable", "git"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["plugin", "disable", "git"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("disabled"));

    let config = fs::read_to_string(tmp.path().join(".wai/config.toml")).unwrap();
    assert!(config.contains("git"));
    assert!(config.contains("enabled = false"));
}

#[test]
fn plugin_enable_unknown_plugin_fails() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["plugin", "enable", "nonexistent"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("not found"));
}

#[test]
fn plugin_enable_json_outputs_state() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["plugin", "enable", "git", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"plugin\""));
    assert!(stdout.contains("\"enabled\": true"));
}

#[test]
fn plugin_passthrough_ready_executes_beads_command() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    fs::create_dir_all(tmp.path().join(".beads")).unwrap();
    let fake_bin = install_fake_bd_passthrough(tmp.path());
    let path = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = wai_cmd(tmp.path())
        .args(["beads", "ready"])
        .env("PATH", path)
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("fake ready issue"));
}

#[test]
fn plugin_passthrough_unknown_command_fails_for_detected_plugin() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    fs::create_dir_all(tmp.path().join(".beads")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["beads", "unknown-cmd"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("has no command 'unknown-cmd'"));
}

#[test]
fn plugin_trust_skips_unapproved_hook_on_status() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");
    write_malicious_plugin(tmp.path(), "hook-marker");

    // status should NOT execute the unapproved hook.
    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["status"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not trusted"));

    assert!(
        !tmp.path().join("hook-marker").exists(),
        "unapproved hook must not execute"
    );
}

#[test]
fn plugin_trust_approve_enables_hook() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("wai-data");
    init_workspace(tmp.path());
    create_project(tmp.path(), "test-p");
    write_malicious_plugin(tmp.path(), "hook-marker");

    // Approve the plugin, then status should execute the hook.
    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["plugin", "trust", "evil"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Trusted"));

    wai_cmd_with_data_dir(tmp.path(), &data_dir)
        .args(["status"])
        .assert()
        .success();

    assert!(
        tmp.path().join("hook-marker").exists(),
        "approved hook must execute"
    );
}

#[test]
fn way_check_documentation_partial() {
    let tmp = TempDir::new().unwrap();
    // Has critical files but missing LICENSE, CONTRIBUTING, docs/, and doc tool
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project documentation"));
    assert!(stdout.contains("Essential files present"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_documentation_license_md_variant() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Test").unwrap();
    fs::write(tmp.path().join("LICENSE.md"), "MIT").unwrap();
    fs::write(tmp.path().join("CONTRIBUTING.md"), "# Contributing").unwrap();
    fs::write(tmp.path().join(".gitignore"), "*.tmp").unwrap();
    fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::create_dir(tmp.path().join("docs")).unwrap();
    fs::write(tmp.path().join("docs").join("index.md"), "# Docs").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project documentation"));
    assert!(stdout.contains("Complete"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_ai_instructions_claude_md() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "# Instructions").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("AI-agent context"));
    assert!(stdout.contains("CLAUDE.md detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_ai_instructions_suggests_reflect_when_no_reflections() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "# Instructions").unwrap();
    // No .wai/resources/reflections/ directory — reflect has never been run.

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("AI-agent context"));
    assert!(stdout.contains("CLAUDE.md detected"));
    assert!(stdout.contains("wai reflect"));
    assert!(stdout.contains("resources/reflections"));
}

#[test]
fn way_check_ai_instructions_no_reflect_suggestion_when_reflections_exist() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "# Instructions").unwrap();
    // Create a reflection file to simulate reflect having been run.
    let refl_dir = tmp.path().join(".wai/resources/reflections");
    fs::create_dir_all(&refl_dir).unwrap();
    fs::write(refl_dir.join("2026-01-01-my-proj.md"), "## patterns").unwrap();

    let output = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("AI-agent context"),
        "expected AI-agent context check"
    );
    assert!(
        stdout.contains("CLAUDE.md detected"),
        "expected CLAUDE.md detected message"
    );
    // No suggestion should be present when reflections exist.
    assert!(
        !stdout.contains("wai reflect"),
        "unexpected wai reflect suggestion when reflection file exists"
    );
}

#[test]
fn way_check_ai_instructions_agents_md() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("AGENTS.md"), "# Instructions").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("AI-agent context"));
    assert!(stdout.contains("AGENTS.md detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_ai_instructions_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("AI-agent context"));
    assert!(stdout.contains("No AI instruction files detected"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_agent_config_sync_pass() {
    let tmp = TempDir::new().unwrap();
    let agent_config = tmp.path().join(".wai/resources/agent-config");
    fs::create_dir_all(&agent_config).unwrap();
    fs::write(
        agent_config.join(".projections.yml"),
        "projections:\n  - target: .agents",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Agent context sync"));
    assert!(stdout.contains("Agent config projections configured"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_agent_config_sync_info_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Agent context sync"));
    assert!(stdout.contains("No agent config projections found"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_agent_config_sync_info_empty() {
    let tmp = TempDir::new().unwrap();
    let agent_config = tmp.path().join(".wai/resources/agent-config");
    fs::create_dir_all(&agent_config).unwrap();
    fs::write(agent_config.join(".projections.yml"), "projections: []").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Agent context sync"));
    assert!(stdout.contains("Projections file exists but no projections are configured"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_cicd_github_actions() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".github/workflows/ci.yml"), "name: CI").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Automated verification"));
    assert!(stdout.contains("GitHub Actions configured"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_cicd_gitlab() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".gitlab-ci.yml"), "stages: [test]").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Automated verification"));
    assert!(stdout.contains("GitLab CI configured"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_cicd_circleci() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".circleci")).unwrap();
    fs::write(tmp.path().join(".circleci/config.yml"), "version: 2.1").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Automated verification"));
    assert!(stdout.contains("CircleCI configured"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_cicd_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Automated verification"));
    assert!(stdout.contains("No CI/CD configuration detected"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_devcontainer_dir() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".devcontainer")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Reproducible environments"));
    assert!(stdout.contains(".devcontainer/ directory detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_devcontainer_json() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".devcontainer.json"), "{}").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Reproducible environments"));
    assert!(stdout.contains(".devcontainer.json detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_devcontainer_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Reproducible environments"));
    assert!(stdout.contains("No dev container configuration detected"));
    assert!(stdout.contains("\"warn\""));
}

// ─── wai way: new feature checks ────────────────────────────────────────────

#[test]
fn way_check_llm_txt_present() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("llm.txt"), "# Project docs").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("LLM-friendly context"));
    assert!(stdout.contains("llm.txt detected"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_llm_txt_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("LLM-friendly context"));
    assert!(stdout.contains("No llm.txt detected"));
    assert!(stdout.contains("llmstxt.org"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_agent_skills_present() {
    let tmp = TempDir::new().unwrap();
    let skills_base = tmp.path().join(".wai/resources/agent-config/skills");
    fs::create_dir_all(skills_base.join("rule-of-5-universal")).unwrap();
    fs::write(
        skills_base.join("rule-of-5-universal/SKILL.md"),
        "# Rule of 5 Universal",
    )
    .unwrap();
    fs::create_dir_all(skills_base.join("commit")).unwrap();
    fs::write(skills_base.join("commit/SKILL.md"), "# Commit").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Extended agent capabilities"));
    assert!(stdout.contains("2 skill(s) configured"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_agent_skills_empty_dir() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".wai/resources/agent-config/skills")).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Extended agent capabilities"));
    assert!(stdout.contains("Skills directory present but empty"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_agent_skills_missing() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Extended agent capabilities"));
    assert!(stdout.contains("No skills configured"));
    assert!(stdout.contains("\"warn\""));
}

#[test]
fn way_check_agent_skills_missing_suggests_fix_command() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("wai way --fix skills"));
}

#[test]
fn way_check_ubiquitous_language_fully_configured_tree_passes() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(
        tmp.path()
            .join(".wai/resources/ubiquitous-language/contexts"),
    )
    .unwrap();
    fs::write(
        tmp.path()
            .join(".wai/resources/ubiquitous-language/README.md"),
        "# Ubiquitous language\n",
    )
    .unwrap();
    fs::write(
        tmp.path()
            .join(".wai/resources/ubiquitous-language/contexts/billing.md"),
        "# Billing\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Ubiquitous language context"));
    assert!(stdout.contains("\"pass\""));
    assert!(stdout.contains("fully configured"));
}

#[test]
fn way_check_ubiquitous_language_skeleton_tree_reports_info() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".wai/resources/ubiquitous-language")).unwrap();
    fs::write(
        tmp.path()
            .join(".wai/resources/ubiquitous-language/README.md"),
        "# Ubiquitous language\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Ubiquitous language context"));
    assert!(stdout.contains("\"warn\""));
    assert!(stdout.contains("bounded-context files"));
}

#[test]
fn way_check_ubiquitous_language_shared_only_tree_reports_info() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".wai/resources/ubiquitous-language")).unwrap();
    fs::write(
        tmp.path()
            .join(".wai/resources/ubiquitous-language/README.md"),
        "# Ubiquitous language\n",
    )
    .unwrap();
    fs::write(
        tmp.path()
            .join(".wai/resources/ubiquitous-language/shared.md"),
        "# Shared\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Ubiquitous language context"));
    assert!(stdout.contains("\"warn\""));
    assert!(stdout.contains("valid starting point"));
}

#[test]
fn way_check_ubiquitous_language_malformed_tree_without_index_reports_info() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(
        tmp.path()
            .join(".wai/resources/ubiquitous-language/contexts"),
    )
    .unwrap();
    fs::write(
        tmp.path()
            .join(".wai/resources/ubiquitous-language/contexts/billing.md"),
        "# Billing\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Ubiquitous language context"));
    assert!(stdout.contains("\"warn\""));
    assert!(stdout.contains("README.md is required"));
}

#[test]
fn way_fix_skills_prints_description_before_acting() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let output = wai_cmd(tmp.path())
        .args(["way", "--fix", "skills"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    // Should explain what it's doing before listing created skills
    assert!(
        stdout.contains("rule-of-5-universal") && stdout.contains("commit"),
        "Should mention both skills, got:\n{stdout}"
    );
    assert!(
        stdout.contains("scaffold") || stdout.contains("Scaffold") || stdout.contains("skill"),
        "Should describe the action before acting, got:\n{stdout}"
    );
}

#[test]
fn way_fix_skills_scaffolds_both_skills() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["way", "--fix", "skills"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills/rule-of-5-universal/SKILL.md")
            .exists(),
        "rule-of-5-universal/SKILL.md should be created"
    );
    assert!(
        tmp.path()
            .join(".wai/resources/agent-config/skills/commit/SKILL.md")
            .exists(),
        "commit/SKILL.md should be created"
    );
}

#[test]
fn way_check_justfile_recipes() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("justfile"),
        "test:\n\techo test\n\ninstall:\n\techo install\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Command standardization"));
    assert!(stdout.contains("recipes:"));
    assert!(stdout.contains("\"pass\""));
}

// ─── way: check_test_coverage ─────────────────────────────────────────────────

#[test]
fn way_check_test_coverage_tarpaulin_with_threshold() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("tarpaulin.toml"),
        "[report]\nfail-under = 80\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Test coverage"));
    assert!(stdout.contains("tarpaulin configured (threshold enforced)"));
    assert!(stdout.contains("\"pass\""));
}

#[test]
fn way_check_test_coverage_tarpaulin_no_threshold() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("tarpaulin.toml"), "[report]\n").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Test coverage"));
    assert!(stdout.contains("tarpaulin configured"));
    assert!(stdout.contains("\"pass\""));
    assert!(stdout.contains("fail-under"));
}
