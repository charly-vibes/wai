//! Terminal and JSON rendering for `wai why` responses.

use owo_colors::OwoColorize;

use super::parsing::{ParsedResponse, Relevance};
use super::shared::section_header;

/// Pretty-print the parsed response to stdout with colors and icons.
pub fn format_terminal(response: &ParsedResponse, query: &str) {
    println!();
    println!("  {} {}", "◆".cyan(), query.bold());
    println!();

    // Answer
    section_header("Answer");
    println!();
    for line in response.answer.lines() {
        println!("  {}", line);
    }
    println!();

    // Relevant Artifacts
    if !response.relevant_artifacts.is_empty() {
        section_header("Relevant Artifacts");
        println!();
        for artifact in &response.relevant_artifacts {
            let relevance_display = match &artifact.relevance {
                Some(r) => format!("{} [{}]", r.icon(), r.as_str()),
                None => "○".to_string(),
            };
            let colored = match &artifact.relevance {
                Some(Relevance::High) => relevance_display.red().to_string(),
                Some(Relevance::Medium) => relevance_display.yellow().to_string(),
                Some(Relevance::Low) => relevance_display.green().to_string(),
                None => relevance_display.dimmed().to_string(),
            };
            // file:line format makes paths clickable in supporting terminals
            let clickable_path = format!("{}:1", artifact.path);
            println!("  {}  {}", colored, clickable_path.cyan());
            if !artifact.description.is_empty() {
                println!("     {}", artifact.description.dimmed());
            }
            println!();
        }
    }

    // Decision Chain
    if !response.decision_chain.is_empty() {
        section_header("Decision Chain");
        println!();
        for line in response.decision_chain.lines() {
            println!("  {}", line);
        }
        println!();
    }

    // Suggestions
    if !response.suggestions.is_empty() {
        section_header("Suggestions");
        println!();
        for suggestion in &response.suggestions {
            println!("  {} {}", "→".cyan(), suggestion);
        }
        println!();
    }
}

// ── JSON formatter ─────────────────────────────────────────────────────────────

/// Serialize the parsed response as JSON for machine-readable output.
pub fn format_json(response: &ParsedResponse, query: &str) -> String {
    use serde_json::{Value, json};

    let artifacts: Vec<Value> = response
        .relevant_artifacts
        .iter()
        .map(|a| {
            json!({
                "path": a.path,
                "relevance": a.relevance.as_ref().map(|r| r.as_str()),
                "description": a.description,
            })
        })
        .collect();

    let v = json!({
        "query": query,
        "answer": response.answer,
        "relevant_artifacts": artifacts,
        "decision_chain": response.decision_chain,
        "suggestions": response.suggestions,
    });

    serde_json::to_string_pretty(&v).unwrap_or_else(|_| response.raw.clone())
}
