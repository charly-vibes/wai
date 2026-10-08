use miette::Result;
use owo_colors::OwoColorize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{projects_dir, wai_dir};
use crate::context::current_context;
use crate::json::{SearchPayload, SearchResult};
use crate::output::print_envelope_list;
use crate::plugin::fetch_memories_for_query;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemoryMatch {
    key: String,
    value: String,
    raw: String,
}

use super::require_project;

type Matcher = Box<dyn Fn(&str) -> Option<(usize, usize)>>;

/// A raw search hit: (display path, line number, line, match start, match
/// end, surrounding context lines).
type RawMatch = (String, usize, String, usize, usize, Vec<String>);

const DEFAULT_LIMIT: usize = 20;

/// CLI surface for `wai search` (clap derive).
#[derive(Debug, Clone, clap::Args)]
pub struct SearchArgs {
    /// Search query (supports regex with --regex flag)
    pub query: String,

    /// Filter by artifact type (research, plan, design, handoff)
    #[arg(long = "type")]
    pub type_filter: Option<String>,

    /// Search within a specific project
    #[arg(long = "in")]
    pub project: Option<String>,

    /// Treat query as a regular expression
    #[arg(long)]
    pub regex: bool,

    /// Limit number of results shown
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    /// Filter by tag (frontmatter-based; repeatable)
    #[arg(long)]
    pub tag: Vec<String>,

    /// Return only the most recently dated match
    #[arg(long)]
    pub latest: bool,

    /// Number of surrounding context lines to show (like grep -C)
    #[arg(short = 'C', long = "context", default_value_t = 0)]
    pub context_size: usize,

    /// Include bd memories in search results
    #[arg(long)]
    pub include_memories: bool,
}

/// Valid `--type` values for `wai search` (canonical + plural aliases).
fn known_type_values() -> &'static [&'static str] {
    &[
        "research", "plan", "plans", "design", "designs", "handoff", "handoffs", "review",
        "reviews",
    ]
}

/// Reject invalid values for `--type` early, before any directory walk.
fn validate_type_filter(type_filter: Option<&str>) -> Result<()> {
    if let Some(type_f) = type_filter
        && !known_type_values().contains(&type_f)
    {
        return Err(crate::error::WaiError::InvalidSearchType {
            value: type_f.to_string(),
            valid: "research, plan, design, handoff, review".to_string(),
        }
        .into());
    }
    Ok(())
}

pub fn run(args: SearchArgs) -> Result<()> {
    let SearchArgs {
        query,
        type_filter,
        project,
        regex,
        limit,
        tag,
        latest,
        context_size,
        include_memories,
    } = args;
    let display_limit = limit.unwrap_or(DEFAULT_LIMIT);
    validate_type_filter(type_filter.as_deref())?;
    let project_root = require_project()?;
    let context = current_context();

    let search_root = if let Some(ref proj_name) = project {
        let dir = projects_dir(&project_root).join(proj_name);
        if !dir.exists() {
            return Err(crate::error::WaiError::ProjectNotFound {
                name: proj_name.clone(),
            }
            .into());
        }
        dir
    } else {
        wai_dir(&project_root)
    };

    let matcher: Matcher = if regex {
        let re = regex::Regex::new(&query)
            .map_err(|e| miette::miette!("Invalid regex '{}': {}", query, e))?;
        Box::new(move |line: &str| re.find(line).map(|m| (m.start(), m.end())))
    } else {
        let query_lower = query.to_lowercase();
        Box::new(move |line: &str| {
            let lower = line.to_lowercase();
            let lower_start = lower.find(&query_lower)?;
            let lower_end = lower_start + query_lower.len();
            // lower_start/lower_end are byte offsets in the *lowercased* string.
            // Some chars change byte length when lowercased (e.g. 'İ' 2 bytes → 'i' 1 byte),
            // so convert via char count to get valid byte offsets in the original line.
            let char_start = lower[..lower_start].chars().count();
            let char_end = lower[..lower_end].chars().count();
            let byte_start = line
                .char_indices()
                .nth(char_start)
                .map_or(line.len(), |(i, _)| i);
            let byte_end = line
                .char_indices()
                .nth(char_end)
                .map_or(line.len(), |(i, _)| i);
            Some((byte_start, byte_end))
        })
    };

    // results: (file_path, line_num, line, start, end, context_lines)
    let mut results: Vec<RawMatch> = Vec::new();

    // Canonical paths already scanned, so the global resources walk (wai-gkk3)
    // never duplicates a hit for a file also present in the repo scope (they
    // can coincide when the workspace lives under $HOME/.wai).
    let mut scanned: HashSet<PathBuf> = HashSet::new();

    // Managed files that should not appear in artifact search results.
    let agents_md = search_root.join("AGENTS.md");

    collect_matches(
        &search_root,
        &project_root,
        &agents_md,
        &matcher,
        type_filter.as_deref(),
        &tag,
        context_size,
        "",
        &mut scanned,
        &mut results,
    );

    // Global scope (wai-gkk3): ~/.wai/resources is searchable from any repo
    // workspace, with hits tagged `global:<rel-path>`. Skipped when --in
    // scopes the search to a single project.
    if project.is_none() {
        let global_dir = crate::config::global_resources_dir();
        let global_agents_md = global_dir.join("AGENTS.md");
        collect_matches(
            &global_dir,
            &global_dir,
            &global_agents_md,
            &matcher,
            type_filter.as_deref(),
            &tag,
            context_size,
            "global:",
            &mut scanned,
            &mut results,
        );
    }

    // Apply --latest: keep only matches from the file with the greatest date prefix.
    if latest && !results.is_empty() {
        let best_path = results
            .iter()
            .map(|(path, ..)| path.clone())
            .max_by(|a, b| date_prefix(a).cmp(date_prefix(b)))
            .unwrap_or_default();
        results.retain(|(path, ..)| *path == best_path);
    }

    let memory_matches = if include_memories && !context.json {
        fetch_memories_for_query(&project_root, &query)
            .map(|raw| parse_memory_matches(&raw))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    if context.json {
        let total = results.len();
        let limited_results = &results[..total.min(display_limit)];
        let payload = SearchPayload {
            query: query.clone(),
            results: limited_results
                .iter()
                .map(
                    |(path, line_num, line, _start, _end, context_lines)| SearchResult {
                        path: path.clone(),
                        line_number: *line_num,
                        line: line.clone(),
                        context: context_lines.clone(),
                    },
                )
                .collect(),
        };
        return print_envelope_list(payload);
    }

    if results.is_empty() && memory_matches.is_empty() {
        println!();
        println!("  {} No results found for '{}'", "○".dimmed(), query);
        println!();
        return Ok(());
    }

    if results.is_empty() {
        println!();
        println!(
            "  {} No artifact results found for '{}'",
            "○".dimmed(),
            query
        );
        println!();
    }

    if !results.is_empty() {
        let total = results.len();
        let truncated = total > display_limit;
        let display_results = &results[..total.min(display_limit)];

        println!();
        println!(
            "  {} Search results for '{}' ({} matches)",
            "◆".cyan(),
            query.bold(),
            total,
        );
        println!();

        // Compute the width needed to pad line numbers for alignment.
        // When context is shown, also account for surrounding line numbers.
        let max_line_num = display_results
            .iter()
            .map(|(_, line_num, ..)| line_num + context_size)
            .max()
            .unwrap_or(1);
        let line_num_width = max_line_num.to_string().len();

        let mut current_file = String::new();
        // last_shown_end tracks the last line number (1-based) printed for the current file,
        // so we can insert "--" separators between non-adjacent context blocks.
        let mut last_shown_end: Option<usize> = None;

        for (path, line_num, line, start, end, context_lines) in display_results {
            if *path != current_file {
                current_file = path.clone();
                last_shown_end = None;
                println!("  {}", path.cyan());
            }

            if context_size > 0 {
                // extract_context_lines uses `line_num` (0-based) as the center, so
                // the number of pre-context lines actually collected is
                // min(context_size, line_num_0based) = min(context_size, line_num - 1).
                let pre_count = context_size.min(line_num.saturating_sub(1));
                let first_line_num = line_num.saturating_sub(pre_count); // 1-based number of first context line

                // Insert separator when this block is not adjacent to the previous one.
                if let Some(prev_end) = last_shown_end
                    && first_line_num > prev_end + 1
                {
                    println!("    {}", "--".dimmed());
                }

                // Print pre-context lines (dim).
                for (i, ctx_line) in context_lines.iter().enumerate() {
                    let abs_line_num = first_line_num + i;
                    if abs_line_num == *line_num {
                        // This is the match line — print highlighted.
                        let padded_num =
                            format!("{:>width$}", abs_line_num, width = line_num_width);
                        println!(
                            "    {}:  {}",
                            padded_num.dimmed(),
                            highlight_match(ctx_line, *start, *end),
                        );
                    } else {
                        // Context line — print dim.
                        let padded_num =
                            format!("{:>width$}", abs_line_num, width = line_num_width);
                        println!("    {}:  {}", padded_num.dimmed(), ctx_line.dimmed(),);
                    }
                }

                let last_line_num = first_line_num + context_lines.len().saturating_sub(1);
                last_shown_end = Some(last_line_num);
            } else {
                let padded_num = format!("{:>width$}", line_num, width = line_num_width);
                println!(
                    "    {}:  {}",
                    padded_num.dimmed(),
                    highlight_match(line, *start, *end),
                );
            }
        }

        println!();

        if truncated {
            eprintln!(
                "Showing first {} of {} results. Use -n to see more.",
                display_limit, total
            );
        }
    }

    render_memory_matches(&memory_matches, context.verbose);

    Ok(())
}

/// Walk `dir` collecting text/regex matches, appending to `results`.
///
/// `display_base` is stripped from each file path for display; `prefix` is
/// prepended to the display path ("global:" for the user-level resources
/// walk, wai-gkk3). Files whose canonical path is already in `scanned` are
/// skipped so repo and global walks never duplicate a hit.
#[allow(clippy::too_many_arguments)]
fn collect_matches(
    dir: &Path,
    display_base: &Path,
    agents_md: &Path,
    matcher: &Matcher,
    type_filter: Option<&str>,
    tag: &[String],
    context_size: usize,
    prefix: &str,
    scanned: &mut HashSet<PathBuf>,
    results: &mut Vec<RawMatch>,
) {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "md" || ext == "yml" || ext == "yaml" || ext == "toml")
                .unwrap_or(false)
        })
        // Skip managed files (e.g. .wai/AGENTS.md) — not user artifacts
        .filter(|e| e.path() != agents_md)
    {
        let Ok(canonical) = entry.path().canonicalize() else {
            continue;
        };
        if !scanned.insert(canonical) {
            continue;
        }

        // Apply type filter
        if let Some(type_f) = type_filter {
            let path_str = entry.path().to_str().unwrap_or("");
            let matches = match type_f {
                "research" => path_str.contains("/research/"),
                "plan" | "plans" => path_str.contains("/plans/"),
                "design" | "designs" => path_str.contains("/designs/"),
                "handoff" | "handoffs" => path_str.contains("/handoffs/"),
                "review" | "reviews" => path_str.contains("/reviews/"),
                _ => true,
            };
            if !matches {
                continue;
            }
        }

        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Apply tag filter: parse YAML frontmatter and check tags.
        if !tag.is_empty() {
            let file_tags = parse_frontmatter_tags(&content);
            let matches_all = tag
                .iter()
                .all(|required| file_tags.iter().any(|ft| ft.eq_ignore_ascii_case(required)));
            if !matches_all {
                continue;
            }
        }

        for (line_num, line) in content.lines().enumerate() {
            if let Some((start, end)) = matcher(line) {
                let rel_path = entry
                    .path()
                    .strip_prefix(display_base)
                    .unwrap_or(entry.path());
                let context_lines = extract_context_lines(&content, line_num, context_size);
                results.push((
                    format!("{prefix}{}", rel_path.display()),
                    line_num + 1,
                    line.to_string(),
                    start,
                    end,
                    context_lines,
                ));
            }
        }
    }
}

/// Parse the YAML frontmatter block at the top of a file and return any tags listed.
///
/// Handles both inline list (`tags: [a, b]`) and block list (`tags:\n  - a`) forms.
/// Returns an empty vec if frontmatter is absent or malformed.
fn parse_frontmatter_tags(content: &str) -> Vec<String> {
    let body = content.trim_start();
    if !body.starts_with("---") {
        return Vec::new();
    }
    // Find the closing ---
    let rest = &body[3..];
    let end = rest
        .find("\n---")
        .unwrap_or(rest.find("\r\n---").unwrap_or(rest.len()));
    let frontmatter = &rest[..end];

    let mut tags = Vec::new();
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("tags:") {
            let value = value.trim();
            if value.starts_with('[') {
                // Inline list: tags: [a, b, c]
                let inner = value.trim_start_matches('[').trim_end_matches(']');
                for tag in inner.split(',') {
                    let t = tag.trim().to_string();
                    if !t.is_empty() {
                        tags.push(t);
                    }
                }
            }
        } else if line.starts_with("- ") && !tags.is_empty() {
            // Block list items that follow a `tags:` key
            // (simple heuristic: accumulate while we're still in a list context)
            tags.push(line[2..].trim().to_string());
        }
    }
    tags
}

/// Extract the YYYY-MM-DD date prefix from a file path, if present.
fn date_prefix(path: &str) -> &str {
    // Take the filename component and return up to 10 chars (YYYY-MM-DD).
    let name = path.rsplit('/').next().unwrap_or(path);
    if name.len() >= 10 && name.chars().nth(4) == Some('-') && name.chars().nth(7) == Some('-') {
        &name[..10]
    } else {
        ""
    }
}

fn render_memory_matches(matches: &[MemoryMatch], verbose: u8) {
    if matches.is_empty() {
        return;
    }

    println!();
    println!("  {} Memories", "◆".cyan());
    println!();
    for memory in matches {
        println!(
            "  {}  {}",
            "[mem]".dimmed(),
            format_memory_match(memory, verbose)
        );
    }
    println!();
}

fn format_memory_match(memory: &MemoryMatch, verbose: u8) -> String {
    if memory.value.is_empty() {
        return memory.raw.clone();
    }

    let value = if verbose > 0 {
        memory.value.clone()
    } else {
        truncate_with_ellipsis(&memory.value, 80)
    };

    format!("{}: {}", memory.key, value)
}

fn parse_memory_matches(raw: &str) -> Vec<MemoryMatch> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(parse_memory_match)
        .collect()
}

fn parse_memory_match(line: &str) -> MemoryMatch {
    for separator in [": ", "\t", " = ", "="] {
        if let Some((key, value)) = line.split_once(separator) {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if !key.is_empty() && !value.is_empty() {
                return MemoryMatch {
                    key,
                    value,
                    raw: line.to_string(),
                };
            }
        }
    }

    MemoryMatch {
        key: line.to_string(),
        value: String::new(),
        raw: line.to_string(),
    }
}

fn truncate_with_ellipsis(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }
    let take = max_chars.saturating_sub(1);
    format!("{}…", text.chars().take(take).collect::<String>())
}

fn extract_context_lines(content: &str, line_num: usize, context: usize) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let start = line_num.saturating_sub(context);
    let end = (line_num + context + 1).min(lines.len());
    lines[start..end]
        .iter()
        .map(|line| (*line).to_string())
        .collect()
}

fn highlight_match(line: &str, start: usize, end: usize) -> String {
    if start <= line.len()
        && end <= line.len()
        && line.is_char_boundary(start)
        && line.is_char_boundary(end)
    {
        let before = &line[..start];
        let matched = &line[start..end];
        let after = &line[end..];
        format!("{}{}{}", before, matched.yellow().bold(), after)
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_inline_tags() {
        let content = "---\ntags: [rust, performance]\n---\n\ncontent here";
        let tags = parse_frontmatter_tags(content);
        assert_eq!(tags, vec!["rust", "performance"]);
    }

    #[test]
    fn parse_tags_no_frontmatter() {
        let content = "# Just a heading\n\nno frontmatter";
        assert!(parse_frontmatter_tags(content).is_empty());
    }

    #[test]
    fn parse_tags_malformed_frontmatter() {
        let content = "---\nnot: valid yaml: [[\n---\ncontent";
        // Should not panic, just return empty or whatever was parseable
        let _ = parse_frontmatter_tags(content);
    }

    #[test]
    fn highlight_match_multibyte_no_panic() {
        // 'İ' (U+0130) is 2 bytes but lowercases to 'i' (1 byte),
        // so byte offsets from the lowercased string are wrong for the original.
        // This must not panic.
        let line = "İstanbul";
        let result = highlight_match(line, 0, 1);
        // Falls back to plain line when boundaries are invalid
        assert!(!result.is_empty());

        // ASCII still highlights correctly
        let line2 = "hello world";
        let result2 = highlight_match(line2, 6, 11);
        assert!(result2.contains("world"));
    }

    #[test]
    fn validate_type_filter_accepts_known_values() {
        for known in [
            "research", "plan", "plans", "design", "designs", "handoff", "handoffs", "review",
            "reviews",
        ] {
            assert!(
                validate_type_filter(Some(known)).is_ok(),
                "{known} should be accepted"
            );
        }
    }

    #[test]
    fn validate_type_filter_rejects_unknown_values() {
        for unknown in ["blog", "epic", "specs", "notes", ""] {
            assert!(
                validate_type_filter(Some(unknown)).is_err(),
                "{unknown:?} should be rejected"
            );
        }
    }

    #[test]
    fn date_prefix_extracts_correctly() {
        assert_eq!(
            date_prefix(".wai/projects/p/research/2026-02-25-notes.md"),
            "2026-02-25"
        );
        assert_eq!(date_prefix("no-date-here.md"), "");
        assert_eq!(date_prefix("2025-01-01-something.md"), "2025-01-01");
    }
}
