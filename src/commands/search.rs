use miette::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::{projects_dir, wai_dir};
use crate::context::current_context;
use crate::json::{SearchPayload, SearchResult};
use crate::output::print_json;
use crate::plugin::fetch_memories_for_query;

use super::require_project;

const DEFAULT_LIMIT: usize = 20;

pub fn run(
    query: String,
    type_filter: Option<String>,
    project: Option<String>,
    use_regex: bool,
    limit: Option<usize>,
    tag_filter: Vec<String>,
    latest: bool,
    context_size: usize,
    include_memories: bool,
) -> Result<()> {
    let display_limit = limit.unwrap_or(DEFAULT_LIMIT);
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

    type Matcher = Box<dyn Fn(&str) -> Option<(usize, usize)>>;
    let matcher: Matcher = if use_regex {
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
    let mut results: Vec<(String, usize, String, usize, usize, Vec<String>)> = Vec::new();

    for entry in WalkDir::new(&search_root)
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
    {
        // Apply type filter
        if let Some(ref type_f) = type_filter {
            let path_str = entry.path().to_str().unwrap_or("");
            let matches = match type_f.as_str() {
                "research" => path_str.contains("/research/"),
                "plan" | "plans" => path_str.contains("/plans/"),
                "design" | "designs" => path_str.contains("/designs/"),
                "handoff" | "handoffs" => path_str.contains("/handoffs/"),
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
        if !tag_filter.is_empty() {
            let file_tags = parse_frontmatter_tags(&content);
            let matches_all = tag_filter
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
                    .strip_prefix(&project_root)
                    .unwrap_or(entry.path());
                let context_lines = extract_context_lines(&content, line_num, context_size);
                results.push((
                    rel_path.display().to_string(),
                    line_num + 1,
                    line.to_string(),
                    start,
                    end,
                    context_lines,
                ));
            }
        }
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

    if context.json {
        let payload = SearchPayload {
            query: query.clone(),
            results: results
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
        return print_json(&payload);
    }

    if results.is_empty() {
        println!();
        println!("  {} No results found for '{}'", "○".dimmed(), query);
        println!();
        return Ok(());
    }

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
            if let Some(prev_end) = last_shown_end {
                if first_line_num > prev_end + 1 {
                    println!("    {}", "--".dimmed());
                }
            }

            // Print pre-context lines (dim).
            for (i, ctx_line) in context_lines.iter().enumerate() {
                let abs_line_num = first_line_num + i;
                if abs_line_num == *line_num {
                    // This is the match line — print highlighted.
                    let padded_num = format!("{:>width$}", abs_line_num, width = line_num_width);
                    println!(
                        "    {}:  {}",
                        padded_num.dimmed(),
                        highlight_match(ctx_line, *start, *end),
                    );
                } else {
                    // Context line — print dim.
                    let padded_num = format!("{:>width$}", abs_line_num, width = line_num_width);
                    println!(
                        "    {}:  {}",
                        padded_num.dimmed(),
                        ctx_line.dimmed(),
                    );
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

    // bd memories section — shown when --include-memories is passed
    if include_memories && !context.json {
        if let Some(mem_raw) = fetch_memories_for_query(&project_root, &query) {
            let mem_lines: Vec<&str> = mem_raw
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect();
            if !mem_lines.is_empty() {
                println!();
                println!("  {} Memories", "◆".cyan());
                println!();
                for line in &mem_lines {
                    println!("  {}  {}", "[mem]".dimmed(), line);
                }
                println!();
            }
        }
    }

    Ok(())
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
    fn date_prefix_extracts_correctly() {
        assert_eq!(
            date_prefix(".wai/projects/p/research/2026-02-25-notes.md"),
            "2026-02-25"
        );
        assert_eq!(date_prefix("no-date-here.md"), "");
        assert_eq!(date_prefix("2025-01-01-something.md"), "2025-01-01");
    }
}
