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
