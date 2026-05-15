use miette::{IntoDiagnostic, Result};

use crate::cli::ArtifactsCommands;
use crate::freshness::{FreshnessReport, scan_freshness};

use super::require_project;

pub fn run(cmd: ArtifactsCommands) -> Result<()> {
    match cmd {
        ArtifactsCommands::Stale { json } => run_stale(json),
    }
}

fn run_stale(json: bool) -> Result<()> {
    let project_root = require_project()?;
    let report = scan_freshness(&project_root);

    if json {
        let out = serde_json::to_string_pretty(&report).into_diagnostic()?;
        println!("{}", out);
    } else {
        print_human(&report);
    }
    Ok(())
}

fn print_human(report: &FreshnessReport) {
    if report.stale.is_empty() && report.untracked.is_empty() {
        println!("All {} tracked artifact(s) are current.", report.clean);
        return;
    }

    if !report.stale.is_empty() {
        println!("Stale artifacts ({}):", report.stale_count);
        for entry in &report.stale {
            println!("  stale  {}", entry.artifact);
            for path in &entry.changed_paths {
                println!("         changed: {}", path);
            }
        }
    }

    if !report.untracked.is_empty() {
        println!("Untracked artifacts (no sidecar yet):");
        for art in &report.untracked {
            println!("  untracked  {}", art);
        }
    }

    println!(
        "\nSummary: {} stale, {} untracked, {} clean",
        report.stale_count,
        report.untracked.len(),
        report.clean
    );
}
