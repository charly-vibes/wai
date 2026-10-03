// REFLECT and REFLECT:REF managed blocks.
//
// The REFLECT:REF block is injected by `wai init` under the slim WAI block to
// point agents at accumulated project patterns; the REFLECT block wraps
// agent-written reflection content. Extracted from managed_block.rs
// (tidy-first move, no behavior change).

use std::path::Path;

pub const REFLECT_REF_START: &str = "<!-- WAI:REFLECT:REF:START -->";
pub const REFLECT_REF_END: &str = "<!-- WAI:REFLECT:REF:END -->";

/// Returns the slim reference block content that tells agents where project
/// patterns live and instructs them to search before starting research.
pub fn wai_reflect_ref_content() -> &'static str {
    "## Accumulated Project Patterns\n\
     \n\
     Project-specific conventions, gotchas, and architecture notes live in\n\
     `.wai/resources/reflections/`. Run `wai search \"<topic>\"` to retrieve relevant\n\
     context before starting research or creating tickets.\n\
     \n\
     > **Before research or ticket creation**: always run `wai search \"<topic>\"` to\n\
     > check for known patterns. Do not rediscover what is already documented.\n"
}

// ── REFLECT block ────────────────────────────────────────────────────────────

const REFLECT_START: &str = "<!-- WAI:REFLECT:START -->";
const REFLECT_END: &str = "<!-- WAI:REFLECT:END -->";

/// Returns true if the file at `path` contains a WAI:REFLECT managed block.
pub fn has_reflect_block(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match std::fs::read_to_string(path) {
        Ok(content) => content.contains(REFLECT_START) && content.contains(REFLECT_END),
        Err(_) => false,
    }
}

/// Read the content between the WAI:REFLECT markers (excluding the markers
/// themselves). Returns `None` if the file does not exist or has no block.
pub fn read_reflect_block(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let start = content.find(REFLECT_START)? + REFLECT_START.len();
    let end = content.find(REFLECT_END)?;
    if start > end {
        return None;
    }
    Some(content[start..end].to_string())
}

#[cfg(test)]
mod reflect_ref_tests {
    use super::*;
    use crate::managed_block::WAI_END;
    use crate::managed_block::inject_managed_block;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    // 6.3: wai_reflect_ref_content() contains "wai search" and the resource path
    #[test]
    fn reflect_ref_content_contains_wai_search() {
        let content = wai_reflect_ref_content();
        assert!(
            content.contains("wai search"),
            "expected 'wai search' in reflect_ref_content"
        );
    }

    #[test]
    fn reflect_ref_content_contains_resource_path() {
        let content = wai_reflect_ref_content();
        assert!(
            content.contains(".wai/resources/reflections/"),
            "expected resource path in reflect_ref_content"
        );
    }

    // 6.5: WAI:REFLECT:REF:START/END block injected by inject_managed_block()
    #[test]
    fn inject_managed_block_adds_reflect_ref_block() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "# Header\n").unwrap();
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains(REFLECT_REF_START),
            "expected WAI:REFLECT:REF:START in output"
        );
        assert!(
            content.contains(REFLECT_REF_END),
            "expected WAI:REFLECT:REF:END in output"
        );
    }

    #[test]
    fn inject_managed_block_reflect_ref_after_wai_end() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "# Header\n").unwrap();
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let wai_end_pos = content.find(WAI_END).expect("WAI:END not found");
        let ref_start_pos = content
            .find(REFLECT_REF_START)
            .expect("REFLECT:REF:START not found");
        assert!(
            ref_start_pos > wai_end_pos,
            "REFLECT:REF block should appear after WAI:END"
        );
    }

    #[test]
    fn inject_managed_block_updates_reflect_ref_in_place() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(
            &path,
            "<!-- WAI:START -->\nwai\n<!-- WAI:END -->\n\n\
             <!-- WAI:REFLECT:REF:START -->\nold content\n<!-- WAI:REFLECT:REF:END -->\n",
        )
        .unwrap();
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let count = content.matches(REFLECT_REF_START).count();
        assert_eq!(count, 1, "should not duplicate REFLECT:REF block");
        assert!(
            !content.contains("old content"),
            "should have replaced old REF content"
        );
    }
}

#[cfg(test)]
mod reflect_tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn has_reflect_block_false_when_file_missing() {
        let dir = tmp();
        assert!(!has_reflect_block(&dir.path().join("CLAUDE.md")));
    }

    #[test]
    fn has_reflect_block_false_when_no_markers() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "# Hello\nSome content\n").unwrap();
        assert!(!has_reflect_block(&path));
    }

    #[test]
    fn has_reflect_block_true_when_markers_present() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(
            &path,
            "# Hello\n<!-- WAI:REFLECT:START -->\nfoo\n<!-- WAI:REFLECT:END -->\n",
        )
        .unwrap();
        assert!(has_reflect_block(&path));
    }

    #[test]
    fn read_reflect_block_returns_none_when_missing() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "# No block here\n").unwrap();
        assert_eq!(read_reflect_block(&path), None);
    }

    #[test]
    fn read_reflect_block_returns_inner_content() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(
            &path,
            "pre\n<!-- WAI:REFLECT:START -->\ninner content\n<!-- WAI:REFLECT:END -->\npost\n",
        )
        .unwrap();
        let got = read_reflect_block(&path).unwrap();
        assert!(got.contains("inner content"));
    }
}
