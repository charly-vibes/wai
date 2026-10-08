#![allow(clippy::too_many_lines)]

mod common;

use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn resource_add_skill_template_unknown_fails_with_valid_names() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "add",
            "skill",
            "my-skill",
            "--template",
            "bogus",
        ])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("gather"));
    assert!(stderr.contains("tdd"));
    assert!(stderr.contains("rule-of-5"));
    assert!(stderr.contains("ubiquitous-language"));
}

#[test]
fn resource_add_skill_template_ubiquitous_language_preserves_progressive_disclosure() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args([
            "resource",
            "add",
            "skill",
            "term-curator",
            "--template",
            "ubiquitous-language",
        ])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/term-curator/SKILL.md");
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(content.contains(".wai/resources/ubiquitous-language/README.md"));
    assert!(content.contains("contexts/"));
    assert!(content.contains("giant glossary file"));
}

#[test]
fn resource_add_skill_no_template_still_creates_bare_stub() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "bare-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/bare-skill/SKILL.md");
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("name: bare-skill"),
        "bare stub should have skill name"
    );
    assert!(
        !content.contains("$ARGUMENTS"),
        "bare stub should not use $ARGUMENTS"
    );
}

// ── wai add skill ────────────────────────────────────────────────────────────

#[test]
fn add_skill_creates_directory_and_skill_md() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["add", "skill", "my-skill"])
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
        content.contains("name: my-skill"),
        "SKILL.md should include skill name"
    );
}

#[test]
fn add_skill_with_template_creates_templated_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["add", "skill", "issue/gather", "--template", "gather"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/issue/gather/SKILL.md");
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("wai search"),
        "gather template should contain wai search"
    );
}

#[test]
fn resource_help_lists_ubiquitous_language_template() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["resource", "--help", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("ubiquitous-language"));
}

#[test]
fn add_skill_help_lists_ubiquitous_language_template() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["add", "skill", "--help", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("ubiquitous-language"));
}

#[test]
fn resource_add_skill_deprecated_still_works_and_warns() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "my-skill"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("deprecated"));
    assert!(stderr.contains("wai add skill"));
}

#[test]
fn resource_add_skill_deprecation_warning_text_format() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "my-skill"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("'wai resource add skill' is deprecated. Use: wai add skill my-skill"));
}

// ── wai resource list skills ─────────────────────────────────────────────────

#[test]
fn resource_list_skills_shows_skill_name() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Add a skill then update SKILL.md with a real description
    let out = wai_cmd(tmp.path())
        .args(["resource", "add", "skill", "list-skill"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let skill_md = tmp
        .path()
        .join(".wai/resources/agent-config/skills/list-skill/SKILL.md");
    fs::write(
        &skill_md,
        "---\nname: list-skill\ndescription: A skill for listing tests\n---\n\n# List Skill\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["resource", "list", "skills"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("list-skill"));
}

#[test]
fn resource_list_skills_empty_shows_hint() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // No skills added — directory is empty after init
    let out = wai_cmd(tmp.path())
        .args(["resource", "list", "skills"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!((stdout.contains("No skills found") || stdout.contains("resource add skill")));
}
