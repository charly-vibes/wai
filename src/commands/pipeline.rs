use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::cli::PipelineCommands;
use crate::config::pipelines_dir;
use crate::context::require_safe_mode;

use super::require_project;

// ─── Data structures ─────────────────────────────────────────────────────────

// ── New TOML-based pipeline model ────────────────────────────────────────────

/// One step in a TOML pipeline definition.
///
/// Stored as `[[steps]]` entries in a `.wai/resources/pipelines/<name>.toml` file.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PipelineStep {
    pub id: String,
    pub prompt: String,
}

/// A pipeline definition deserialized from a TOML file.
///
/// The TOML format uses a `[pipeline]` table and `[[steps]]` arrays:
/// ```toml
/// [pipeline]
/// name = "feature"
/// description = "Full feature workflow"
///
/// [[steps]]
/// id = "research"
/// prompt = "Research {topic}: gather background, constraints, prior art."
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PipelineDefinition {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<PipelineStep>,
}

/// Top-level TOML file wrapper.
///
/// The TOML format uses a `[pipeline]` section for metadata and top-level
/// `[[steps]]` arrays. This wrapper captures both and merges them into a
/// [`PipelineDefinition`].
#[derive(serde::Deserialize)]
struct PipelineDefinitionFile {
    pipeline: PipelineMetadata,
    #[serde(default)]
    steps: Vec<PipelineStep>,
}

/// Pipeline metadata from the `[pipeline]` TOML section.
#[derive(serde::Deserialize)]
struct PipelineMetadata {
    name: String,
    description: Option<String>,
}

/// Run state stored at `.wai/pipeline-runs/<run-id>.yml`.
#[derive(Debug, serde::Serialize, Deserialize)]
pub struct PipelineRun {
    pub run_id: String,
    pub pipeline: String,
    pub topic: String,
    pub created_at: String,
    /// Index of the current (not-yet-completed) step; equals total step count when done.
    pub current_step: usize,
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn run(cmd: PipelineCommands) -> Result<()> {
    match cmd {
        PipelineCommands::Status { name, run } => cmd_status(&name, run.as_deref()),
        PipelineCommands::List => cmd_list(),
        PipelineCommands::Init { name } => cmd_init(&name),
        PipelineCommands::Start { name, topic } => cmd_start(&name, topic.as_deref()),
        PipelineCommands::Next => cmd_next(),
        PipelineCommands::Current => cmd_current(),
        PipelineCommands::Suggest { description } => cmd_suggest(description.as_deref()),
    }
}

// ─── status ──────────────────────────────────────────────────────────────────

fn cmd_status(_name: &str, _run_filter: Option<&str>) -> Result<()> {
    miette::bail!(
        "'wai pipeline status' is deprecated. Use 'wai pipeline current' to see the active step."
    )
}

// ─── list ────────────────────────────────────────────────────────────────────

fn cmd_list() -> Result<()> {
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

// ─── init ─────────────────────────────────────────────────────────────────────

fn cmd_init(name: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline init")?;

    validate_pipeline_name(name)?;

    let pipelines = pipelines_dir(&project_root);
    fs::create_dir_all(&pipelines).into_diagnostic()?;

    let file_path = pipelines.join(format!("{}.toml", name));
    if file_path.exists() {
        miette::bail!(
            "Pipeline '{}' already exists: {}",
            name,
            file_path.display()
        );
    }

    // The template uses {topic} as the variable substitution placeholder.
    // We build this as a plain string (no format!) to avoid escaping collisions.
    let template = concat!(
        "# Step prompts are navigation hints. Instructions for HOW to do the\n",
        "# work belong in skills.\n",
        "[pipeline]\n",
        "name = \"PIPELINE_NAME\"\n",
        "description = \"Describe what this pipeline does\"\n",
        "\n",
        "[[steps]]\n",
        "id = \"step-one\"\n",
        "prompt = \"\"\"\n",
        "{topic}: TODO describe step one task.\n",
        "Use skill `<skill-name>` if available.\n",
        "Record findings: `wai add research \"...\"`\n",
        "Advance: `wai pipeline next`\n",
        "\"\"\"\n",
        "\n",
        "[[steps]]\n",
        "id = \"step-two\"\n",
        "prompt = \"\"\"\n",
        "{topic}: TODO describe step two task.\n",
        "Use skill `<skill-name>` if available.\n",
        "Record decisions: `wai add design \"...\"`\n",
        "Advance: `wai pipeline next`\n",
        "\"\"\"\n",
    );
    let template = template.replace("PIPELINE_NAME", name);

    fs::write(&file_path, template).into_diagnostic()?;

    log::success(format!("Created pipeline: {}", file_path.display())).into_diagnostic()?;
    println!(
        "  {} Edit the prompts, then start with: wai pipeline start {} --topic=<your-topic>",
        "→".cyan(),
        name
    );
    Ok(())
}

// ─── start ────────────────────────────────────────────────────────────────────

fn cmd_start(name: &str, topic: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline start")?;

    // 1. Find and load the pipeline TOML definition
    let def_path = pipelines_dir(&project_root).join(format!("{}.toml", name));
    if !def_path.exists() {
        miette::bail!(
            "Pipeline '{}' not found. Create it with: wai pipeline init {}",
            name,
            name
        );
    }
    let definition = load_pipeline_toml(&def_path)?;

    if definition.steps.is_empty() {
        miette::bail!("Pipeline '{}' has no steps defined", name);
    }

    // 2. Generate a unique run ID: <name>-<YYYY-MM-DD>-<topic-slug>
    let date = chrono::Utc::now().format("%Y-%m-%d");
    let topic_str = topic.unwrap_or("");
    let topic_slug = if topic_str.is_empty() {
        "run".to_string()
    } else {
        slug::slugify(topic_str)
    };
    let run_id = format!("{}-{}-{}", name, date, topic_slug);

    // 3. Create run state
    let run = PipelineRun {
        run_id: run_id.clone(),
        pipeline: name.to_string(),
        topic: topic_str.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        current_step: 0,
    };

    // 4. Write run state to .wai/pipeline-runs/<run-id>.yml
    let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
    fs::create_dir_all(&runs_dir).into_diagnostic()?;
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    // 5. Write .last-run pointer file
    let last_run = crate::config::last_run_path(&project_root);
    fs::create_dir_all(last_run.parent().unwrap()).into_diagnostic()?;
    fs::write(&last_run, &run_id).into_diagnostic()?;

    // 6. Write .wai/.pipeline-run so `wai add` picks up the run ID automatically
    crate::config::write_pipeline_run_state(&project_root, &run_id)
        .map_err(|e| miette::miette!("Failed to write pipeline run state: {}", e))?;

    // 7. Print env export line + first step prompt block
    println!("export WAI_PIPELINE_RUN={}", run_id);
    println!();
    print_step(&definition, 0, topic_str);

    Ok(())
}

// ─── next ─────────────────────────────────────────────────────────────────────

fn cmd_next() -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline next")?;

    // 1. Resolve run ID (env var → .last-run)
    let run_id = resolve_active_run_id(&project_root)?;

    // 2. Load run state
    let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    if !run_path.exists() {
        miette::bail!(
            "Run state file not found for run '{}'. The run may have been deleted or the ID is stale.",
            run_id
        );
    }
    let run: PipelineRun =
        serde_yml::from_str(&fs::read_to_string(&run_path).into_diagnostic()?)
            .map_err(|e| miette::miette!("Failed to parse run state for '{}': {}", run_id, e))?;

    // 3. Load pipeline definition
    let def_path =
        crate::config::pipelines_dir(&project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    // 4. Check not already complete
    if run.current_step >= definition.steps.len() {
        miette::bail!(
            "Pipeline run '{}' is already complete. Start a new run with: wai pipeline start {} --topic=<topic>",
            run_id,
            run.pipeline
        );
    }

    // 5. Advance step
    let next_step = run.current_step + 1;
    let updated = PipelineRun {
        current_step: next_step,
        ..run
    };
    let yaml = serde_yml::to_string(&updated)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    // 6. Print next step or completion block
    if next_step >= definition.steps.len() {
        println!("──────────────────────────────────────────────");
        println!("Pipeline '{}' complete!", definition.name);
        println!();
        println!("Next: wai close");
        println!("      wai pipeline suggest   # start another pipeline");
    } else {
        print_step(&definition, next_step, &updated.topic);
    }

    Ok(())
}

// ─── current ──────────────────────────────────────────────────────────────────

fn cmd_current() -> Result<()> {
    let project_root = require_project()?;

    // 1. Resolve run ID (env var → .last-run)
    let run_id = resolve_active_run_id(&project_root)?;

    // 2. Load run state
    let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    if !run_path.exists() {
        miette::bail!(
            "No run state found for '{}'. The .last-run pointer may be stale.\nStart a new run with: wai pipeline start <name> --topic=<topic>",
            run_id
        );
    }
    let run: PipelineRun =
        serde_yml::from_str(&fs::read_to_string(&run_path).into_diagnostic()?)
            .map_err(|e| miette::miette!("Failed to parse run state for '{}': {}", run_id, e))?;

    // 3. Load pipeline definition
    let def_path =
        crate::config::pipelines_dir(&project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    // 4. Print current step (or completion block if the run is done)
    if run.current_step >= definition.steps.len() {
        println!("──────────────────────────────────────────────");
        println!("Pipeline '{}' is already complete!", definition.name);
        println!();
        println!("Next: wai close");
    } else {
        print_step(&definition, run.current_step, &run.topic);
    }

    Ok(())
}

// ─── suggest ──────────────────────────────────────────────────────────────────

fn cmd_suggest(description: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    let pipelines = pipelines_dir(&project_root);

    // Collect all valid pipeline TOMLs
    let mut found: Vec<(String, PipelineDefinition)> = vec![];
    if pipelines.exists() {
        for entry in fs::read_dir(&pipelines).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    match load_pipeline_toml(&path) {
                        Ok(def) => found.push((name.to_string(), def)),
                        Err(e) => eprintln!("warning: skipping {}: {}", path.display(), e),
                    }
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

/// Resolve the active run ID: check `WAI_PIPELINE_RUN` env var first, then
/// fall back to the `.last-run` pointer file at `.wai/resources/pipelines/.last-run`.
fn resolve_active_run_id(project_root: &Path) -> Result<String> {
    // Try env var first
    if let Ok(run_id) = std::env::var("WAI_PIPELINE_RUN") {
        if !run_id.is_empty() {
            return Ok(run_id);
        }
    }
    // Fall back to .last-run pointer file
    let last_run = crate::config::last_run_path(project_root);
    if last_run.exists() {
        let run_id = fs::read_to_string(&last_run)
            .into_diagnostic()?
            .trim()
            .to_string();
        if !run_id.is_empty() {
            return Ok(run_id);
        }
    }
    miette::bail!(
        "No active pipeline run. Start one with: wai pipeline start <name> --topic=<topic>"
    )
}

/// Print a step prompt block with a "step N/M" header and rendered prompt.
fn print_step(definition: &PipelineDefinition, idx: usize, topic: &str) {
    let step = &definition.steps[idx];
    let total = definition.steps.len();
    println!(
        "── step {}/{}: {} ──────────────────────────────",
        idx + 1,
        total,
        step.id
    );
    println!("{}", render_prompt(&step.prompt, topic));
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

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

/// Load a TOML pipeline definition from `.wai/resources/pipelines/<name>.toml`.
///
/// Validates that all step IDs are unique and all prompts are non-empty.
pub fn load_pipeline_toml(path: &Path) -> Result<PipelineDefinition> {
    let content = fs::read_to_string(path).into_diagnostic()?;
    let file: PipelineDefinitionFile = toml::from_str(&content)
        .map_err(|e| miette::miette!("Failed to parse pipeline TOML: {}", e))?;
    let def = PipelineDefinition {
        name: file.pipeline.name,
        description: file.pipeline.description,
        steps: file.steps,
    };

    // Validate unique IDs
    let mut seen_ids = HashSet::new();
    for step in &def.steps {
        if !seen_ids.insert(step.id.as_str()) {
            miette::bail!("duplicate step id: {}", step.id);
        }
    }

    // Validate non-empty prompts
    for step in &def.steps {
        if step.prompt.trim().is_empty() {
            miette::bail!("empty prompt for step: {}", step.id);
        }
    }

    Ok(def)
}

/// Substitute `{topic}` in a prompt string with the given topic value.
///
/// If the prompt contains no `{topic}` placeholder, the prompt is returned unchanged.
pub fn render_prompt(prompt: &str, topic: &str) -> String {
    prompt.replace("{topic}", topic)
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ── render_prompt ──────────────────────────────────────────────────────

    #[test]
    fn render_prompt_substitutes_topic() {
        assert_eq!(render_prompt("Hello {topic}!", "world"), "Hello world!");
    }

    #[test]
    fn render_prompt_no_placeholder_no_panic() {
        assert_eq!(render_prompt("Hello!", "world"), "Hello!");
    }

    #[test]
    fn render_prompt_multiple_occurrences() {
        assert_eq!(
            render_prompt("Research {topic}. Focus on {topic} constraints.", "auth"),
            "Research auth. Focus on auth constraints."
        );
    }

    // ── load_pipeline_toml ─────────────────────────────────────────────────

    fn write_toml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("create tempfile");
        f.write_all(content.as_bytes()).expect("write toml");
        f
    }

    #[test]
    fn load_pipeline_toml_valid() {
        let toml = r#"
[pipeline]
name = "feature"
description = "A feature workflow"

[[steps]]
id = "research"
prompt = "Research {topic}: gather background."

[[steps]]
id = "implement"
prompt = "Implement {topic}."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse valid TOML");
        assert_eq!(def.name, "feature");
        assert_eq!(def.description.as_deref(), Some("A feature workflow"));
        assert_eq!(def.steps.len(), 2);
        assert_eq!(def.steps[0].id, "research");
        assert_eq!(def.steps[1].id, "implement");
    }

    #[test]
    fn load_pipeline_toml_no_description() {
        let toml = r#"
[pipeline]
name = "minimal"

[[steps]]
id = "go"
prompt = "Do {topic}."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse minimal TOML");
        assert_eq!(def.description, None);
        assert_eq!(def.steps.len(), 1);
    }

    #[test]
    fn load_pipeline_toml_rejects_duplicate_ids() {
        let toml = r#"
[pipeline]
name = "broken"

[[steps]]
id = "research"
prompt = "Research {topic}."

[[steps]]
id = "research"
prompt = "More research on {topic}."
"#;
        let f = write_toml(toml);
        let result = load_pipeline_toml(f.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("duplicate step id: research"),
            "expected 'duplicate step id: research' in error, got: {msg}"
        );
    }

    #[test]
    fn load_pipeline_toml_rejects_empty_prompt() {
        let toml = r#"
[pipeline]
name = "broken"

[[steps]]
id = "research"
prompt = ""
"#;
        let f = write_toml(toml);
        let result = load_pipeline_toml(f.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("empty prompt for step: research"),
            "expected 'empty prompt for step: research' in error, got: {msg}"
        );
    }

    #[test]
    fn load_pipeline_toml_rejects_whitespace_only_prompt() {
        let toml = "[pipeline]\nname = \"broken\"\n\n[[steps]]\nid = \"step1\"\nprompt = \"   \"\n";
        let f = write_toml(toml);
        let result = load_pipeline_toml(f.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("empty prompt for step: step1"),
            "expected 'empty prompt for step: step1' in error, got: {msg}"
        );
    }
}
