use std::path::Path;
use std::time::SystemTime;

use crate::config::{STATE_FILE, wai_dir};
use crate::plugin::fetch_memories;
use walkdir::WalkDir;

// Hard character budget for LLM context. Keeps prompts within a safe size
// for all supported backends and prevents runaway costs from massive local files.
pub(super) const MAX_CONTEXT_CHARS: usize = 100_000;
pub(super) const MAX_ARTIFACTS_WHEN_TRUNCATING: usize = 50;

// ── Data types ────────────────────────────────────────────────────────────────

/// A single artifact read from the `.wai/` directory tree.
#[derive(Debug, Clone)]
pub struct Artifact {
    /// Path relative to project root (e.g. `.wai/projects/foo/research/2024-01-01.md`).
    pub rel_path: String,
    pub kind: ArtifactKind,
    pub content: String,
    /// Used for recency-based sorting and truncation priority.
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactKind {
    Research,
    Design,
    Plan,
    Handoff,
}

impl ArtifactKind {
    fn from_path(path: &str) -> Option<Self> {
        if path.contains("/research/") {
            Some(ArtifactKind::Research)
        } else if path.contains("/designs/") {
            Some(ArtifactKind::Design)
        } else if path.contains("/plans/") {
            Some(ArtifactKind::Plan)
        } else if path.contains("/handoffs/") {
            Some(ArtifactKind::Handoff)
        } else {
            None
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ArtifactKind::Research => "research",
            ArtifactKind::Design => "design",
            ArtifactKind::Plan => "plan",
            ArtifactKind::Handoff => "handoff",
        }
    }
}

/// Lightweight project metadata included in prompts.
#[derive(Debug, Default)]
pub struct ProjectMeta {
    pub current_phase: Option<String>,
    pub recent_commits: Vec<String>,
}

/// Everything gathered before calling the LLM.
#[derive(Debug)]
pub struct GatheredContext {
    pub query: String,
    // Read in Phase 4 output formatter
    #[allow(dead_code)]
    pub is_file_query: bool,
    pub artifacts: Vec<Artifact>,
    /// Git log for file queries; `None` for natural-language queries.
    pub git_context: Option<String>,
    pub meta: ProjectMeta,
    /// True when artifacts were truncated to fit the context budget.
    pub truncated: bool,
    /// Global memories from bd, injected as supplementary context.
    pub memories: Option<String>,
}

impl GatheredContext {
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

// ── File-path detection ───────────────────────────────────────────────────────

/// Return `true` if `query` looks like a file path rather than a natural-language question.
///
/// Heuristics (conservative — false negatives are fine, false positives are disruptive):
/// - File actually exists at the path
/// - Starts with `./`, `../`, or `src/`
/// - Contains `/` with no spaces (bare path component like `path/to/file.rs`)
pub fn detect_file_query(query: &str) -> bool {
    // Spaces mean it's almost certainly a question, not a path
    if query.contains(' ') {
        return false;
    }
    if Path::new(query).exists() {
        return true;
    }
    if query.starts_with("./") || query.starts_with("../") || query.starts_with("src/") {
        return true;
    }
    if query.contains('/') {
        return true;
    }
    false
}

// ── Context gathering ─────────────────────────────────────────────────────────

/// Gather all context needed to answer `query`.
pub fn gather_context(project_root: &Path, query: &str) -> GatheredContext {
    let is_file_query = detect_file_query(query);

    // Read artifacts, sorted most-recent first
    let mut artifacts = read_artifacts(project_root);
    artifacts.sort_by(|a, b| {
        b.modified
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .cmp(&a.modified.unwrap_or(SystemTime::UNIX_EPOCH))
    });

    let (artifacts, truncated) = truncate_context(artifacts, query, MAX_CONTEXT_CHARS);

    let git_context = if is_file_query {
        gather_git_file_context(query, project_root)
    } else {
        None
    };

    let meta = gather_meta(project_root);
    let memories = fetch_memories(project_root);

    GatheredContext {
        query: query.to_string(),
        is_file_query,
        artifacts,
        git_context,
        meta,
        truncated,
        memories,
    }
}

/// Read all `.md` artifacts from the `.wai/` tree, skipping hidden files and
/// non-artifact directories (resources, plugins, archives config files).
pub(super) fn read_artifacts(project_root: &Path) -> Vec<Artifact> {
    let root = wai_dir(project_root);
    let mut artifacts = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x == "md")
                .unwrap_or(false)
        })
    {
        // Skip dot-files (e.g. .state)
        if entry
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        let path_str = entry.path().to_string_lossy().to_string();
        let kind = match ArtifactKind::from_path(&path_str) {
            Some(k) => k,
            None => continue, // skip non-artifact dirs (resources, plugins, etc.)
        };

        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let modified = entry.metadata().ok().and_then(|m| m.modified().ok());

        let rel_path = entry
            .path()
            .strip_prefix(project_root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        artifacts.push(Artifact {
            rel_path,
            kind,
            content,
            modified,
        });
    }

    artifacts
}

/// Trim artifact list to stay within `max_chars`.
///
/// Priority order:
/// 1. Hard cap at `MAX_ARTIFACTS_WHEN_TRUNCATING` items (most recent first)
/// 2. Within that set, boost artifacts mentioning query terms
/// 3. Fill greedily until `max_chars` is exhausted
///
/// Returns `(selected, was_truncated)`.
pub fn truncate_context(
    artifacts: Vec<Artifact>,
    query: &str,
    max_chars: usize,
) -> (Vec<Artifact>, bool) {
    let total_chars: usize = artifacts.iter().map(|a| a.content.len()).sum();
    if total_chars <= max_chars {
        return (artifacts, false);
    }

    // Hard count cap (already sorted by recency)
    let candidates: Vec<Artifact> = artifacts
        .into_iter()
        .take(MAX_ARTIFACTS_WHEN_TRUNCATING)
        .collect();

    // Score by query-term relevance then fill greedily
    let query_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(String::from)
        .collect();

    let mut scored: Vec<(usize, Artifact)> = candidates
        .into_iter()
        .map(|a| {
            let score = score_relevance(&a.content, &query_terms);
            (score, a)
        })
        .collect();
    // Higher score first; ties keep recency order (stable sort)
    scored.sort_by_key(|e| std::cmp::Reverse(e.0));

    let mut selected = Vec::new();
    let mut chars_used = 0;

    for (_, artifact) in scored {
        if chars_used + artifact.content.len() > max_chars {
            break;
        }
        chars_used += artifact.content.len();
        selected.push(artifact);
    }

    (selected, true)
}

fn score_relevance(content: &str, terms: &[String]) -> usize {
    let lower = content.to_lowercase();
    terms.iter().filter(|t| lower.contains(t.as_str())).count()
}

// ── Git context ───────────────────────────────────────────────────────────────

/// Gather git log for a specific file. Returns `None` if git is unavailable,
/// the directory isn't a repo, or the file isn't tracked.
///
/// All git commands run with `project_root` as the working directory so that
/// relative paths are resolved correctly regardless of where wai was invoked.
pub(super) fn gather_git_file_context(file_path: &str, project_root: &Path) -> Option<String> {
    // Verify project_root is inside a git repo.
    let in_repo = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_root)
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !in_repo {
        return None;
    }

    // Resolve the path so it works when git runs from project_root.
    // If the path can be canonicalized (file exists), make it relative to
    // project_root so git can match it against its index. If it can't be
    // canonicalized (e.g. non-existent path given as hint), use as-is.
    let resolved: String = Path::new(file_path)
        .canonicalize()
        .ok()
        .and_then(|abs| abs.strip_prefix(project_root).ok().map(|r| r.to_path_buf()))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_path.to_string());

    let log = std::process::Command::new("git")
        .args(["log", "--oneline", "-10", "--", &resolved])
        .current_dir(project_root)
        .output()
        .ok()?;

    if !log.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&log.stdout).to_string();
    if text.trim().is_empty() {
        return None;
    }

    Some(format!("Git history for {}:\n{}", file_path, text.trim()))
}

// ── Project metadata ──────────────────────────────────────────────────────────

pub(super) fn gather_meta(project_root: &Path) -> ProjectMeta {
    ProjectMeta {
        current_phase: read_first_project_phase(project_root),
        recent_commits: gather_recent_commits(project_root),
    }
}

fn read_first_project_phase(project_root: &Path) -> Option<String> {
    let projects = crate::config::projects_dir(project_root);
    let entries = std::fs::read_dir(&projects).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let state_path = entry.path().join(STATE_FILE);
        if let Ok(state) = crate::state::ProjectState::load(&state_path) {
            return Some(state.current.to_string());
        }
    }
    None
}

fn gather_recent_commits(project_root: &Path) -> Vec<String> {
    std::process::Command::new("git")
        .args(["log", "--oneline", "-5"])
        .current_dir(project_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── detect_file_query ──

    #[test]
    fn natural_language_question_is_not_file_query() {
        assert!(!detect_file_query("why use TOML for config?"));
        assert!(!detect_file_query("what is the design philosophy"));
        assert!(!detect_file_query("config vs yaml"));
    }

    #[test]
    fn path_with_slash_and_no_spaces_is_file_query() {
        assert!(detect_file_query("src/config.rs"));
        assert!(detect_file_query("path/to/file.rs"));
    }

    #[test]
    fn dotslash_prefix_is_file_query() {
        assert!(detect_file_query("./src/main.rs"));
        assert!(detect_file_query("../other/file.rs"));
    }

    #[test]
    fn src_prefix_is_file_query() {
        assert!(detect_file_query("src/commands/why.rs"));
    }

    #[test]
    fn question_with_slash_but_has_spaces_is_not_file_query() {
        assert!(!detect_file_query("config/yaml vs toml"));
    }

    #[test]
    fn existing_file_on_disk_is_file_query() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("my_module.rs");
        fs::write(&file, "fn main() {}").unwrap();
        // Use absolute path string
        assert!(detect_file_query(file.to_str().unwrap()));
    }

    // ── truncate_context ──

    fn make_artifact(kind: ArtifactKind, content: &str) -> Artifact {
        Artifact {
            rel_path: format!(".wai/projects/test/{}/file.md", kind.label()),
            kind,
            content: content.to_string(),
            modified: None,
        }
    }

    #[test]
    fn under_limit_returns_all_unmodified() {
        let artifacts = vec![
            make_artifact(ArtifactKind::Research, "small content"),
            make_artifact(ArtifactKind::Design, "also small"),
        ];
        let (result, truncated) = truncate_context(artifacts.clone(), "query", 1_000_000);
        assert!(!truncated);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn over_limit_returns_truncated_flag() {
        let big = "x".repeat(300_001);
        let artifacts = vec![
            make_artifact(ArtifactKind::Research, &big),
            make_artifact(ArtifactKind::Design, &big),
        ];
        let (_, truncated) = truncate_context(artifacts, "query", 300_000);
        assert!(truncated);
    }

    #[test]
    fn relevant_artifact_kept_over_irrelevant() {
        let relevant = make_artifact(ArtifactKind::Research, "TOML configuration design");
        let irrelevant = make_artifact(ArtifactKind::Design, "z".repeat(200_000).as_str());

        // Budget allows only one
        let budget = relevant.content.len() + 1;
        let (result, _) = truncate_context(vec![irrelevant, relevant], "TOML", budget);
        assert_eq!(result.len(), 1);
        assert!(result[0].content.contains("TOML"));
    }

    #[test]
    fn empty_artifacts_returns_empty_not_truncated() {
        let (result, truncated) = truncate_context(vec![], "anything", MAX_CONTEXT_CHARS);
        assert!(!truncated);
        assert!(result.is_empty());
    }

    // ── read_artifacts ──

    pub(super) fn setup_wai_project(tmp: &TempDir) {
        let wai = tmp.path().join(".wai");
        let research = wai.join("projects").join("myproj").join("research");
        let designs = wai.join("projects").join("myproj").join("designs");
        let plans = wai.join("projects").join("myproj").join("plans");
        fs::create_dir_all(&research).unwrap();
        fs::create_dir_all(&designs).unwrap();
        fs::create_dir_all(&plans).unwrap();

        fs::write(research.join("2024-01-01-notes.md"), "research content").unwrap();
        fs::write(designs.join("2024-01-02-arch.md"), "design content").unwrap();
        fs::write(plans.join("2024-01-03-plan.md"), "plan content").unwrap();
        // A dot-file that should be ignored
        fs::write(research.join(".state"), "ignored").unwrap();
        // A non-md file that should be ignored
        fs::write(research.join("notes.txt"), "ignored").unwrap();
    }

    #[test]
    fn reads_artifacts_from_all_subdirs() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);
        let artifacts = read_artifacts(tmp.path());
        assert_eq!(artifacts.len(), 3, "should read research, design, plan");
    }

    #[test]
    fn artifact_kinds_are_detected_correctly() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);
        let mut artifacts = read_artifacts(tmp.path());
        artifacts.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        let kinds: Vec<&str> = artifacts.iter().map(|a| a.kind.label()).collect();
        assert!(kinds.contains(&"research"));
        assert!(kinds.contains(&"design"));
        assert!(kinds.contains(&"plan"));
    }

    #[test]
    fn empty_wai_dir_returns_no_artifacts() {
        let tmp = TempDir::new().unwrap();
        let wai = tmp.path().join(".wai");
        fs::create_dir_all(&wai).unwrap();
        let artifacts = read_artifacts(tmp.path());
        assert!(artifacts.is_empty());
    }

    // ── gathered_context is_empty ──

    #[test]
    fn gathered_context_is_empty_when_no_artifacts() {
        let ctx = GatheredContext {
            query: "test".to_string(),
            is_file_query: false,
            artifacts: vec![],
            git_context: None,
            meta: ProjectMeta::default(),
            truncated: false,
            memories: None,
        };
        assert!(ctx.is_empty());
    }

    #[test]
    fn gathered_context_not_empty_with_artifacts() {
        let ctx = GatheredContext {
            query: "test".to_string(),
            is_file_query: false,
            artifacts: vec![make_artifact(ArtifactKind::Research, "content")],
            git_context: None,
            meta: ProjectMeta::default(),
            truncated: false,
            memories: None,
        };
        assert!(!ctx.is_empty());
    }

    // ── gather_context end-to-end ──

    #[test]
    fn gather_context_populates_artifacts_from_tmpdir() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);
        let ctx = gather_context(tmp.path(), "why was this designed this way?");
        assert!(!ctx.artifacts.is_empty(), "should find artifacts in tmpdir");
        assert!(
            !ctx.is_file_query,
            "natural language query is not a file query"
        );
        assert!(ctx.git_context.is_none(), "no git context for NL queries");
        assert!(!ctx.is_empty());
    }

    #[test]
    fn gather_context_marks_file_query_for_existing_path() {
        let tmp = TempDir::new().unwrap();
        setup_wai_project(&tmp);
        // Create a real file so Path::exists() returns true
        let src_file = tmp.path().join("src").join("main.rs");
        fs::create_dir_all(src_file.parent().unwrap()).unwrap();
        fs::write(&src_file, "fn main() {}").unwrap();
        let ctx = gather_context(tmp.path(), src_file.to_str().unwrap());
        assert!(
            ctx.is_file_query,
            "absolute path to existing file is a file query"
        );
    }

    #[test]
    fn gather_git_file_context_returns_history_for_tracked_file() {
        let tmp = TempDir::new().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let src_file = src_dir.join("main.rs");
        fs::write(&src_file, "fn main() {}\n").unwrap();

        std::process::Command::new("git")
            .args(["add", "src/main.rs"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args([
                "-c",
                "user.name=Test User",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "add main",
            ])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let history = gather_git_file_context("src/main.rs", tmp.path())
            .expect("tracked file should produce git history");
        assert!(history.contains("Git history for src/main.rs"));
        assert!(history.contains("add main"));
    }

    #[test]
    fn gather_git_file_context_returns_none_for_untracked_file() {
        let tmp = TempDir::new().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let src_file = src_dir.join("main.rs");
        fs::write(&src_file, "fn main() {}\n").unwrap();

        assert!(gather_git_file_context("src/main.rs", tmp.path()).is_none());
    }
}
