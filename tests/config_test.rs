use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    // Update notice off in tests: existing assertions expect quiet stderr and
    // must not depend on real ~/.cache state (wai-r3p0 tidy).
    cmd.env("GENESIS_NO_UPDATE_CHECK", "1");
    cmd
}

fn init_workspace(dir: &std::path::Path) {
    let out = wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

// ── config list ───────────────────────────────────────────────────────────────

#[test]
fn config_list_shows_empty_sections_on_fresh_workspace() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["config", "list"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Agent Configuration"));
}

#[test]
fn config_list_shows_added_skill_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let skill_file = tmp.path().join("my-skill.md");
    fs::write(&skill_file, "# My Skill").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["config", "add", "skill", skill_file.to_str().unwrap()])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["config", "list"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-skill.md"));
}

// ── config add ────────────────────────────────────────────────────────────────

#[test]
fn config_add_skill_copies_file_to_agent_config_dir() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let skill_file = tmp.path().join("my-skill.md");
    fs::write(&skill_file, "# My Skill").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["config", "add", "skill", skill_file.to_str().unwrap()])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success());
    assert!(stderr.contains("my-skill.md"));

    let dest = tmp
        .path()
        .join(".wai/resources/agent-config/skills/my-skill.md");
    assert!(
        dest.exists(),
        "skill file should be copied to agent-config/skills/"
    );
}

#[test]
fn config_add_rule_copies_file_to_rules_dir() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let rule_file = tmp.path().join("my-rule.md");
    fs::write(&rule_file, "# My Rule").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["config", "add", "rule", rule_file.to_str().unwrap()])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let dest = tmp
        .path()
        .join(".wai/resources/agent-config/rules/my-rule.md");
    assert!(
        dest.exists(),
        "rule file should be copied to agent-config/rules/"
    );
}

// ── failure: unknown config type ──────────────────────────────────────────────

#[test]
fn config_add_unknown_type_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let file = tmp.path().join("stuff.md");
    fs::write(&file, "content").unwrap();

    let out = wai_cmd(tmp.path())
        .args(["config", "add", "unknown-type", file.to_str().unwrap()])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("Unknown config type"));
}
