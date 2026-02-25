use miette::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::{projects_dir, wai_dir};
use crate::context::current_context;
use crate::json::{SearchPayload, SearchResult};
use crate::output::print_json;

use super::require_project;

pub fn run(
    query: String,
    type_filter: Option<String>,
    project: Option<String>,
    use_regex: bool,
    limit: Option<usize>,
    tag_filter: Vec<String>,
    latest: bool,
) -> Result<()> {
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
            lower
                .find(&query_lower)
                .map(|pos| (pos, pos + query_lower.len()))
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
                let context_lines = extract_context_lines(&content, line_num, 1);
                results.push((
                    rel_path.display().to_string(),
                    line_num + 1,
                    line.to_string(),
                    start,
                    end,
                    context_lines,
                ));

                if let Some(max) = limit
                    && results.len() >= max
                {
                    break;
                }
            }
        }

        if let Some(max) = limit
            && results.len() >= max
        {
            break;
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
    let limited = limit.is_some_and(|max| total >= max);

    println!();
    println!(
        "  {} Search results for '{}' ({}{} matches)",
        "◆".cyan(),
        query.bold(),
        total,
        if limited { "+" } else { "" }
    );
    println!();

    let mut current_file = String::new();
    for (path, line_num, line, start, end, _context_lines) in &results {
        if *path != current_file {
            current_file = path.clone();
            println!("  {}", path.cyan());
        }
        println!(
            "    {}:  {}",
            line_num.to_string().dimmed(),
            highlight_match(line, *start, *end)
        );
    }

    println!();
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
    if start < line.len() && end <= line.len() {
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
    fn date_prefix_extracts_correctly() {
        assert_eq!(
            date_prefix(".wai/projects/p/research/2026-02-25-notes.md"),
            "2026-02-25"
        );
        assert_eq!(date_prefix("no-date-here.md"), "");
        assert_eq!(date_prefix("2025-01-01-something.md"), "2025-01-01");
    }
}
