use std::cmp::Reverse;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use miette::{IntoDiagnostic, Result};
use walkdir::WalkDir;

use crate::config::wai_dir;
use crate::managed_block::read_reflect_block;
use crate::plugin::fetch_memories;

/// Budget allocations for context tiers (conversation, handoffs, secondary, previous reflections).
const CONVERSATION_BUDGET: usize = 30_000;
const HANDOFF_BUDGET: usize = 40_000;
const SECONDARY_BUDGET: usize = 30_000;
const PREVIOUS_REFLECTIONS_BUDGET: usize = 20_000;

/// All context gathered before calling the LLM.
#[derive(Debug)]
pub struct ReflectContext {
    /// Conversation transcript content (truncated to budget from the top).
    pub conversation: Option<String>,
    /// Handoff artifacts, newest-first, concatenated up to budget.
    pub handoffs: Vec<HandoffEntry>,
    /// Number of handoff files actually loaded (for YAML front-matter).
    pub handoff_count: usize,
    /// Research/design/plan artifacts up to budget.
    pub secondary: Vec<SecondaryEntry>,
    /// Existing REFLECT block content per target file (to avoid repeating).
    pub existing_blocks: Vec<(PathBuf, String)>,
    /// Previous reflection resource files, newest-first, up to budget.
    pub previous_reflections: Vec<ReflectionEntry>,
    /// Global memories from bd, to avoid re-deriving already-captured insights.
    pub memories: Option<String>,
}

#[derive(Debug)]
pub struct HandoffEntry {
    pub rel_path: String,
    pub content: String,
}

#[derive(Debug)]
pub struct SecondaryEntry {
    pub rel_path: String,
    pub kind: &'static str,
    pub content: String,
}

/// A previous reflection file loaded from `.wai/resources/reflections/`.
#[derive(Debug)]
pub struct ReflectionEntry {
    pub rel_path: String,
    pub content: String,
}

/// Read the conversation transcript from `path`, truncating to `budget` chars
/// by removing the oldest content from the top.
pub fn read_conversation(path: &Path, budget: usize) -> Result<String> {
    let raw = std::fs::read_to_string(path).into_diagnostic()?;
    Ok(truncate_from_top(&raw, budget))
}

/// Truncate text to `max_chars` by discarding characters from the front
/// (oldest content) so the most recent content is preserved.
pub fn truncate_from_top(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    let overflow = text.len() - max_chars;
    // Advance past the overflow point to the next newline so we don't split mid-line.
    let cut_at = text[overflow..]
        .find('\n')
        .map(|i| overflow + i + 1)
        .unwrap_or(overflow);
    text[cut_at..].to_string()
}

/// Read all handoff artifacts from `.wai/projects/*/handoffs/*.md`, sorted by
/// mtime descending (newest first), concatenated up to `budget` chars.
pub fn read_handoffs(project_root: &Path, budget: usize) -> Vec<HandoffEntry> {
    let wai = wai_dir(project_root);
    let mut entries: Vec<(SystemTime, String, String)> = Vec::new(); // (mtime, rel_path, content)

    let projects_dir = wai.join("projects");
    if !projects_dir.exists() {
        return Vec::new();
    }

    for entry in WalkDir::new(&projects_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Must be in a handoffs/ subdirectory and have .md extension.
        if !path.to_string_lossy().contains("/handoffs/") {
            continue;
        }
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let rel_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        entries.push((mtime, rel_path, content));
    }

    // Sort newest-first.
    entries.sort_by_key(|e| Reverse(e.0));

    // Fill up to budget.
    let mut result = Vec::new();
    let mut used = 0usize;
    for (_, rel_path, content) in entries {
        if used >= budget {
            break;
        }
        let mut take = content.len().min(budget - used);
        while take > 0 && !content.is_char_boundary(take) {
            take -= 1;
        }
        let trimmed = content[..take].to_string();
        used += trimmed.len();
        result.push(HandoffEntry {
            rel_path,
            content: trimmed,
        });
    }
    result
}

/// Read secondary artifacts (research, design, plan) from the `.wai/` tree,
/// sorted newest-first, up to `budget` chars.
pub fn read_secondary_artifacts(project_root: &Path, budget: usize) -> Vec<SecondaryEntry> {
    let wai = wai_dir(project_root);
    let mut entries: Vec<(SystemTime, String, &'static str, String)> = Vec::new();

    for entry in WalkDir::new(&wai)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        // Skip hidden files.
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        let path_str = path.to_string_lossy();
        let kind: Option<&'static str> = if path_str.contains("/research/") {
            Some("research")
        } else if path_str.contains("/designs/") {
            Some("design")
        } else if path_str.contains("/plans/") {
            Some("plan")
        } else {
            None
        };
        let kind = match kind {
            Some(k) => k,
            None => continue,
        };

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let rel_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        entries.push((mtime, rel_path, kind, content));
    }

    entries.sort_by_key(|e| Reverse(e.0));

    let mut result = Vec::new();
    let mut used = 0usize;
    for (_, rel_path, kind, content) in entries {
        if used >= budget {
            break;
        }
        let mut take = content.len().min(budget - used);
        while take > 0 && !content.is_char_boundary(take) {
            take -= 1;
        }
        let trimmed = content[..take].to_string();
        used += trimmed.len();
        result.push(SecondaryEntry {
            rel_path,
            kind,
            content: trimmed,
        });
    }
    result
}

/// Read previous reflection files from `.wai/resources/reflections/`, sorted
/// newest-first, up to `budget` chars.
/// Older files beyond the budget are dropped entirely. Uses `max_depth(1)` — only
/// direct children of the reflections directory are read.
pub fn read_previous_reflections(project_root: &Path, budget: usize) -> Vec<ReflectionEntry> {
    let refl_dir = crate::config::reflections_dir(project_root);
    if !refl_dir.exists() {
        return Vec::new();
    }

    let mut entries: Vec<(SystemTime, String, String)> = Vec::new();

    for entry in WalkDir::new(&refl_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let rel_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        entries.push((mtime, rel_path, content));
    }

    entries.sort_by_key(|e| Reverse(e.0));

    let mut result = Vec::new();
    let mut used = 0usize;
    for (_, rel_path, content) in entries {
        if used >= budget {
            break;
        }
        let mut take = content.len().min(budget - used);
        while take > 0 && !content.is_char_boundary(take) {
            take -= 1;
        }
        let trimmed = content[..take].to_string();
        used += trimmed.len();
        result.push(ReflectionEntry {
            rel_path,
            content: trimmed,
        });
    }
    result
}

/// Gather all context for a reflect run.
pub fn gather_reflect_context(
    project_root: &Path,
    conversation_path: Option<&Path>,
    output_targets: &[PathBuf],
) -> Result<ReflectContext> {
    let conversation = match conversation_path {
        Some(p) => Some(read_conversation(p, CONVERSATION_BUDGET)?),
        None => None,
    };

    let handoffs = read_handoffs(project_root, HANDOFF_BUDGET);
    let handoff_count = handoffs.len();
    let secondary = read_secondary_artifacts(project_root, SECONDARY_BUDGET);
    let previous_reflections = read_previous_reflections(project_root, PREVIOUS_REFLECTIONS_BUDGET);
    let memories = fetch_memories(project_root);

    // Read existing REFLECT blocks from each target so LLM can avoid repeating them.
    let existing_blocks = output_targets
        .iter()
        .filter_map(|p| read_reflect_block(p).map(|block| (p.clone(), block)))
        .collect();

    Ok(ReflectContext {
        conversation,
        handoffs,
        handoff_count,
        secondary,
        existing_blocks,
        previous_reflections,
        memories,
    })
}

// ── Close nudge helpers (Phase 5) ────────────────────────────────────────────

/// Count handoff `.md` files in `project_handoffs_dir` whose mtime is
/// after `since_date` (format: `"YYYY-MM-DD"`).
///
/// If `since_date` is empty or unparsable, all handoffs are counted.
pub fn count_handoffs_since(project_handoffs_dir: &Path, since_date: &str) -> u32 {
    let cutoff: Option<SystemTime> = if since_date.is_empty() {
        None
    } else {
        parse_date_to_system_time(since_date)
    };

    if !project_handoffs_dir.exists() {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(project_handoffs_dir) else {
        return 0;
    };
    let mut count = 0u32;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        match cutoff {
            None => count += 1,
            Some(cutoff_time) => {
                let mtime = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                if mtime > cutoff_time {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Parse a `YYYY-MM-DD` date string into a `SystemTime` at midnight UTC.
///
/// Returns `None` if the string is not a valid date.
pub fn parse_date_to_system_time(date: &str) -> Option<SystemTime> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    use std::time::UNIX_EPOCH;

    // Compute days since Unix epoch (1970-01-01).
    let days = days_since_epoch(year, month, day)?;
    UNIX_EPOCH.checked_add(std::time::Duration::from_secs(days * 86400))
}

/// Compute the number of days from 1970-01-01 to the given date.
fn days_since_epoch(year: i32, month: u32, day: u32) -> Option<u64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Use the Julian Day Number algorithm.
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    let jdn = day as i32 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    // Julian Day Number of 1970-01-01
    const JDN_EPOCH: i32 = 2440588;
    let days = jdn - JDN_EPOCH;
    if days < 0 { None } else { Some(days as u64) }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    fn make_wai_handoff(root: &Path, project: &str, filename: &str, content: &str) {
        let dir = root.join(".wai/projects").join(project).join("handoffs");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(filename), content).unwrap();
    }

    fn make_wai_artifact(root: &Path, project: &str, kind: &str, filename: &str, content: &str) {
        let dir = root.join(".wai/projects").join(project).join(kind);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(filename), content).unwrap();
    }

    // ── Conversation transcript reader tests ──────────────────────────────

    #[test]
    fn truncate_from_top_returns_full_when_within_budget() {
        let text = "short text";
        assert_eq!(truncate_from_top(text, 1000), text);
    }

    #[test]
    fn truncate_from_top_preserves_recent_content() {
        let text = "old line\nnewer line\nnewest line\n";
        let truncated = truncate_from_top(text, 20);
        assert!(truncated.contains("newest line"));
        assert!(!truncated.contains("old line"));
    }

    #[test]
    fn read_conversation_reads_file_within_budget() {
        let dir = tmp();
        let path = dir.path().join("transcript.txt");
        fs::write(&path, "session content here").unwrap();
        let content = read_conversation(&path, 30_000).unwrap();
        assert_eq!(content, "session content here");
    }

    #[test]
    fn read_conversation_truncates_large_file() {
        let dir = tmp();
        let path = dir.path().join("transcript.txt");
        // Write old block (a's) then new block (b's); budget fits only the new block.
        // 100 a's + \n + 100 b's + \n = 202 chars. Budget=110 cuts off the a's.
        let big = "a".repeat(100) + "\n" + &"b".repeat(100) + "\n";
        fs::write(&path, &big).unwrap();
        let content = read_conversation(&path, 110).unwrap();
        assert!(content.trim_end().ends_with('b'));
        assert!(!content.contains('a'));
    }

    // ── Handoff artifact reader tests ─────────────────────────────────────

    #[test]
    fn read_handoffs_returns_empty_when_no_wai_dir() {
        let dir = tmp();
        let handoffs = read_handoffs(dir.path(), HANDOFF_BUDGET);
        assert!(handoffs.is_empty());
    }

    #[test]
    fn read_handoffs_collects_handoff_files() {
        let dir = tmp();
        make_wai_handoff(
            dir.path(),
            "my-project",
            "2026-02-24-session.md",
            "# Session\nContent",
        );
        let handoffs = read_handoffs(dir.path(), HANDOFF_BUDGET);
        assert_eq!(handoffs.len(), 1);
        assert!(handoffs[0].content.contains("Content"));
    }

    #[test]
    fn read_handoffs_respects_budget() {
        let dir = tmp();
        make_wai_handoff(dir.path(), "proj", "h1.md", &"x".repeat(30_000));
        make_wai_handoff(dir.path(), "proj", "h2.md", &"y".repeat(30_000));
        let handoffs = read_handoffs(dir.path(), 40_000);
        let total: usize = handoffs.iter().map(|h| h.content.len()).sum();
        assert!(total <= 40_000);
    }

    // ── Close nudge tests (Phase 5) ───────────────────────────────────────

    #[test]
    fn parse_date_to_system_time_known_date() {
        // 1970-01-01 should be epoch.
        let t = parse_date_to_system_time("1970-01-01").unwrap();
        assert_eq!(t, std::time::UNIX_EPOCH);
    }

    #[test]
    fn parse_date_to_system_time_returns_none_for_bad_date() {
        assert!(parse_date_to_system_time("not-a-date").is_none());
        assert!(parse_date_to_system_time("2026-13-01").is_none());
    }

    #[test]
    fn count_handoffs_since_returns_zero_when_dir_missing() {
        let dir = tmp();
        let handoffs = dir.path().join("handoffs");
        assert_eq!(count_handoffs_since(&handoffs, ""), 0);
    }

    #[test]
    fn count_handoffs_since_counts_all_when_no_date() {
        let dir = tmp();
        let handoffs = dir.path().join("handoffs");
        fs::create_dir_all(&handoffs).unwrap();
        fs::write(handoffs.join("h1.md"), "content").unwrap();
        fs::write(handoffs.join("h2.md"), "content").unwrap();
        assert_eq!(count_handoffs_since(&handoffs, ""), 2);
    }

    #[test]
    fn count_handoffs_since_skips_non_md_files() {
        let dir = tmp();
        let handoffs = dir.path().join("handoffs");
        fs::create_dir_all(&handoffs).unwrap();
        fs::write(handoffs.join("h1.md"), "content").unwrap();
        fs::write(handoffs.join("notes.txt"), "ignored").unwrap();
        assert_eq!(count_handoffs_since(&handoffs, ""), 1);
    }

    #[test]
    fn count_handoffs_since_excludes_old_files_by_date() {
        let dir = tmp();
        let handoffs = dir.path().join("handoffs");
        fs::create_dir_all(&handoffs).unwrap();
        // Write a file then immediately use a far-future cutoff date.
        fs::write(handoffs.join("h1.md"), "content").unwrap();
        // 9999-12-31 is far in the future, so the file's mtime is before it.
        assert_eq!(count_handoffs_since(&handoffs, "9999-12-31"), 0);
    }

    // ── Secondary artifact reader tests ───────────────────────────────────

    #[test]
    fn read_secondary_artifacts_collects_research_design_plan() {
        let dir = tmp();
        make_wai_artifact(dir.path(), "proj", "research", "r.md", "research content");
        make_wai_artifact(dir.path(), "proj", "designs", "d.md", "design content");
        make_wai_artifact(dir.path(), "proj", "plans", "p.md", "plan content");
        let artifacts = read_secondary_artifacts(dir.path(), SECONDARY_BUDGET);
        assert_eq!(artifacts.len(), 3);
        let kinds: Vec<&str> = artifacts.iter().map(|a| a.kind).collect();
        assert!(kinds.contains(&"research"));
        assert!(kinds.contains(&"design"));
        assert!(kinds.contains(&"plan"));
    }

    #[test]
    fn read_secondary_artifacts_skips_handoffs() {
        let dir = tmp();
        make_wai_handoff(dir.path(), "proj", "h.md", "handoff content");
        make_wai_artifact(dir.path(), "proj", "research", "r.md", "research content");
        let artifacts = read_secondary_artifacts(dir.path(), SECONDARY_BUDGET);
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].kind, "research");
    }

    #[test]
    fn read_secondary_artifacts_respects_budget() {
        let dir = tmp();
        make_wai_artifact(dir.path(), "proj", "research", "r1.md", &"a".repeat(20_000));
        make_wai_artifact(dir.path(), "proj", "research", "r2.md", &"b".repeat(20_000));
        let artifacts = read_secondary_artifacts(dir.path(), 25_000);
        let total: usize = artifacts.iter().map(|a| a.content.len()).sum();
        assert!(total <= 25_000);
    }

    // ── gather_reflect_context handoff_count tests ────────────────────────

    #[test]
    fn gather_reflect_context_handoff_count_zero_when_no_handoffs() {
        let dir = tmp();
        fs::write(dir.path().join("CLAUDE.md"), "# Claude\n").unwrap();
        let targets = vec![dir.path().join("CLAUDE.md")];
        let ctx = gather_reflect_context(dir.path(), None, &targets).unwrap();
        assert_eq!(ctx.handoff_count, 0);
    }

    #[test]
    fn gather_reflect_context_handoff_count_matches_loaded_entries() {
        let dir = tmp();
        fs::write(dir.path().join("CLAUDE.md"), "# Claude\n").unwrap();
        make_wai_handoff(dir.path(), "proj", "h1.md", "handoff 1");
        make_wai_handoff(dir.path(), "proj", "h2.md", "handoff 2");
        let targets = vec![dir.path().join("CLAUDE.md")];
        let ctx = gather_reflect_context(dir.path(), None, &targets).unwrap();
        assert_eq!(ctx.handoff_count, 2);
        assert_eq!(ctx.handoff_count, ctx.handoffs.len());
    }
}
