use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;

use crate::config::pipelines_dir;
use crate::context::current_context;
use crate::json::PipelineCurrentPayload;
use crate::output::print_json;

use super::definition::{list_pipeline_names, load_pipeline_toml, validate_pipeline};
use super::gates::{
    evaluate_gates, find_step_artifacts, format_gate_summary, parse_frontmatter, print_gate_status,
};
use super::orchestration::{pipeline_current_status, resolve_active_run_id};
use super::{PipelineDefinition, PipelineRun, ValidationLevel};

use crate::commands::require_project;

// ─── list ────────────────────────────────────────────────────────────────────

pub(super) fn cmd_list() -> Result<()> {
    let project_root = require_project()?;
    let pipelines = pipelines_dir(&project_root);

    if !pipelines.exists() {
        println!();
        println!("  {} No pipelines defined", "○".dimmed());
        println!("  {} Create one with: wai pipeline init <name>", "→".cyan());
        println!();
        return Ok(());
    }

    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(&pipelines).into_diagnostic()?.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("toml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            names.push(stem.to_string());
        }
    }

    if names.is_empty() {
        println!();
        println!("  {} No pipelines defined", "○".dimmed());
        println!("  {} Create one with: wai pipeline init <name>", "→".cyan());
        println!();
        return Ok(());
    }

    names.sort();

    println!();
    println!("  {} Pipelines", "◆".cyan());
    println!();
    for name in &names {
        // Load step count from TOML definition
        let def_path = pipelines.join(format!("{}.toml", name));
        let step_info = match load_pipeline_toml(&def_path) {
            Ok(d) => format!("{} steps", d.steps.len()),
            Err(_) => "(invalid)".to_string(),
        };
        println!(
            "    {} {}  {}",
            "•".dimmed(),
            name.bold(),
            step_info.dimmed(),
        );
    }
    println!();

    Ok(())
}

// ─── current ──────────────────────────────────────────────────────────────────

pub(super) fn cmd_current(json: bool) -> Result<()> {
    let json = json || current_context().json;
    let project_root = require_project()?;

    let Some(status) = pipeline_current_status(&project_root)? else {
        if json {
            return print_json(&PipelineCurrentPayload {
                active: false,
                message: Some(
                    "No active pipeline run. Start one with: wai pipeline start <name> --topic=<topic>"
                        .to_string(),
                ),
                pipeline: None,
                run_id: None,
                topic: None,
                step: None,
                gate_summary: None,
                next_command: Some("wai pipeline start <name> --topic=<topic>".to_string()),
            });
        }
        miette::bail!(
            "No active pipeline run. Start one with: wai pipeline start <name> --topic=<topic>"
        );
    };

    if json {
        return print_json(&status);
    }

    if let Some(ref step) = status.step {
        println!(
            "── step {}/{}: {} ──────────────────────────────",
            step.index, step.total, step.id
        );
        println!("{}", step.prompt);

        let addenda = find_step_addenda(&project_root, &step.id);
        if !addenda.is_empty() {
            println!();
            println!("{} Addenda ({})", "◆".cyan(), addenda.len());
            for path in &addenda {
                println!("  {} {}", "•".dimmed(), path);
            }
        }
    } else {
        println!("──────────────────────────────────────────────");
        println!(
            "Pipeline '{}' is already complete!",
            status.pipeline.unwrap()
        );
        println!();
        println!(
            "Next: {}",
            status
                .next_command
                .unwrap_or_else(|| "wai close".to_string())
        );
    }

    Ok(())
}

// ─── suggest ──────────────────────────────────────────────────────────────────

pub(super) fn cmd_suggest(description: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    let pipelines = pipelines_dir(&project_root);

    // Collect all valid pipeline TOMLs
    let mut found: Vec<(String, PipelineDefinition)> = vec![];
    if pipelines.exists() {
        for entry in fs::read_dir(&pipelines).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
            {
                match load_pipeline_toml(&path) {
                    Ok(def) => found.push((name.to_string(), def)),
                    Err(e) => eprintln!("warning: skipping {}: {}", path.display(), e),
                }
            }
        }
    }

    if found.is_empty() {
        println!("No pipelines defined.");
        println!();
        println!("Create one with: wai pipeline init <name>");
        return Ok(());
    }

    // Normalize description: treat empty string as None
    let query = description
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_lowercase());

    // Score and sort
    if let Some(ref q) = query {
        let words: Vec<&str> = q.split_whitespace().collect();
        found.sort_by(|(a_name, a_def), (b_name, b_def)| {
            let score_a = score_pipeline(a_name, a_def, &words);
            let score_b = score_pipeline(b_name, b_def, &words);
            // Descending score, then ascending name for ties
            score_b.cmp(&score_a).then(a_name.cmp(b_name))
        });
    } else {
        found.sort_by(|(a, _), (b, _)| a.cmp(b));
    }

    // Print each pipeline
    for (name, def) in &found {
        let desc = def.description.as_deref().unwrap_or("(no description)");
        let steps = def.steps.len();
        println!("  {} — {} ({} steps)", name, desc, steps);
    }

    println!();
    let top_name = &found[0].0;
    println!(
        "Start: wai pipeline start {} --topic=<your-topic>",
        top_name
    );

    Ok(())
}

/// Score a pipeline by counting how many query words appear in its name + description.
fn score_pipeline(name: &str, def: &PipelineDefinition, words: &[&str]) -> usize {
    let haystack = format!("{} {}", name, def.description.as_deref().unwrap_or("")).to_lowercase();
    words.iter().filter(|w| haystack.contains(*w)).count()
}

// ─── show ─────────────────────────────────────────────────────────────────────

pub(super) fn cmd_show(name: &str) -> Result<()> {
    let project_root = require_project()?;
    let def_path = pipelines_dir(&project_root).join(format!("{}.toml", name));
    if !def_path.exists() {
        let available = list_pipeline_names(&project_root);
        if available.is_empty() {
            miette::bail!("Pipeline '{}' not found. No pipelines defined.", name);
        } else {
            miette::bail!(
                "Pipeline '{}' not found. Available: {}",
                name,
                available.join(", ")
            );
        }
    }
    let def = load_pipeline_toml(&def_path)?;

    // Header
    println!();
    println!("  {} {}", "◆".cyan(), def.name.bold());
    if let Some(ref desc) = def.description {
        println!("  {}", desc.dimmed());
    }

    // Metadata
    if let Some(ref meta) = def.metadata {
        println!();
        if let Some(ref when) = meta.when {
            println!("  {} When: {}", "•".dimmed(), when);
        }
        if !meta.skills.is_empty() {
            println!("  {} Skills: {}", "•".dimmed(), meta.skills.join(", "));
        }
    }

    // Steps
    println!();
    println!("  {} Steps ({}):", "◆".cyan(), def.steps.len());
    for (i, step) in def.steps.iter().enumerate() {
        let gate_summary = format_gate_summary(&step.gate);
        if gate_summary.is_empty() {
            println!(
                "    {}. {} {}",
                i + 1,
                step.id.bold(),
                "(no gates)".dimmed()
            );
        } else {
            println!(
                "    {}. {} {}",
                i + 1,
                step.id.bold(),
                gate_summary.dimmed()
            );
        }
    }

    // Oracle directory
    let oracles_dir = crate::config::wai_dir(&project_root)
        .join("resources")
        .join("oracles");
    println!();
    println!("  {} Oracles: {}", "•".dimmed(), oracles_dir.display());
    println!();

    Ok(())
}

// ─── gates ────────────────────────────────────────────────────────────────────

pub(super) fn cmd_gates(name: Option<&str>, step_filter: Option<&str>) -> Result<()> {
    let project_root = require_project()?;

    // Try to resolve active run for live status
    let active_run = resolve_active_run_id(&project_root).ok();

    if let Some(ref run_id) = active_run
        && name.is_none()
    {
        // Active run, no explicit pipeline name — show live status for current step
        let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
        let run_path = runs_dir.join(format!("{}.yml", run_id));
        let run: PipelineRun =
            serde_yml::from_str(&fs::read_to_string(&run_path).into_diagnostic()?)
                .map_err(|e| miette::miette!("Failed to parse run state: {}", e))?;
        let def_path = pipelines_dir(&project_root).join(format!("{}.toml", run.pipeline));
        let definition = load_pipeline_toml(&def_path)?;

        if run.current_step >= definition.steps.len() {
            miette::bail!("Pipeline run is complete.");
        }

        let step = &definition.steps[run.current_step];
        print_gate_status(step, Some(&run), Some(&definition), &project_root)?;
    } else if let Some(pipeline_name) = name {
        // Show gate definitions (not live status)
        let def_path = pipelines_dir(&project_root).join(format!("{}.toml", pipeline_name));
        if !def_path.exists() {
            miette::bail!(
                "Pipeline '{}' not found. Specify a pipeline name: wai pipeline gates <name>",
                pipeline_name
            );
        }
        let definition = load_pipeline_toml(&def_path)?;

        if let Some(step_id) = step_filter {
            let step = definition
                .steps
                .iter()
                .find(|s| s.id == step_id)
                .ok_or_else(|| {
                    miette::miette!(
                        "Step '{}' not found in pipeline '{}'. Steps: {}",
                        step_id,
                        pipeline_name,
                        definition
                            .steps
                            .iter()
                            .map(|s| s.id.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;
            print_gate_status(step, None, None, &project_root)?;
        } else {
            // Show all steps' gates
            for step in &definition.steps {
                print_gate_status(step, None, None, &project_root)?;
            }
        }
    } else {
        miette::bail!("No active pipeline run. Specify a pipeline name: wai pipeline gates <name>");
    }

    Ok(())
}

// ─── check ────────────────────────────────────────────────────────────────────

pub(super) fn cmd_check(oracle_name: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    let run_id = resolve_active_run_id(&project_root)?;

    let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    if !run_path.exists() {
        miette::bail!("No active pipeline run.");
    }
    let run: PipelineRun = serde_yml::from_str(&fs::read_to_string(&run_path).into_diagnostic()?)
        .map_err(|e| miette::miette!("Failed to parse run state: {}", e))?;

    let def_path = pipelines_dir(&project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    if run.current_step >= definition.steps.len() {
        miette::bail!("Pipeline run is complete.");
    }

    let step = &definition.steps[run.current_step];

    if let Some(oracle_filter) = oracle_name {
        // Single oracle mode
        let step_artifacts = find_step_artifacts(&project_root, &run.run_id, &step.id);
        let oracle = step
            .gate
            .as_ref()
            .and_then(|g| g.oracles.iter().find(|o| o.name == oracle_filter));

        let Some(oracle) = oracle else {
            miette::bail!(
                "Oracle '{}' not configured for step '{}'. Available oracles: {}",
                oracle_filter,
                step.id,
                step.gate
                    .as_ref()
                    .map(|g| g
                        .oracles
                        .iter()
                        .map(|o| o.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "))
                    .unwrap_or_default()
            );
        };

        let failures = super::gates::run_oracle(oracle, &step_artifacts, &project_root)?;
        println!();
        if failures.is_empty() {
            println!("  {} Oracle '{}': PASS", "✓".green(), oracle_filter);
        } else {
            for f in &failures {
                println!("  {} {}", "✗".red(), f);
            }
        }
        println!();
        return Ok(());
    }

    // Full gate check
    let Some(ref gate) = step.gate else {
        println!();
        println!("  No gates configured for step '{}'. Result: PASS", step.id);
        println!();
        return Ok(());
    };

    let failures = evaluate_gates(gate, step, &run, &definition, &project_root)?;

    println!();
    println!("  {} Gate check for step '{}':", "◆".cyan(), step.id);
    println!();

    if failures.is_empty() {
        println!("  {} Result: PASS", "✓".green());
    } else {
        for f in &failures {
            println!("  {} {}", "✗".red(), f);
        }
        println!();
        println!(
            "  {} Result: BLOCKED — resolve {} failure(s)",
            "✗".red(),
            failures.len()
        );
    }
    println!();

    Ok(())
}

// ─── validate ─────────────────────────────────────────────────────────────────

pub(super) fn cmd_validate(name: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    let pipelines = pipelines_dir(&project_root);

    if !pipelines.exists() {
        miette::bail!("No pipelines directory found.");
    }

    let targets: Vec<(String, std::path::PathBuf)> = if let Some(name) = name {
        let path = pipelines.join(format!("{}.toml", name));
        if !path.exists() {
            let available = list_pipeline_names(&project_root);
            miette::bail!(
                "Pipeline '{}' not found. Available: {}",
                name,
                available.join(", ")
            );
        }
        vec![(name.to_string(), path)]
    } else {
        // Validate all
        let mut targets = Vec::new();
        for entry in fs::read_dir(&pipelines).into_diagnostic()?.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                targets.push((stem.to_string(), path));
            }
        }
        targets.sort_by(|a, b| a.0.cmp(&b.0));
        targets
    };

    let mut had_errors = false;

    for (pname, path) in &targets {
        match load_pipeline_toml(path) {
            Err(e) => {
                cliclack::log::error(format!("{}: {}", pname, e)).into_diagnostic()?;
                had_errors = true;
            }
            Ok(def) => {
                let issues = validate_pipeline(&def, &project_root);
                let errors: Vec<_> = issues
                    .iter()
                    .filter(|i| i.level == ValidationLevel::Error)
                    .collect();
                let warnings: Vec<_> = issues
                    .iter()
                    .filter(|i| i.level == ValidationLevel::Warn)
                    .collect();

                if errors.is_empty() && warnings.is_empty() {
                    let gate_count = def.steps.iter().filter(|s| s.gate.is_some()).count();
                    cliclack::log::success(format!(
                        "{}: {} steps, {} gated",
                        pname,
                        def.steps.len(),
                        gate_count
                    ))
                    .into_diagnostic()?;
                } else {
                    for e in &errors {
                        cliclack::log::error(format!("{}: {}", pname, e.message))
                            .into_diagnostic()?;
                        had_errors = true;
                    }
                    for w in &warnings {
                        cliclack::log::warning(format!("{}: {}", pname, w.message))
                            .into_diagnostic()?;
                    }
                }
            }
        }
    }

    if had_errors {
        std::process::exit(1);
    }

    Ok(())
}

// ─── print_step ───────────────────────────────────────────────────────────────

/// Print a step prompt block with a "step N/M" header and rendered prompt.
pub(super) fn print_step(definition: &PipelineDefinition, idx: usize, topic: &str) {
    let step = &definition.steps[idx];
    let total = definition.steps.len();
    println!(
        "── step {}/{}: {} ──────────────────────────────",
        idx + 1,
        total,
        step.id
    );
    println!("{}", super::render_prompt(&step.prompt, topic));
}

// ─── find_step_addenda ────────────────────────────────────────────────────────

/// Find addenda for a given pipeline step by scanning artifact directories.
///
/// An addendum is an artifact whose YAML frontmatter contains a
/// `pipeline-addendum:<step_id>` tag. Returns relative paths like
/// `research/2026-04-14-correction.md`.
pub(super) fn find_step_addenda(project_root: &Path, step_id: &str) -> Vec<String> {
    let addendum_tag = format!("pipeline-addendum:{}", step_id);
    let projects = crate::config::projects_dir(project_root);

    let mut addenda = Vec::new();

    let Ok(entries) = fs::read_dir(&projects) else {
        return addenda;
    };
    for entry in entries.flatten() {
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }
        for dir_name in &["research", "plans", "designs", "handoffs", "reviews"] {
            let dir = project_dir.join(dir_name);
            if !dir.exists() {
                continue;
            }
            let Ok(files) = fs::read_dir(&dir) else {
                continue;
            };
            for file_entry in files.flatten() {
                let path = file_entry.path();
                if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let Ok(content) = fs::read_to_string(&path) else {
                    continue;
                };
                let fm = parse_frontmatter(&content);
                if fm.tags.contains(&addendum_tag) {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    addenda.push(format!("{}/{}", dir_name, filename));
                }
            }
        }
    }
    addenda.sort();
    addenda
}
