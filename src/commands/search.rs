use miette::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::{wai_dir, projects_dir};

use super::require_project;

pub fn run(
    query: String,
    type_filter: Option<String>,
    project: Option<String>,
    use_regex: bool,
    limit: Option<usize>,
) -> Result<()> {
    let project_root = require_project()?;

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
        Box::new(move |line: &str| {
            re.find(line).map(|m| (m.start(), m.end()))
        })
    } else {
        let query_lower = query.to_lowercase();
        Box::new(move |line: &str| {
            let lower = line.to_lowercase();
            lower.find(&query_lower).map(|pos| (pos, pos + query_lower.len()))
        })
    };

    let mut results = Vec::new();

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

        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for (line_num, line) in content.lines().enumerate() {
                if let Some((start, end)) = matcher(line) {
                    let rel_path = entry
                        .path()
                        .strip_prefix(&project_root)
                        .unwrap_or(entry.path());
                    results.push((
                        rel_path.display().to_string(),
                        line_num + 1,
                        line.to_string(),
                        start,
                        end,
                    ));

                    if let Some(max) = limit
                        && results.len() >= max
                    {
                        break;
                    }
                }
            }
        }

        if let Some(max) = limit
            && results.len() >= max
        {
            break;
        }
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
    for (path, line_num, line, start, end) in &results {
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
