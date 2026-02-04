use miette::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::{wai_dir, projects_dir};

use super::require_project;

pub fn run(query: String, type_filter: Option<String>, project: Option<String>) -> Result<()> {
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

    let query_lower = query.to_lowercase();
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
                if line.to_lowercase().contains(&query_lower) {
                    let rel_path = entry
                        .path()
                        .strip_prefix(&project_root)
                        .unwrap_or(entry.path());
                    results.push((
                        rel_path.display().to_string(),
                        line_num + 1,
                        line.to_string(),
                    ));
                }
            }
        }
    }

    if results.is_empty() {
        println!();
        println!("  {} No results found for '{}'", "○".dimmed(), query);
        println!();
        return Ok(());
    }

    println!();
    println!(
        "  {} Search results for '{}' ({} matches)",
        "◆".cyan(),
        query.bold(),
        results.len()
    );
    println!();

    let mut current_file = String::new();
    for (path, line_num, line) in &results {
        if *path != current_file {
            current_file = path.clone();
            println!("  {}", path.cyan());
        }
        println!(
            "    {}:  {}",
            line_num.to_string().dimmed(),
            highlight_match(line, &query)
        );
    }

    println!();
    Ok(())
}

fn highlight_match(line: &str, query: &str) -> String {
    let lower = line.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(pos) = lower.find(&query_lower) {
        let before = &line[..pos];
        let matched = &line[pos..pos + query.len()];
        let after = &line[pos + query.len()..];
        format!("{}{}{}", before, matched.yellow().bold(), after)
    } else {
        line.to_string()
    }
}
