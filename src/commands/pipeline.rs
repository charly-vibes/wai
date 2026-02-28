use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::cli::PipelineCommands;
use crate::config::{SKILLS_DIR, agent_config_dir, pipelines_dir, wai_dir};
use crate::context::require_safe_mode;

use super::require_project;

// ─── Data structures ─────────────────────────────────────────────────────────

/// A pipeline definition stored at `.wai/resources/pipelines/<name>.yml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineDefinition {
    pub name: String,
    pub stages: Vec<PipelineStage>,
}

/// One stage in a pipeline definition: a skill name and expected artifact type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub skill: String,
    pub artifact: String,
}

/// Run state stored at `.wai/resources/pipelines/<name>/runs/<run-id>.yml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineRun {
    pub run_id: String,
    pub pipeline: String,
    pub topic: String,
    pub created_at: String,
    /// Index of the current (not-yet-completed) stage; equals `stages.len()` when done.
    pub current_stage: usize,
    pub stages: Vec<RunStage>,
}

/// Per-stage state within a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStage {
    pub skill: String,
    pub artifact: String,
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_path: Option<String>,
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn run(cmd: PipelineCommands) -> Result<()> {
    match cmd {
        PipelineCommands::Create { name, stages } => cmd_create(&name, &stages),
        PipelineCommands::Run { name, topic } => cmd_run(&name, &topic),
        PipelineCommands::Advance { run_id } => cmd_advance(&run_id),
        PipelineCommands::Status { name, run } => cmd_status(&name, run.as_deref()),
        PipelineCommands::List => cmd_list(),
    }
}

// ─── create ──────────────────────────────────────────────────────────────────

fn cmd_create(name: &str, stages_str: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline create")?;

    validate_pipeline_name(name)?;

    let stages = parse_stages(stages_str)?;
    if stages.is_empty() {
        miette::bail!("At least one stage is required");
    }

    // Validate each skill exists
    for stage in &stages {
        validate_skill_exists(&project_root, &stage.skill)?;
    }

    let pipelines = pipelines_dir(&project_root);
    fs::create_dir_all(&pipelines).into_diagnostic()?;

    let def_path = pipelines.join(format!("{}.yml", name));
    if def_path.exists() {
        miette::bail!("Pipeline '{}' already exists", name);
    }

    let definition = PipelineDefinition {
        name: name.to_string(),
        stages,
    };

    let yaml = serde_yml::to_string(&definition)
        .map_err(|e| miette::miette!("Failed to serialize pipeline: {}", e))?;
    fs::write(&def_path, yaml).into_diagnostic()?;

    log::success(format!("Created pipeline '{}'", name)).into_diagnostic()?;
    println!(
        "  {} {} stages defined",
        "•".dimmed(),
        definition.stages.len()
    );
    println!(
        "  {} Run with: wai pipeline run {} --topic=<slug>",
        "→".cyan(),
        name
    );
    Ok(())
}

// ─── run ─────────────────────────────────────────────────────────────────────

fn cmd_run(name: &str, topic: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline run")?;

    let def = load_pipeline_definition(&project_root, name)?;

    // Generate run ID: <name>-<YYYY-MM-DD>-<topic>
    let date = chrono::Utc::now().format("%Y-%m-%d");
    let topic_slug = slug::slugify(topic);
    let run_id = format!("{}-{}-{}", name, date, topic_slug);

    // Build initial run state
    let run = PipelineRun {
        run_id: run_id.clone(),
        pipeline: name.to_string(),
        topic: topic.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        current_stage: 0,
        stages: def
            .stages
            .iter()
            .map(|s| RunStage {
                skill: s.skill.clone(),
                artifact: s.artifact.clone(),
                completed: false,
                artifact_path: None,
            })
            .collect(),
    };

    // Persist run state
    let runs_dir = pipelines_dir(&project_root).join(name).join("runs");
    fs::create_dir_all(&runs_dir).into_diagnostic()?;

    let run_path = runs_dir.join(format!("{}.yml", run_id));
    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    // Output
    println!();
    println!("  {} Pipeline run started", "◆".cyan());
    println!();
    println!("  Run ID: {}", run_id.bold());
    println!();
    println!("  Set the environment variable so `wai add` auto-tags artifacts:");
    println!("    export WAI_PIPELINE_RUN={}", run_id);
    println!();
    print_stage_hint(&run, 0);
    Ok(())
}

// ─── advance ─────────────────────────────────────────────────────────────────

fn cmd_advance(run_id: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline advance")?;

    let (mut run, run_path) = find_run(&project_root, run_id)?;

    if run.current_stage >= run.stages.len() {
        miette::bail!(
            "Run '{}' has already completed all {} stages",
            run_id,
            run.stages.len()
        );
    }

    // Look for an artifact tagged with pipeline-run:<run-id>
    let artifact_path = find_latest_tagged_artifact(&project_root, run_id);

    run.stages[run.current_stage].completed = true;
    run.stages[run.current_stage].artifact_path = artifact_path.clone();
    run.current_stage += 1;

    // Persist
    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    let completed_idx = run.current_stage - 1;
    println!();
    println!(
        "  {} Stage {} complete: {}",
        "✓".green(),
        completed_idx + 1,
        run.stages[completed_idx].skill.bold()
    );
    if let Some(ref path) = artifact_path {
        println!("    artifact: {}", path.dimmed());
    }

    if run.current_stage >= run.stages.len() {
        println!();
        println!("  {} All stages complete!", "◆".green());
        println!(
            "  {} Run 'wai pipeline status {}' to review",
            "→".cyan(),
            run.pipeline
        );
    } else {
        println!();
        print_stage_hint(&run, run.current_stage);
    }

    Ok(())
}

// ─── status ──────────────────────────────────────────────────────────────────

fn cmd_status(name: &str, run_filter: Option<&str>) -> Result<()> {
    let project_root = require_project()?;

    // Verify pipeline exists
    let _ = load_pipeline_definition(&project_root, name)?;

    let runs_dir = pipelines_dir(&project_root).join(name).join("runs");

    if !runs_dir.exists() {
        println!();
        println!("  {} No runs found for pipeline '{}'", "○".dimmed(), name);
        println!(
            "  {} Start one with: wai pipeline run {} --topic=<slug>",
            "→".cyan(),
            name
        );
        println!();
        return Ok(());
    }

    let mut runs: Vec<PipelineRun> = Vec::new();
    for entry in fs::read_dir(&runs_dir).into_diagnostic()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(run) = serde_yml::from_str::<PipelineRun>(&content)
        {
            if let Some(filter) = run_filter
                && run.run_id != filter
            {
                continue;
            }
            runs.push(run);
        }
    }

    if runs.is_empty() {
        if let Some(filter) = run_filter {
            miette::bail!("Run '{}' not found in pipeline '{}'", filter, name);
        }
        println!();
        println!("  {} No runs found for pipeline '{}'", "○".dimmed(), name);
        println!();
        return Ok(());
    }

    // Sort by created_at ascending
    runs.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    println!();
    println!(
        "  {} Pipeline: {}  ({} run{})",
        "◆".cyan(),
        name.bold(),
        runs.len(),
        if runs.len() == 1 { "" } else { "s" }
    );

    for run in &runs {
        let done = run.current_stage >= run.stages.len();
        let status_icon = if done {
            "✓".green().to_string()
        } else {
            "●".yellow().to_string()
        };
        println!();
        println!(
            "  {} {} (topic: {})",
            status_icon,
            run.run_id.bold(),
            run.topic.dimmed()
        );
        for (i, stage) in run.stages.iter().enumerate() {
            let stage_icon = if stage.completed {
                "[x]".green().to_string()
            } else if i == run.current_stage {
                "[ ]".yellow().to_string()
            } else {
                "[ ]".dimmed().to_string()
            };
            print!("      {} stage {}: {}", stage_icon, i + 1, stage.skill);
            if let Some(ref path) = stage.artifact_path {
                print!("  → {}", path.dimmed());
            }
            println!();
        }
    }
    println!();

    Ok(())
}

// ─── list ────────────────────────────────────────────────────────────────────

fn cmd_list() -> Result<()> {
    let project_root = require_project()?;
    let pipelines = pipelines_dir(&project_root);

    if !pipelines.exists() {
        println!();
        println!("  {} No pipelines defined", "○".dimmed());
        println!(
            "  {} Create one with: wai pipeline create <name> --stages=\"skill:artifact,...\"",
            "→".cyan()
        );
        println!();
        return Ok(());
    }

    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(&pipelines).into_diagnostic()?.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("yml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            names.push(stem.to_string());
        }
    }

    if names.is_empty() {
        println!();
        println!("  {} No pipelines defined", "○".dimmed());
        println!(
            "  {} Create one with: wai pipeline create <name> --stages=\"skill:artifact,...\"",
            "→".cyan()
        );
        println!();
        return Ok(());
    }

    names.sort();

    println!();
    println!("  {} Pipelines", "◆".cyan());
    println!();
    for name in &names {
        // Load stage count
        let def = load_pipeline_definition(&project_root, name);
        let stage_info = match def {
            Ok(d) => format!("{} stages", d.stages.len()),
            Err(_) => "(invalid)".to_string(),
        };
        // Count runs
        let run_count = count_runs(&project_root, name);
        println!(
            "    {} {}  {}  ({} run{})",
            "•".dimmed(),
            name.bold(),
            stage_info.dimmed(),
            run_count,
            if run_count == 1 { "" } else { "s" }
        );
    }
    println!();

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Parse the stages string "skill:artifact,skill:artifact,..." into a Vec<PipelineStage>.
fn parse_stages(stages_str: &str) -> Result<Vec<PipelineStage>> {
    let mut stages = Vec::new();
    for (i, part) in stages_str.split(',').enumerate() {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let colon = part.find(':').ok_or_else(|| {
            miette::miette!(
                "Stage {} '{}' is missing ':' separator — expected format: skill:artifact-type",
                i + 1,
                part
            )
        })?;
        let skill = part[..colon].trim().to_string();
        let artifact = part[colon + 1..].trim().to_string();
        if skill.is_empty() {
            miette::bail!("Stage {} has an empty skill name", i + 1);
        }
        if artifact.is_empty() {
            miette::bail!("Stage {} '{}' has an empty artifact type", i + 1, skill);
        }
        stages.push(PipelineStage { skill, artifact });
    }
    Ok(stages)
}

/// Validate that a pipeline name is non-empty, lowercase, alphanumeric + hyphens.
fn validate_pipeline_name(name: &str) -> Result<()> {
    if name.is_empty() {
        miette::bail!("Pipeline name cannot be empty");
    }
    if name.len() > 64 {
        miette::bail!("Pipeline name too long ({} chars, max 64)", name.len());
    }
    for ch in name.chars() {
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '-' {
            miette::bail!(
                "Invalid character '{}' in pipeline name — only lowercase letters, digits, and hyphens allowed",
                ch
            );
        }
    }
    if name.starts_with('-') || name.ends_with('-') {
        miette::bail!("Pipeline name cannot start or end with a hyphen");
    }
    Ok(())
}

/// Validate that a skill exists in the local skills directory.
fn validate_skill_exists(project_root: &Path, skill: &str) -> Result<()> {
    let skills_dir = agent_config_dir(project_root).join(SKILLS_DIR);
    // skills_dir/<skill>/SKILL.md for flat, skills_dir/<category>/<action>/SKILL.md for hierarchical
    let skill_file = skills_dir.join(skill).join("SKILL.md");
    if !skill_file.exists() {
        miette::bail!(
            "Skill '{}' not found — run 'wai resource list skills' to see available skills",
            skill
        );
    }
    Ok(())
}

/// Load a pipeline definition YAML from `.wai/resources/pipelines/<name>.yml`.
fn load_pipeline_definition(project_root: &Path, name: &str) -> Result<PipelineDefinition> {
    let path = pipelines_dir(project_root).join(format!("{}.yml", name));
    if !path.exists() {
        miette::bail!(
            "Pipeline '{}' not found — run 'wai pipeline list' to see available pipelines",
            name
        );
    }
    let content = fs::read_to_string(&path).into_diagnostic()?;
    serde_yml::from_str(&content)
        .map_err(|e| miette::miette!("Failed to parse pipeline '{}': {}", name, e))
}

/// Search all pipeline run directories for a run with the given ID.
/// Returns the loaded run and its file path.
fn find_run(project_root: &Path, run_id: &str) -> Result<(PipelineRun, PathBuf)> {
    let pipelines = pipelines_dir(project_root);
    if !pipelines.exists() {
        miette::bail!("Run '{}' not found", run_id);
    }

    for entry in WalkDir::new(&pipelines)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && e.path().extension().and_then(|x| x.to_str()) == Some("yml")
                && e.path()
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    == Some("runs")
        })
    {
        if let Ok(content) = fs::read_to_string(entry.path())
            && let Ok(run) = serde_yml::from_str::<PipelineRun>(&content)
            && run.run_id == run_id
        {
            return Ok((run, entry.path().to_path_buf()));
        }
    }

    miette::bail!(
        "Run '{}' not found — use 'wai pipeline status <pipeline>' to see run IDs",
        run_id
    )
}

/// Search `.wai/` for the most recently modified `.md` file tagged with
/// `pipeline-run:<run_id>`. Returns a path relative to the project root.
fn find_latest_tagged_artifact(project_root: &Path, run_id: &str) -> Option<String> {
    let search_tag = format!("pipeline-run:{}", run_id);
    let wai = wai_dir(project_root);

    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;

    for entry in WalkDir::new(&wai)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|x| x.to_str()) == Some("md")
        })
    {
        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        if has_tag(&content, &search_tag)
            && let Ok(meta) = fs::metadata(entry.path())
            && let Ok(modified) = meta.modified()
        {
            let is_newer = best.as_ref().map(|(t, _)| modified > *t).unwrap_or(true);
            if is_newer {
                best = Some((modified, entry.path().to_path_buf()));
            }
        }
    }

    best.and_then(|(_, path)| {
        path.strip_prefix(project_root)
            .ok()
            .map(|p| p.display().to_string())
    })
}

/// Check whether a markdown file's frontmatter contains a specific tag (case-insensitive).
fn has_tag(content: &str, tag: &str) -> bool {
    let body = content.trim_start();
    if !body.starts_with("---") {
        return false;
    }
    let rest = &body[3..];
    let end = rest.find("\n---").unwrap_or(rest.len());
    let frontmatter = &rest[..end];

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("tags:") {
            let value = value.trim();
            if value.starts_with('[') {
                let inner = value.trim_start_matches('[').trim_end_matches(']');
                for t in inner.split(',') {
                    if t.trim().eq_ignore_ascii_case(tag) {
                        return true;
                    }
                }
            }
        } else if line.starts_with("- ") && line[2..].trim().eq_ignore_ascii_case(tag) {
            return true;
        }
    }
    false
}

/// Count the number of completed runs for a pipeline.
fn count_runs(project_root: &Path, name: &str) -> usize {
    let runs_dir = pipelines_dir(project_root).join(name).join("runs");
    if !runs_dir.exists() {
        return 0;
    }
    fs::read_dir(&runs_dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yml"))
                .count()
        })
        .unwrap_or(0)
}

/// Print a hint for invoking the given stage index.
fn print_stage_hint(run: &PipelineRun, stage_idx: usize) {
    if stage_idx >= run.stages.len() {
        return;
    }
    let stage = &run.stages[stage_idx];
    println!(
        "  {} Stage {}: {} (artifact: {})",
        "→".cyan(),
        stage_idx + 1,
        stage.skill.bold(),
        stage.artifact.dimmed()
    );
    println!("    Set env:   export WAI_PIPELINE_RUN={}", run.run_id);
    println!("    When done: wai pipeline advance {}", run.run_id);
    println!();
}
