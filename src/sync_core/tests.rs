//! Unit tests for sync_core.

use super::*;

use std::fs;
use tempfile::TempDir;

fn make_config_dir(tmp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let project_root = tmp.path().to_path_buf();
    let config_dir = project_root.join("agent-config");
    fs::create_dir_all(&config_dir).unwrap();
    (project_root, config_dir)
}

// ── symlink tests ─────────────────────────────────────────────────────────

#[test]
#[cfg(unix)]
fn symlink_file_to_file() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    fs::write(config.join("notes.md"), "hello").unwrap();

    let proj = Projection {
        target: "out/notes.md".into(),
        strategy: "symlink".into(),
        sources: vec!["notes.md".into()],
    };

    execute_symlink(&root, &config, &proj).unwrap();

    let target = root.join("out/notes.md");
    assert!(target.exists(), "symlink target should exist");
    assert!(target.is_symlink(), "target should be a symlink");
    assert_eq!(fs::read_to_string(&target).unwrap(), "hello");
}

#[test]
#[cfg(unix)]
fn symlink_directory_to_directory() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    let src_dir = config.join("skills");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("a.md"), "skill-a").unwrap();
    fs::write(src_dir.join("b.md"), "skill-b").unwrap();

    let proj = Projection {
        target: "out/skills".into(),
        strategy: "symlink".into(),
        sources: vec!["skills".into()],
    };

    execute_symlink(&root, &config, &proj).unwrap();

    let out = root.join("out/skills");
    assert!(out.is_dir());
    assert!(out.join("a.md").is_symlink());
    assert_eq!(fs::read_to_string(out.join("b.md")).unwrap(), "skill-b");
}

#[test]
#[cfg(unix)]
fn symlink_creates_missing_parent() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    fs::write(config.join("f.md"), "x").unwrap();

    let proj = Projection {
        target: "deep/nested/dir/f.md".into(),
        strategy: "symlink".into(),
        sources: vec!["f.md".into()],
    };

    execute_symlink(&root, &config, &proj).unwrap();
    assert!(root.join("deep/nested/dir/f.md").exists());
}

#[test]
#[cfg(unix)]
fn symlink_overwrites_pre_existing_target() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    fs::write(config.join("f.md"), "new").unwrap();
    // Pre-create the target as a plain file.
    fs::write(root.join("f.md"), "old").unwrap();

    let proj = Projection {
        target: "f.md".into(),
        strategy: "symlink".into(),
        sources: vec!["f.md".into()],
    };

    execute_symlink(&root, &config, &proj).unwrap();
    assert!(root.join("f.md").is_symlink());
    assert_eq!(fs::read_to_string(root.join("f.md")).unwrap(), "new");
}

// ── idempotency tests ──────────────────────────────────────────────────────

#[test]
#[cfg(unix)]
fn symlink_file_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    fs::write(config.join("notes.md"), "hello").unwrap();

    let proj = Projection {
        target: "out/notes.md".into(),
        strategy: "symlink".into(),
        sources: vec!["notes.md".into()],
    };

    execute_symlink(&root, &config, &proj).unwrap();
    let target = root.join("out/notes.md");
    let meta_before = target.symlink_metadata().unwrap();
    let mtime_before = meta_before.modified().unwrap();

    // Small delay to ensure mtime would differ if recreated.
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Second call should be a no-op.
    execute_symlink(&root, &config, &proj).unwrap();
    let meta_after = target.symlink_metadata().unwrap();
    let mtime_after = meta_after.modified().unwrap();

    assert_eq!(
        mtime_before, mtime_after,
        "symlink should not be recreated when already correct"
    );
}

#[test]
#[cfg(unix)]
fn symlink_dir_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    let src_dir = config.join("skills");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("a.md"), "skill-a").unwrap();
    fs::write(src_dir.join("b.md"), "skill-b").unwrap();

    let proj = Projection {
        target: "out/skills".into(),
        strategy: "symlink".into(),
        sources: vec!["skills".into()],
    };

    execute_symlink(&root, &config, &proj).unwrap();
    let link_a = root.join("out/skills/a.md");
    let mtime_before = link_a.symlink_metadata().unwrap().modified().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(50));

    // Second call should be a no-op.
    execute_symlink(&root, &config, &proj).unwrap();
    let mtime_after = link_a.symlink_metadata().unwrap().modified().unwrap();

    assert_eq!(
        mtime_before, mtime_after,
        "dir symlinks should not be recreated when already correct"
    );
}

#[test]
#[cfg(unix)]
fn symlink_dir_updates_when_source_changes() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    let src_dir = config.join("skills");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("a.md"), "skill-a").unwrap();

    let proj = Projection {
        target: "out/skills".into(),
        strategy: "symlink".into(),
        sources: vec!["skills".into()],
    };

    execute_symlink(&root, &config, &proj).unwrap();
    assert!(!root.join("out/skills/b.md").exists());

    // Add a new file to source.
    fs::write(src_dir.join("b.md"), "skill-b").unwrap();

    // Allow removal of the managed dir in non-interactive test context.
    crate::context::set_context(crate::context::CliContext {
        no_input: false,
        json: false,
        yes: true,
        safe: false,
        verbose: 0,
        quiet: false,
    });

    // Should detect the mismatch and recreate.
    execute_symlink(&root, &config, &proj).unwrap();
    assert!(root.join("out/skills/b.md").exists());
    assert_eq!(
        fs::read_to_string(root.join("out/skills/b.md")).unwrap(),
        "skill-b"
    );
}

// ── safety check tests ────────────────────────────────────────────────────

#[test]
fn check_unmanaged_dir_empty_is_safe() {
    let tmp = TempDir::new().unwrap();
    let empty_dir = tmp.path().join("empty");
    fs::create_dir_all(&empty_dir).unwrap();
    // Empty directory must never fail regardless of TTY state.
    check_unmanaged_dir(&empty_dir).unwrap();
}

#[test]
fn check_unmanaged_dir_wai_marker_is_safe() {
    let tmp = TempDir::new().unwrap();
    let managed = tmp.path().join("managed");
    fs::create_dir_all(managed.join(".wai")).unwrap();
    fs::write(managed.join("data.txt"), "important").unwrap();
    // Has .wai/ → treated as managed, must succeed.
    check_unmanaged_dir(&managed).unwrap();
}

#[test]
fn check_unmanaged_dir_non_empty_non_interactive_fails() {
    let tmp = TempDir::new().unwrap();
    let unmanaged = tmp.path().join("unmanaged");
    fs::create_dir_all(&unmanaged).unwrap();
    fs::write(unmanaged.join("secret.txt"), "precious").unwrap();

    // Force no_input so the check never tries to prompt regardless of TTY state.
    crate::context::set_context(crate::context::CliContext {
        no_input: true,
        json: false,
        yes: false,
        safe: false,
        verbose: 0,
        quiet: false,
    });
    let result = check_unmanaged_dir(&unmanaged);
    assert!(
        result.is_err(),
        "should fail for non-empty unmanaged dir in non-interactive mode"
    );
}

// ── copy tests ────────────────────────────────────────────────────────────

#[test]
fn copy_file_to_file() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    fs::write(config.join("notes.md"), "copy me").unwrap();

    let proj = Projection {
        target: "out/notes.md".into(),
        strategy: "copy".into(),
        sources: vec!["notes.md".into()],
    };

    execute_copy(&root, &config, &proj).unwrap();

    let target = root.join("out/notes.md");
    assert!(target.exists());
    assert!(!target.is_symlink());
    assert_eq!(fs::read_to_string(&target).unwrap(), "copy me");
}

#[test]
fn copy_directory_to_directory() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    let src_dir = config.join("rules");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("r1.md"), "rule1").unwrap();

    let proj = Projection {
        target: "out/rules".into(),
        strategy: "copy".into(),
        sources: vec!["rules".into()],
    };

    execute_copy(&root, &config, &proj).unwrap();

    let out = root.join("out/rules");
    assert!(out.is_dir());
    assert_eq!(fs::read_to_string(out.join("r1.md")).unwrap(), "rule1");
}

#[test]
fn copy_creates_missing_parent() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    fs::write(config.join("x.md"), "x").unwrap();

    let proj = Projection {
        target: "a/b/c/x.md".into(),
        strategy: "copy".into(),
        sources: vec!["x.md".into()],
    };

    execute_copy(&root, &config, &proj).unwrap();
    assert!(root.join("a/b/c/x.md").exists());
}

// ── ensure_parent_dirs tests ──────────────────────────────────────────────

#[test]
fn inline_creates_missing_parent() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    fs::write(config.join("src.md"), "# content").unwrap();

    let proj = Projection {
        target: "deep/path/out.md".into(),
        strategy: "inline".into(),
        sources: vec!["src.md".into()],
    };

    execute_inline(&root, &config, &proj).unwrap();
    assert!(root.join("deep/path/out.md").exists());
}

// ── claude-code projection tests ──────────────────────────────────────────

fn make_skill(config: &std::path::Path, category: &str, action: &str, body: &str) {
    let skill_dir = config.join("skills").join(category).join(action);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), body).unwrap();
}

#[test]
fn translate_skill_name_basic() {
    assert_eq!(translate_skill_name("issue", "gather"), "Issue: Gather");
    assert_eq!(
        translate_skill_name("my-cat", "my-action"),
        "My Cat: My Action"
    );
    assert_eq!(translate_skill_name("a", "b"), "A: B");
}

#[test]
fn claude_code_generates_command_file() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    make_skill(
        &config,
        "issue",
        "gather",
        "---\nname: issue/gather\ndescription: Search codebase\n---\n\n# Issue: Gather\n\nDo stuff.\n",
    );

    execute_claude_code(&root, &config).unwrap();

    let out = root.join(".claude/commands/issue/gather.md");
    assert!(out.exists(), "output file should be created");
    let content = fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("name: \"Issue: Gather\""),
        "name should be translated"
    );
    assert!(
        content.contains("description: \"Search codebase\""),
        "description should be copied"
    );
    assert!(
        content.contains("category: \"issue\""),
        "category should be set"
    );
    assert!(
        content.contains("Auto-generated by wai"),
        "auto-generated header should be present"
    );
    assert!(
        content.contains("# Issue: Gather"),
        "body should be retained"
    );
}

#[test]
fn claude_code_skips_flat_skills() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    // Flat skill: SKILL.md directly under skills/<name>/
    let flat_dir = config.join("skills").join("my-skill");
    fs::create_dir_all(&flat_dir).unwrap();
    fs::write(
        flat_dir.join("SKILL.md"),
        "---\nname: my-skill\ndescription: A flat skill\n---\n\nBody.\n",
    )
    .unwrap();

    execute_claude_code(&root, &config).unwrap();

    // No .claude/commands/my-skill/ should be created
    assert!(
        !root.join(".claude/commands/my-skill").exists(),
        "flat skill should not be synced"
    );
}

#[test]
fn claude_code_creates_parent_dirs() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    make_skill(
        &config,
        "deep",
        "nested",
        "---\nname: deep/nested\ndescription: A nested skill\n---\n\nBody.\n",
    );

    execute_claude_code(&root, &config).unwrap();

    assert!(root.join(".claude/commands/deep/nested.md").exists());
}

// ── reference projection preservation tests ────────────────────────────

fn make_reference_proj(target: &str) -> Projection {
    Projection {
        target: target.to_string(),
        strategy: "reference".to_string(),
        sources: vec!["skills".to_string()],
    }
}

#[test]
fn reference_preserves_user_authored_file() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    // Create a user-authored file (no wai header).
    let target_path = root.join("justfile");
    let user_content = "install:\n\tcargo build\n";
    fs::write(&target_path, user_content).unwrap();

    let proj = make_reference_proj("justfile");
    execute_reference(&root, &config, &proj).unwrap();

    // Content must be unchanged.
    assert_eq!(
        fs::read_to_string(&target_path).unwrap(),
        user_content,
        "user-authored file must not be overwritten"
    );
}

#[test]
fn reference_overwrites_wai_generated_file() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    // Create a file that starts with the wai header (simulating a previous reference).
    let target_path = root.join("references.md");
    fs::write(
        &target_path,
        "# Auto-generated by wai — do not edit directly\n# old content\n",
    )
    .unwrap();

    // Create a skill source so reference content is generated.
    let skills_dir = config.join("skills");
    fs::create_dir_all(&skills_dir).unwrap();
    fs::write(skills_dir.join("test-skill.md"), "# test").unwrap();

    let proj = make_reference_proj("references.md");
    execute_reference(&root, &config, &proj).unwrap();

    let content = fs::read_to_string(&target_path).unwrap();
    assert!(
        content.contains("References to agent config sources"),
        "wai-managed file should be overwritten with new reference content"
    );
}

#[test]
fn reference_creates_file_when_missing() {
    let tmp = TempDir::new().unwrap();
    let (root, config) = make_config_dir(&tmp);

    let target_path = root.join("new-ref.md");
    assert!(!target_path.exists());

    // Create a skill source so reference content is generated.
    let skills_dir = config.join("skills");
    fs::create_dir_all(&skills_dir).unwrap();
    fs::write(skills_dir.join("test-skill.md"), "# test").unwrap();

    let proj = make_reference_proj("new-ref.md");
    execute_reference(&root, &config, &proj).unwrap();

    assert!(target_path.exists(), "reference file should be created");
    let content = fs::read_to_string(&target_path).unwrap();
    assert!(content.contains("References to agent config sources"));
}
