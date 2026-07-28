/// Test that genesis::managed_block API is accessible from wai.
///
/// These tests validate that the genesis BlockInjector can replace
/// wai's local injector mechanics in src/managed_block.rs.
use genesis::managed_block::{BlockDef, BlockInjector, BlockRegistry, InjectResult};
use tempfile::TempDir;

fn test_injector() -> BlockInjector {
    let mut reg = BlockRegistry::new();
    reg.register(BlockDef::new("WAI"));
    reg.register(BlockDef::with_markers(
        "WAI:REFLECT:REF",
        "<!-- WAI:REFLECT:REF:START -->",
        "<!-- WAI:REFLECT:REF:END -->",
    ));
    BlockInjector::new(reg)
}

#[test]
fn test_genesis_block_injector_creates_new_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("AGENTS.md");
    let injector = test_injector();

    let result = injector.inject(&path, "WAI", "\n# Test content\n").unwrap();
    assert_eq!(result, InjectResult::Created);
    assert!(path.exists());

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<!-- WAI:START -->"));
    assert!(content.contains("<!-- WAI:END -->"));
    assert!(content.contains("# Test content"));
}

#[test]
fn test_genesis_block_injector_updates_existing_block() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("AGENTS.md");
    let injector = test_injector();

    injector.inject(&path, "WAI", "\n# Old\n").unwrap();
    let result = injector.inject(&path, "WAI", "\n# New\n").unwrap();
    assert_eq!(result, InjectResult::Updated);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("# New"));
    assert!(!content.contains("# Old"));
    assert_eq!(content.matches("<!-- WAI:START -->").count(), 1);
}

#[test]
fn test_genesis_block_injector_prepends_to_existing_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("AGENTS.md");
    std::fs::write(&path, "# Existing content\n").unwrap();
    let injector = test_injector();

    let result = injector.inject(&path, "WAI", "\n# Block\n").unwrap();
    assert_eq!(result, InjectResult::Prepended);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("<!-- WAI:START -->"));
    assert!(content.contains("# Existing content"));
}

#[test]
fn test_genesis_block_injector_has_block() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("AGENTS.md");
    let injector = test_injector();

    assert!(!injector.has_block(&path, "WAI"));
    injector.inject(&path, "WAI", "content").unwrap();
    assert!(injector.has_block(&path, "WAI"));
}

#[test]
fn test_genesis_block_injector_read_block() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("AGENTS.md");
    let injector = test_injector();

    injector.inject(&path, "WAI", "\n# Readable\n").unwrap();
    let content = injector.read_block(&path, "WAI").unwrap();
    assert!(content.contains("<!-- WAI:START -->"));
    assert!(content.contains("<!-- WAI:END -->"));
    assert!(content.contains("# Readable"));
}

#[test]
fn test_genesis_block_injector_multiple_blocks() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("AGENTS.md");
    let injector = test_injector();

    injector.inject(&path, "WAI", "\n# WAI\n").unwrap();
    injector
        .inject(&path, "WAI:REFLECT:REF", "\n# REFLECT\n")
        .unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<!-- WAI:START -->"));
    assert!(content.contains("<!-- WAI:END -->"));
    assert!(content.contains("<!-- WAI:REFLECT:REF:START -->"));
    assert!(content.contains("<!-- WAI:REFLECT:REF:END -->"));
}

/// Regression: `wai sync` still injects/refreshes the WAI:START block
/// after migrating managed_block injector mechanics to genesis.
#[test]
fn test_wai_sync_injects_managed_block() {
    use assert_cmd::Command;

    let dir = TempDir::new().unwrap();

    // Init a workspace, which creates AGENTS.md with the managed block
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir.path());
    cmd.env("NO_COLOR", "1");
    cmd.args(["init", "--name", "regression-test"]);
    cmd.assert().success();

    // Check that AGENTS.md has the WAI:START block
    let agents_md = dir.path().join("AGENTS.md");
    assert!(agents_md.exists(), "AGENTS.md should exist after init");
    let content = std::fs::read_to_string(&agents_md).unwrap();
    eprintln!(
        "DEBUG after init: WAI:START count={}, REFLECT:REF:START count={}",
        content.matches("<!-- WAI:START -->").count(),
        content.matches("<!-- WAI:REFLECT:REF:START -->").count()
    );
    // Print first 300 chars of content
    eprintln!(
        "DEBUG content start: {:?}",
        &content[..content.len().min(300)]
    );
    assert!(
        content.contains("<!-- WAI:START -->"),
        "AGENTS.md should contain WAI:START after init"
    );
    assert!(
        content.contains("<!-- WAI:END -->"),
        "AGENTS.md should contain WAI:END after init"
    );

    // Run sync to verify it refreshes the block
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir.path());
    cmd.env("NO_COLOR", "1");
    cmd.args(["sync"]);
    cmd.assert().success();

    // Block is still there after sync
    let content = std::fs::read_to_string(&agents_md).unwrap();
    eprintln!(
        "DEBUG after sync: WAI:START count={}, REFLECT:REF:START count={}",
        content.matches("<!-- WAI:START -->").count(),
        content.matches("<!-- WAI:REFLECT:REF:START -->").count()
    );
    assert_eq!(
        content.matches("<!-- WAI:START -->").count(),
        1,
        "should have exactly one WAI:START block"
    );
    assert_eq!(
        content.matches("<!-- WAI:END -->").count(),
        1,
        "should have exactly one WAI:END block"
    );
    assert_eq!(
        content.matches("<!-- WAI:REFLECT:REF:START -->").count(),
        1,
        "should have exactly one REFLECT:REF:START block"
    );
}
