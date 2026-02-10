use miette::Result;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::projects_dir;
use crate::context::current_context;
use crate::json::{TimelineEntry as JsonTimelineEntry, TimelinePayload};
use crate::output::print_json;

use super::require_project;

struct TimelineEntry {
    date: String,
    artifact_type: String,
    title: String,
    path: String,
}

pub fn run(project: String, from: Option<String>, to: Option<String>, reverse: bool) -> Result<()> {
    let project_root = require_project()?;
    let proj_dir = projects_dir(&project_root).join(&project);
    let context = current_context();

    if !proj_dir.exists() {
        return Err(crate::error::WaiError::ProjectNotFound {
            name: project.clone(),
        }
        .into());
    }

    let mut entries: Vec<TimelineEntry> = Vec::new();

    for entry in WalkDir::new(&proj_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
    {
        let filename = entry.file_name().to_str().unwrap_or("").to_string();

        // Extract date prefix (YYYY-MM-DD)
        let date = if filename.len() >= 10 && filename.chars().nth(4) == Some('-') {
            filename[..10].to_string()
        } else {
            continue;
        };

        // Apply date range filters
        if let Some(ref from_date) = from
            && date.as_str() < from_date.as_str()
        {
            continue;
        }
        if let Some(ref to_date) = to
            && date.as_str() > to_date.as_str()
        {
            continue;
        }

        // Determine artifact type from parent directory
        let parent = entry
            .path()
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let artifact_type = match parent {
            "research" => "research",
            "plans" => "plan",
            "designs" => "design",
            "handoffs" => "handoff",
            _ => "other",
        };

        // Title from filename (strip date and extension)
        let title = filename
            .strip_prefix(&format!("{}-", date))
            .unwrap_or(&filename)
            .strip_suffix(".md")
            .unwrap_or(&filename)
            .replace('-', " ");

        let path = entry
            .path()
            .strip_prefix(&project_root)
            .unwrap_or(entry.path())
            .display()
            .to_string();

        entries.push(TimelineEntry {
            date,
            artifact_type: artifact_type.to_string(),
            title,
            path,
        });
    }

    // Sort by date
    if reverse {
        entries.sort_by(|a, b| a.date.cmp(&b.date));
    } else {
        entries.sort_by(|a, b| b.date.cmp(&a.date));
    }

    if context.json {
        let payload = TimelinePayload {
            project: project.clone(),
            entries: entries
                .iter()
                .map(|entry| JsonTimelineEntry {
                    date: entry.date.clone(),
                    artifact_type: entry.artifact_type.clone(),
                    title: entry.title.clone(),
                    path: entry.path.clone(),
                })
                .collect(),
        };
        return print_json(&payload);
    }

    if entries.is_empty() {
        println!();
        println!(
            "  {} No dated artifacts found for '{}'",
            "○".dimmed(),
            project
        );
        println!(
            "  {} Add artifacts with: wai add research \"...\"",
            "→".dimmed()
        );
        println!();
        return Ok(());
    }

    println!();
    println!("  {} Timeline for '{}'", "◆".cyan(), project.bold());
    println!();

    let mut current_date = String::new();
    for entry in &entries {
        if entry.date != current_date {
            current_date = entry.date.clone();
            println!("  {}", current_date.bold());
        }

        let type_label = format_type(&entry.artifact_type);
        println!("    {} [{}] {}", "•".dimmed(), type_label, entry.title);
    }

    println!();
    Ok(())
}

fn format_type(t: &str) -> String {
    match t {
        "research" => "research".yellow().to_string(),
        "plan" => "plan".blue().to_string(),
        "design" => "design".magenta().to_string(),
        "handoff" => "handoff".green().to_string(),
        _ => t.dimmed().to_string(),
    }
}
