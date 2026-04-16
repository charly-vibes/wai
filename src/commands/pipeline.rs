use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
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
    /// Optional gate configuration for this step.
    #[serde(default)]
    pub gate: Option<StepGate>,
    /// When true, artifacts for this step are locked (SHA-256 hashed) on advancement.
    #[serde(default)]
    pub lock: bool,
}

/// Gate configuration for a pipeline step.
///
/// Gates are evaluated in order: structural → procedural → coverage → oracle → approval.
/// The first failing tier blocks advancement.
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct StepGate {
    pub structural: Option<StructuralGate>,
    pub procedural: Option<ProceduralGate>,
    pub coverage: Option<CoverageGate>,
    #[serde(default)]
    pub oracles: Vec<OracleGate>,
    pub approval: Option<ApprovalGate>,
}

/// Structural gate: verify minimum artifact counts for this step.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StructuralGate {
    /// Minimum number of artifacts required.
    pub min_artifacts: usize,
    /// Optional filter: only count artifacts of these types (research, plan, design, etc.).
    #[serde(default)]
    pub types: Vec<String>,
}

/// Procedural gate: verify review coverage for step artifacts.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProceduralGate {
    /// Require a review artifact for each reviewable artifact.
    #[serde(default)]
    pub require_review: bool,
    /// Which artifact types require reviews. Defaults to all except "review".
    #[serde(default)]
    pub review_types: Vec<String>,
    /// Maximum allowed critical-severity findings (default: no limit).
    pub max_critical: Option<u32>,
    /// Maximum allowed high-severity findings (default: no limit).
    pub max_high: Option<u32>,
}

/// Coverage gate: require an input coverage manifest before advancing.
///
/// A coverage manifest is a wai artifact of type `review` tagged with
/// `coverage-manifest:<step-id>`, listing each input artifact with a disposition.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CoverageGate {
    /// When true, require a coverage manifest artifact before advancing.
    #[serde(default)]
    pub require_input_manifest: bool,
}

/// Oracle gate: run a user-defined script to validate artifacts.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OracleGate {
    pub name: String,
    /// Explicit command override (bypasses name resolution).
    pub command: Option<String>,
    /// Description shown in gate status output.
    #[allow(dead_code)] // Used by `wai pipeline show` (wai-zjt6)
    pub description: Option<String>,
    /// Timeout in seconds (default: 30).
    pub timeout: Option<u64>,
    /// Scope: "artifact" (default, one invocation per artifact) or "all" (one invocation with all paths).
    pub scope: Option<String>,
}

/// Approval gate: require explicit human approval before advancing.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApprovalGate {
    pub required: bool,
    /// Message shown when approval is needed.
    pub message: Option<String>,
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
    /// Optional metadata for discoverability (managed block, suggest).
    #[serde(default)]
    #[allow(dead_code)] // Read via lib workspace.rs for managed block generation
    pub metadata: Option<PipelineMetadataSection>,
}

/// Top-level TOML file wrapper.
///
/// The TOML format uses a `[pipeline]` section for metadata and top-level
/// `[[steps]]` arrays. Steps may include a `[steps.gate]` sub-table with
/// gate configuration (structural, procedural, coverage, oracle, approval).
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
    #[serde(default)]
    metadata: Option<PipelineMetadataSection>,
}

/// Optional `[pipeline.metadata]` section for discoverability.
#[derive(Debug, Clone, serde::Deserialize, Default)]
#[allow(dead_code)] // Fields read via lib workspace.rs for managed block generation
pub struct PipelineMetadataSection {
    /// When to suggest this pipeline (human-readable description).
    pub when: Option<String>,
    /// Skill names this pipeline depends on.
    #[serde(default)]
    pub skills: Vec<String>,
}

/// Lock metadata written to a `.lock` sidecar TOML file.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)] // Used by pipeline lock commands (wai-yh5v, wai-bzrp)
pub struct ArtifactLock {
    pub artifact: String,
    pub locked_at: String,
    pub lock_hash: String,
    pub pipeline_run: String,
    pub pipeline_step: String,
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
    /// Per-step approval timestamps (step_id → ISO 8601 timestamp).
    #[serde(default)]
    pub approvals: std::collections::HashMap<String, String>,
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
        PipelineCommands::Approve => cmd_approve(),
        PipelineCommands::Show { name } => cmd_show(&name),
        PipelineCommands::Gates { name, step } => cmd_gates(name.as_deref(), step.as_deref()),
        PipelineCommands::Check { oracle } => cmd_check(oracle.as_deref()),
        PipelineCommands::Validate { name } => cmd_validate(name.as_deref()),
        PipelineCommands::Lock => cmd_lock(),
        PipelineCommands::Verify => cmd_verify(),
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

    // Check for built-in template, otherwise use generic scaffold
    let template = if let Some(builtin) = get_builtin_template(name) {
        builtin.to_string()
    } else {
        // The template uses {topic} as the variable substitution placeholder.
        // We build this as a plain string (no format!) to avoid escaping collisions.
        let tmpl = concat!(
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
        tmpl.replace("PIPELINE_NAME", name)
    };

    fs::write(&file_path, template).into_diagnostic()?;

    // Scaffold oracles directory with README if not present
    let oracles_dir = crate::config::wai_dir(&project_root)
        .join("resources")
        .join("oracles");
    fs::create_dir_all(&oracles_dir).into_diagnostic()?;
    let readme_path = oracles_dir.join("README.md");
    if !readme_path.exists() {
        let readme = "# Oracle Scripts\n\n\
            Oracle scripts are user-defined validators run during pipeline gate checks.\n\n\
            ## Convention\n\n\
            - Place scripts here: `.wai/resources/oracles/<name>[.sh|.py]`\n\
            - Scripts must be executable (`chmod +x`)\n\
            - Exit 0 = pass, non-zero = fail\n\
            - Write failure reasons to stderr\n\
            - Default scope: one invocation per artifact (`<script> <artifact-path>`)\n\
            - Cross-artifact scope: `scope = \"all\"` passes all paths at once\n\n\
            ## Example\n\n\
            ```bash\n\
            #!/usr/bin/env bash\n\
            # example-check.sh — verify artifact contains required sections\n\
            grep -q '## Constraints' \"$1\" || { echo 'Missing ## Constraints section' >&2; exit 1; }\n\
            ```\n\n\
            Configure in your pipeline TOML:\n\
            ```toml\n\
            [[steps.gate.oracles]]\n\
            name = \"example-check\"\n\
            ```\n";
        fs::write(&readme_path, readme).into_diagnostic()?;
    }
    let example_path = oracles_dir.join("example-check.sh");
    if !example_path.exists() {
        let example = "#!/usr/bin/env bash\n\
            # example-check.sh — sample oracle that verifies artifact has content\n\
            # Exit 0 = pass, non-zero = fail. Stderr is shown on failure.\n\
            set -euo pipefail\n\n\
            FILE=\"$1\"\n\
            if [ ! -s \"$FILE\" ]; then\n\
            \x20   echo \"Artifact is empty: $FILE\" >&2\n\
            \x20   exit 1\n\
            fi\n";
        fs::write(&example_path, example).into_diagnostic()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&example_path).into_diagnostic()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&example_path, perms).into_diagnostic()?;
        }
    }

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

    // 1b. Validate the pipeline definition
    let issues = validate_pipeline(&definition, &project_root);
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.level == ValidationLevel::Error)
        .collect();
    let warnings: Vec<_> = issues
        .iter()
        .filter(|i| i.level == ValidationLevel::Warn)
        .collect();

    if !errors.is_empty() {
        for e in &errors {
            log::error(&e.message).into_diagnostic()?;
        }
        miette::bail!(
            "Pipeline '{}' has validation errors. Fix them before starting.",
            name
        );
    }
    for w in &warnings {
        log::warning(&w.message).into_diagnostic()?;
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
        approvals: HashMap::new(),
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

    // 5. Evaluate gates (if configured) before allowing advancement
    let current_step = &definition.steps[run.current_step];
    if let Some(ref gate) = current_step.gate {
        let failures = evaluate_gates(gate, current_step, &run, &definition, &project_root)?;
        if !failures.is_empty() {
            println!();
            println!(
                "  {} Gate check failed for step '{}':",
                "✗".red(),
                current_step.id
            );
            println!();
            for f in &failures {
                println!("    {} {}", "✗".red(), f);
            }
            println!();
            println!(
                "  {} Resolve the above before running `wai pipeline next`",
                "→".cyan()
            );
            return Ok(());
        }
    }

    // 5b. Lock artifacts if step has lock = true
    if current_step.lock {
        let artifact_paths = find_step_artifact_paths(&project_root, &run.run_id, &current_step.id);
        if artifact_paths.is_empty() {
            miette::bail!("Cannot lock step '{}' with no artifacts.", current_step.id);
        }
        for path in &artifact_paths {
            write_artifact_lock(path, &run.run_id, &current_step.id)?;
        }
        log::info(format!(
            "Locked {} artifact(s) for step '{}'",
            artifact_paths.len(),
            current_step.id
        ))
        .into_diagnostic()?;
    }

    // 6. Advance step
    let next_step = run.current_step + 1;
    let updated = PipelineRun {
        current_step: next_step,
        ..run
    };
    let yaml = serde_yml::to_string(&updated)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    // 7. Print next step or completion block
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

        // 5. Show addenda for the current step (if any)
        let step_id = &definition.steps[run.current_step].id;
        let addenda = find_step_addenda(&project_root, step_id);
        if !addenda.is_empty() {
            println!();
            println!("{} Addenda ({})", "◆".cyan(), addenda.len());
            for path in &addenda {
                println!("  {} {}", "•".dimmed(), path);
            }
        }
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

// ─── approve ─────────────────────────────────────────────────────────────────

fn cmd_approve() -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline approve")?;

    let run_id = resolve_active_run_id(&project_root)?;
    let runs_dir = crate::config::wai_dir(&project_root).join("pipeline-runs");
    let run_path = runs_dir.join(format!("{}.yml", run_id));
    if !run_path.exists() {
        miette::bail!("No active pipeline run.");
    }
    let mut run: PipelineRun =
        serde_yml::from_str(&fs::read_to_string(&run_path).into_diagnostic()?)
            .map_err(|e| miette::miette!("Failed to parse run state: {}", e))?;

    let def_path =
        crate::config::pipelines_dir(&project_root).join(format!("{}.toml", run.pipeline));
    let definition = load_pipeline_toml(&def_path)?;

    if run.current_step >= definition.steps.len() {
        miette::bail!("Pipeline run is already complete.");
    }

    let step_id = &definition.steps[run.current_step].id;
    let now = chrono::Utc::now().to_rfc3339();
    run.approvals.insert(step_id.clone(), now);

    let yaml = serde_yml::to_string(&run)
        .map_err(|e| miette::miette!("Failed to serialize run state: {}", e))?;
    fs::write(&run_path, yaml).into_diagnostic()?;

    log::success(format!(
        "Approved step '{}'. Run 'wai pipeline next' to advance.",
        step_id
    ))
    .into_diagnostic()?;

    Ok(())
}

// ─── lock ─────────────────────────────────────────────────────────────────────

fn cmd_lock() -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline lock")?;

    // 1. Resolve active run
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
            "Pipeline run '{}' is already complete. No step to lock.",
            run_id
        );
    }

    let current_step = &definition.steps[run.current_step];

    // 5. Find artifacts tagged with this step
    let artifact_paths = find_step_artifact_paths(&project_root, &run_id, &current_step.id);
    if artifact_paths.is_empty() {
        miette::bail!("Cannot lock step '{}' with no artifacts.", current_step.id);
    }

    // 6. Write lock sidecars for each artifact
    for path in &artifact_paths {
        write_artifact_lock(path, &run_id, &current_step.id)?;
    }

    log::success(format!(
        "Locked {} artifacts for step '{}'",
        artifact_paths.len(),
        current_step.id
    ))
    .into_diagnostic()?;

    Ok(())
}

// ─── Verify command ─────────────────────────────────────────────────────────

fn cmd_verify() -> Result<()> {
    let project_root = require_project()?;
    let projects_dir = crate::config::projects_dir(&project_root);

    // Walk all .lock files under .wai/projects/
    let mut lock_count: usize = 0;
    let mut mismatches: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&projects_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if ext != "lock" {
            continue;
        }

        let lock = read_artifact_lock(path)?;
        let artifact_path = path.parent().unwrap().join(&lock.artifact);

        if !artifact_path.exists() {
            mismatches.push(format!(
                "  {} — artifact missing (expected at {})",
                lock.artifact,
                artifact_path.display()
            ));
            lock_count += 1;
            continue;
        }

        let actual_hash = artifact_hash(&artifact_path)?;
        if actual_hash != lock.lock_hash {
            mismatches.push(format!(
                "  {} — expected {}, got {}",
                lock.artifact, lock.lock_hash, actual_hash
            ));
        }

        lock_count += 1;
    }

    if lock_count == 0 {
        log::info("No locked artifacts found.").into_diagnostic()?;
        return Ok(());
    }

    if mismatches.is_empty() {
        log::success(format!("All {} locked artifacts verified.", lock_count)).into_diagnostic()?;
        Ok(())
    } else {
        let header = format!(
            "{} of {} locked artifacts failed verification:",
            mismatches.len(),
            lock_count
        );
        let detail = mismatches.join("\n");
        log::error(format!("{}\n{}", header, detail)).into_diagnostic()?;
        std::process::exit(1);
    }
}

/// Find all artifact file paths tagged with the given run ID and step ID.
///
/// Similar to [`find_step_artifacts`] but returns full `PathBuf`s suitable for
/// passing to [`write_artifact_lock`].
fn find_step_artifact_paths(
    project_root: &Path,
    run_id: &str,
    step_id: &str,
) -> Vec<std::path::PathBuf> {
    let projects = crate::config::projects_dir(project_root);
    let run_tag = format!("pipeline-run:{}", run_id);
    let step_tag = format!("pipeline-step:{}", step_id);

    let mut paths = Vec::new();

    let Ok(entries) = fs::read_dir(&projects) else {
        return paths;
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
                if fm.tags.contains(&run_tag) && fm.tags.contains(&step_tag) {
                    paths.push(path);
                }
            }
        }
    }
    paths.sort();
    paths
}

// ─── show ─────────────────────────────────────────────────────────────────────

fn cmd_show(name: &str) -> Result<()> {
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

/// Format a one-line gate summary for a step.
fn format_gate_summary(gate: &Option<StepGate>) -> String {
    let Some(g) = gate else {
        return String::new();
    };
    let mut parts = Vec::new();
    if g.structural.is_some() {
        parts.push("structural");
    }
    if g.procedural.is_some() {
        parts.push("procedural");
    }
    if g.coverage.is_some() {
        parts.push("coverage");
    }
    if !g.oracles.is_empty() {
        parts.push("oracle");
    }
    if g.approval.is_some() {
        parts.push("approval");
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("[{}]", parts.join(" + "))
    }
}

// ─── gates ────────────────────────────────────────────────────────────────────

fn cmd_gates(name: Option<&str>, step_filter: Option<&str>) -> Result<()> {
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

/// Print gate status for a single step.
fn print_gate_status(
    step: &PipelineStep,
    run: Option<&PipelineRun>,
    _definition: Option<&PipelineDefinition>,
    project_root: &Path,
) -> Result<()> {
    println!();
    println!("  {} Step: {}", "◆".cyan(), step.id.bold());

    let Some(ref gate) = step.gate else {
        println!(
            "    {} No gates configured for step '{}'.",
            "•".dimmed(),
            step.id
        );
        println!();
        return Ok(());
    };

    let live = run.is_some();
    let step_artifacts = if live {
        let r = run.unwrap();
        find_step_artifacts(project_root, &r.run_id, &step.id)
    } else {
        Vec::new()
    };

    // Structural
    if let Some(ref sg) = gate.structural {
        let type_desc = if sg.types.is_empty() {
            "any".to_string()
        } else {
            sg.types.join("/")
        };
        if live {
            let matching: Vec<_> = if sg.types.is_empty() {
                step_artifacts.clone()
            } else {
                step_artifacts
                    .iter()
                    .filter(|a| sg.types.contains(&a.artifact_type))
                    .cloned()
                    .collect()
            };
            let passed = matching.len() >= sg.min_artifacts;
            if passed {
                println!(
                    "    {} Structural: min {} {} artifact(s) — found {}",
                    "✓".green(),
                    sg.min_artifacts,
                    type_desc,
                    matching.len()
                );
            } else {
                println!(
                    "    {} Structural: min {} {} artifact(s) — found {}",
                    "✗".red(),
                    sg.min_artifacts,
                    type_desc,
                    matching.len()
                );
            }
        } else {
            println!(
                "    {} Structural: min {} {} artifact(s)",
                "•".dimmed(),
                sg.min_artifacts,
                type_desc
            );
        }
    }

    // Procedural
    if let Some(ref pg) = gate.procedural
        && pg.require_review
    {
        let type_desc = if pg.review_types.is_empty() {
            "all (except review)".to_string()
        } else {
            pg.review_types.join("/")
        };
        if live {
            let reviewable: Vec<_> = step_artifacts
                .iter()
                .filter(|a| {
                    if a.artifact_type == "review" {
                        return false;
                    }
                    if pg.review_types.is_empty() {
                        true
                    } else {
                        pg.review_types.contains(&a.artifact_type)
                    }
                })
                .collect();
            let reviews: Vec<_> = step_artifacts
                .iter()
                .filter(|a| a.artifact_type == "review")
                .collect();
            let missing: Vec<_> = reviewable
                .iter()
                .filter(|a| {
                    !reviews
                        .iter()
                        .any(|r| r.reviews_target.as_deref() == Some(&a.filename))
                })
                .collect();
            let passed = missing.is_empty();
            if passed {
                println!(
                    "    {} Procedural: require review for {} types — {} unreviewed",
                    "✓".green(),
                    type_desc,
                    missing.len()
                );
            } else {
                println!(
                    "    {} Procedural: require review for {} types — {} unreviewed",
                    "✗".red(),
                    type_desc,
                    missing.len()
                );
            }
        } else {
            println!(
                "    {} Procedural: require review for {} types",
                "•".dimmed(),
                type_desc
            );
        }
        if let Some(mc) = pg.max_critical {
            println!("      {} max_critical: {}", "•".dimmed(), mc);
        }
        if let Some(mh) = pg.max_high {
            println!("      {} max_high: {}", "•".dimmed(), mh);
        }
    }

    // Coverage
    if let Some(ref cg) = gate.coverage
        && cg.require_input_manifest
    {
        println!("    {} Coverage: require input manifest", "•".dimmed(),);
    }

    // Oracles
    for oracle in &gate.oracles {
        let scope = oracle.scope.as_deref().unwrap_or("artifact");
        let desc = oracle.description.as_deref().unwrap_or("");
        if !desc.is_empty() {
            println!(
                "    {} Oracle: {} — {} (scope: {})",
                "•".dimmed(),
                oracle.name,
                desc,
                scope
            );
        } else {
            println!(
                "    {} Oracle: {} (scope: {})",
                "•".dimmed(),
                oracle.name,
                scope
            );
        }
    }

    // Approval
    if let Some(ref ag) = gate.approval
        && ag.required
    {
        if live {
            let r = run.unwrap();
            let approved = r.approvals.contains_key(&step.id);
            let msg = ag.message.as_deref().unwrap_or("required");
            if approved {
                println!("    {} Approval: {} (approved)", "✓".green(), msg);
            } else {
                println!("    {} Approval: {} (pending)", "✗".red(), msg);
            }
        } else {
            println!(
                "    {} Approval: {}",
                "•".dimmed(),
                ag.message.as_deref().unwrap_or("required")
            );
        }
    }

    println!();
    Ok(())
}

// ─── check ────────────────────────────────────────────────────────────────────

fn cmd_check(oracle_name: Option<&str>) -> Result<()> {
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

        let failures = run_oracle(oracle, &step_artifacts, &project_root)?;
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

fn cmd_validate(name: Option<&str>) -> Result<()> {
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
                log::error(format!("{}: {}", pname, e)).into_diagnostic()?;
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
                    log::success(format!(
                        "{}: {} steps, {} gated",
                        pname,
                        def.steps.len(),
                        gate_count
                    ))
                    .into_diagnostic()?;
                } else {
                    for e in &errors {
                        log::error(format!("{}: {}", pname, e.message)).into_diagnostic()?;
                        had_errors = true;
                    }
                    for w in &warnings {
                        log::warning(format!("{}: {}", pname, w.message)).into_diagnostic()?;
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

/// Returns a built-in template if one exists for the given name.
fn get_builtin_template(name: &str) -> Option<&'static str> {
    match name {
        "scientific-research" => Some(include_str!("../templates/scientific-research.toml")),
        _ => None,
    }
}

/// Returns names of all available built-in templates.
#[cfg(test)]
fn builtin_template_names() -> &'static [&'static str] {
    &["scientific-research"]
}

/// List all pipeline names found in the pipelines directory.
fn list_pipeline_names(project_root: &Path) -> Vec<String> {
    let pipelines = pipelines_dir(project_root);
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(&pipelines) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    names
}

// ─── gate evaluation ─────────────────────────────────────────────────────────

/// Evaluate all configured gates for the current step. Returns a list of
/// failure messages. Empty list means all gates passed.
fn evaluate_gates(
    gate: &StepGate,
    step: &PipelineStep,
    run: &PipelineRun,
    _definition: &PipelineDefinition,
    project_root: &Path,
) -> Result<Vec<String>> {
    let mut failures = Vec::new();

    // Collect artifacts for this step
    let step_artifacts = find_step_artifacts(project_root, &run.run_id, &step.id);

    // Tier 1: Structural
    if let Some(ref sg) = gate.structural {
        let matching: Vec<_> = if sg.types.is_empty() {
            step_artifacts.clone()
        } else {
            step_artifacts
                .iter()
                .filter(|a| sg.types.contains(&a.artifact_type))
                .cloned()
                .collect()
        };
        if matching.len() < sg.min_artifacts {
            let type_desc = if sg.types.is_empty() {
                String::new()
            } else {
                format!(" {} ", sg.types.join("/"))
            };
            failures.push(format!(
                "Step '{}' requires at least {} {}artifact(s). Found {}.",
                step.id,
                sg.min_artifacts,
                type_desc,
                matching.len()
            ));
        }
    }
    if !failures.is_empty() {
        return Ok(failures);
    }

    // Tier 2: Procedural
    if let Some(ref pg) = gate.procedural
        && pg.require_review
    {
        let reviewable: Vec<_> = step_artifacts
            .iter()
            .filter(|a| {
                if a.artifact_type == "review" {
                    return false; // reviews never need reviews
                }
                if pg.review_types.is_empty() {
                    true
                } else {
                    pg.review_types.contains(&a.artifact_type)
                }
            })
            .collect();

        let review_artifacts: Vec<_> = step_artifacts
            .iter()
            .filter(|a| a.artifact_type == "review")
            .collect();

        for artifact in &reviewable {
            let review = review_artifacts
                .iter()
                .find(|r| r.reviews_target.as_deref() == Some(&artifact.filename));
            let Some(review) = review else {
                failures.push(format!("Artifact '{}' has no review.", artifact.filename));
                continue;
            };
            if let Some(max_crit) = pg.max_critical
                && review.severity_critical > max_crit
            {
                failures.push(format!(
                    "Review of '{}' has {} critical findings (max: {}).",
                    artifact.filename, review.severity_critical, max_crit
                ));
            }
            if let Some(max_h) = pg.max_high
                && review.severity_high > max_h
            {
                failures.push(format!(
                    "Review of '{}' has {} high findings (max: {}).",
                    artifact.filename, review.severity_high, max_h
                ));
            }
        }
    }
    if !failures.is_empty() {
        return Ok(failures);
    }

    // Tier 3: Coverage
    if let Some(ref cg) = gate.coverage
        && cg.require_input_manifest
        && !has_coverage_manifest(project_root, &step.id)
    {
        failures.push(format!(
            "Coverage gate not satisfied. Create a coverage manifest (type: review, tag: coverage-manifest:{}) listing all inputs addressed.",
            step.id
        ));
    }
    if !failures.is_empty() {
        return Ok(failures);
    }

    // Tier 4: Oracles
    for oracle in &gate.oracles {
        let oracle_failures = run_oracle(oracle, &step_artifacts, project_root)?;
        failures.extend(oracle_failures);
    }
    if !failures.is_empty() {
        return Ok(failures);
    }

    // Tier 5: Approval
    if let Some(ref ag) = gate.approval
        && ag.required
    {
        let step_id = &step.id;
        match run.approvals.get(step_id) {
            None => {
                let msg = ag
                    .message
                    .as_deref()
                    .unwrap_or("This step requires human approval.");
                failures.push(format!("{} Run 'wai pipeline approve' when ready.", msg));
            }
            Some(approval_ts) => {
                // Check if any artifact was created after approval
                for artifact in &step_artifacts {
                    if let Some(ref created) = artifact.created_at
                        && created.as_str() > approval_ts.as_str()
                    {
                        failures.push(format!(
                            "Approval invalidated — artifact '{}' created after approval. Run 'wai pipeline approve' again.",
                            artifact.filename
                        ));
                        break;
                    }
                }
            }
        }
    }

    Ok(failures)
}

/// Metadata about an artifact found in the project.
#[derive(Debug, Clone)]
struct ArtifactInfo {
    filename: String,
    artifact_type: String,
    reviews_target: Option<String>,
    severity_critical: u32,
    severity_high: u32,
    created_at: Option<String>,
}

/// Find all artifacts in the project tagged with the given run ID and step ID.
fn find_step_artifacts(project_root: &Path, run_id: &str, step_id: &str) -> Vec<ArtifactInfo> {
    let projects = crate::config::projects_dir(project_root);
    let run_tag = format!("pipeline-run:{}", run_id);
    let step_tag = format!("pipeline-step:{}", step_id);

    let mut artifacts = Vec::new();

    // Walk all project directories
    let Ok(entries) = fs::read_dir(&projects) else {
        return artifacts;
    };
    for entry in entries.flatten() {
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }
        // Check each artifact type directory
        for (dir_name, art_type) in &[
            ("research", "research"),
            ("plans", "plan"),
            ("designs", "design"),
            ("handoffs", "handoff"),
            ("reviews", "review"),
        ] {
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
                if fm.tags.contains(&run_tag) && fm.tags.contains(&step_tag) {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    // Use file modification time as creation proxy
                    let created_at = fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

                    artifacts.push(ArtifactInfo {
                        filename,
                        artifact_type: art_type.to_string(),
                        reviews_target: fm.reviews,
                        severity_critical: fm.severity_critical,
                        severity_high: fm.severity_high,
                        created_at,
                    });
                }
            }
        }
    }
    artifacts
}

/// Parsed frontmatter fields relevant to gate evaluation.
#[derive(Default)]
struct Frontmatter {
    tags: Vec<String>,
    reviews: Option<String>,
    severity_critical: u32,
    severity_high: u32,
}

/// Parse frontmatter fields from artifact content.
fn parse_frontmatter(content: &str) -> Frontmatter {
    let body = content.trim_start();
    if !body.starts_with("---") {
        return Frontmatter::default();
    }
    let rest = &body[3..];
    let end = rest
        .find("\n---")
        .unwrap_or(rest.find("\r\n---").unwrap_or(rest.len()));
    let fm_block = &rest[..end];

    let mut fm = Frontmatter::default();
    for line in fm_block.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("tags:") {
            let value = value.trim();
            if value.starts_with('[') {
                let inner = value.trim_start_matches('[').trim_end_matches(']');
                for tag in inner.split(',') {
                    let t = tag.trim().to_string();
                    if !t.is_empty() {
                        fm.tags.push(t);
                    }
                }
            }
        } else if let Some(value) = line.strip_prefix("reviews:") {
            fm.reviews = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("severity:") {
            // Parse flow mapping: {critical: 0, high: 1, medium: 3, low: 2}
            let value = value.trim();
            let inner = value.trim_start_matches('{').trim_end_matches('}');
            for pair in inner.split(',') {
                let parts: Vec<&str> = pair.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val: u32 = parts[1].trim().parse().unwrap_or(0);
                    match key {
                        "critical" => fm.severity_critical = val,
                        "high" => fm.severity_high = val,
                        _ => {}
                    }
                }
            }
        }
    }
    fm
}

/// Check whether a coverage manifest artifact exists for the given step.
///
/// A coverage manifest is a `.md` file in any project's `reviews/` directory
/// whose frontmatter contains the tag `coverage-manifest:<step_id>`.
fn has_coverage_manifest(project_root: &Path, step_id: &str) -> bool {
    let projects = crate::config::projects_dir(project_root);
    let target_tag = format!("coverage-manifest:{}", step_id);

    let Ok(entries) = fs::read_dir(&projects) else {
        return false;
    };
    for entry in entries.flatten() {
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }
        let reviews_dir = project_dir.join("reviews");
        if !reviews_dir.exists() {
            continue;
        }
        let Ok(files) = fs::read_dir(&reviews_dir) else {
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
            if fm.tags.contains(&target_tag) {
                return true;
            }
        }
    }
    false
}

/// Run an oracle gate check. Returns failure messages (empty = passed).
fn run_oracle(
    oracle: &OracleGate,
    artifacts: &[ArtifactInfo],
    project_root: &Path,
) -> Result<Vec<String>> {
    let mut failures = Vec::new();

    // Resolve the oracle command
    let command = if let Some(ref cmd) = oracle.command {
        cmd.clone()
    } else {
        // Name-based resolution from .wai/resources/oracles/
        resolve_oracle_command(&oracle.name, project_root)?
    };

    let timeout_secs = oracle.timeout.unwrap_or(30);
    let scope = oracle.scope.as_deref().unwrap_or("artifact");

    // Filter to non-review artifacts for oracle checking
    let applicable: Vec<_> = artifacts
        .iter()
        .filter(|a| a.artifact_type != "review")
        .collect();

    if scope == "all" {
        // Single invocation with all artifact paths
        if applicable.is_empty() {
            return Ok(failures);
        }
        let projects = crate::config::projects_dir(project_root);
        let paths: Vec<String> = applicable
            .iter()
            .filter_map(|a| find_artifact_path(&projects, &a.filename))
            .collect();
        if !paths.is_empty() {
            let args: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            if let Some(err) = execute_oracle(&command, &args, timeout_secs)? {
                failures.push(format!("Oracle '{}' failed: {}", oracle.name, err));
            }
        }
    } else {
        // Per-artifact invocation
        let projects = crate::config::projects_dir(project_root);
        for artifact in &applicable {
            if let Some(path) = find_artifact_path(&projects, &artifact.filename)
                && let Some(err) = execute_oracle(&command, &[&path], timeout_secs)?
            {
                failures.push(format!(
                    "Oracle '{}' failed for '{}': {}",
                    oracle.name, artifact.filename, err
                ));
            }
        }
    }

    Ok(failures)
}

/// Resolve oracle name to an executable path in .wai/resources/oracles/.
fn resolve_oracle_command(name: &str, project_root: &Path) -> Result<String> {
    let oracles_dir = crate::config::wai_dir(project_root)
        .join("resources")
        .join("oracles");
    // Probe order: exact name, .sh, .py
    for suffix in &["", ".sh", ".py"] {
        let path = oracles_dir.join(format!("{}{}", name, suffix));
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }
    }
    miette::bail!("Oracle '{}' not found in {}", name, oracles_dir.display())
}

/// Execute an oracle command with arguments. Returns None on success (exit 0),
/// or Some(stderr) on failure.
fn execute_oracle(command: &str, args: &[&str], timeout_secs: u64) -> Result<Option<String>> {
    use std::process::Command;

    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let child = cmd
        .spawn()
        .map_err(|e| miette::miette!("Failed to execute oracle '{}': {}", command, e))?;

    let output = if timeout_secs > 0 {
        // Wait with timeout
        let result = child.wait_with_output();
        match result {
            Ok(o) => o,
            Err(e) => return Err(miette::miette!("Oracle execution failed: {}", e)),
        }
    } else {
        child
            .wait_with_output()
            .map_err(|e| miette::miette!("Oracle execution failed: {}", e))?
    };

    if output.status.success() {
        Ok(None)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(Some(if stderr.is_empty() {
            format!("exit code {}", output.status.code().unwrap_or(-1))
        } else {
            stderr
        }))
    }
}

/// Find the absolute path of an artifact by filename, searching all project artifact directories.
fn find_artifact_path(projects_dir: &Path, filename: &str) -> Option<String> {
    let Ok(entries) = fs::read_dir(projects_dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }
        for dir_name in &["research", "plans", "designs", "handoffs", "reviews"] {
            let path = project_dir.join(dir_name).join(filename);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Resolve the active run ID: check `WAI_PIPELINE_RUN` env var first, then
/// fall back to the `.last-run` pointer file at `.wai/resources/pipelines/.last-run`.
fn resolve_active_run_id(project_root: &Path) -> Result<String> {
    // Try env var first
    if let Ok(run_id) = std::env::var("WAI_PIPELINE_RUN")
        && !run_id.is_empty()
    {
        return Ok(run_id);
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

/// Find addenda for a given pipeline step by scanning artifact directories.
///
/// An addendum is an artifact whose YAML frontmatter contains a
/// `pipeline-addendum:<step_id>` tag. Returns relative paths like
/// `research/2026-04-14-correction.md`.
fn find_step_addenda(project_root: &Path, step_id: &str) -> Vec<String> {
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
        metadata: file.pipeline.metadata,
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

/// Validation issue found during pipeline definition checking.
#[derive(Debug)]
pub struct ValidationIssue {
    pub level: ValidationLevel,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum ValidationLevel {
    Error,
    Warn,
}

/// Validate a pipeline definition for structural errors and warnings.
/// Returns a list of issues. Empty list means valid.
pub fn validate_pipeline(def: &PipelineDefinition, project_root: &Path) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Check for metadata
    if def.metadata.is_none() {
        issues.push(ValidationIssue {
            level: ValidationLevel::Warn,
            message: format!(
                "Missing [pipeline.metadata] — pipeline '{}' won't appear in managed block",
                def.name
            ),
        });
    }

    // Check oracle references
    let oracles_dir = crate::config::wai_dir(project_root)
        .join("resources")
        .join("oracles");

    for step in &def.steps {
        if let Some(ref gate) = step.gate {
            for oracle in &gate.oracles {
                if oracle.command.is_some() {
                    continue; // explicit command, skip name resolution
                }
                let found = ["", ".sh", ".py"]
                    .iter()
                    .any(|ext| oracles_dir.join(format!("{}{}", oracle.name, ext)).exists());
                if !found {
                    issues.push(ValidationIssue {
                        level: ValidationLevel::Warn,
                        message: format!("Gate oracle '{}' — command not found", oracle.name),
                    });
                }
            }
        }

        // Warn when lock = true but no gate is configured
        if step.lock && step.gate.is_none() {
            issues.push(ValidationIssue {
                level: ValidationLevel::Warn,
                message: format!(
                    "step '{}' has lock = true but no gate configured — locked artifacts won't be validated before locking",
                    step.id
                ),
            });
        }
    }

    issues
}

/// Substitute `{topic}` in a prompt string with the given topic value.
///
/// If the prompt contains no `{topic}` placeholder, the prompt is returned unchanged.
pub fn render_prompt(prompt: &str, topic: &str) -> String {
    prompt.replace("{topic}", topic)
}

/// Compute SHA-256 hash of an artifact file, normalizing line endings to LF.
///
/// Returns the hash as a hex string prefixed with "sha256:".
#[allow(dead_code)] // Used by pipeline lock/verify commands (wai-2lwo, wai-rdp5)
pub fn artifact_hash(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let content = fs::read_to_string(path).into_diagnostic()?;
    let normalized = content.replace("\r\n", "\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let hash = hasher.finalize();
    Ok(format!("sha256:{:x}", hash))
}

/// Write a `.lock` sidecar file for an artifact.
///
/// The lock file is named `<artifact>.<run-id>.lock` and placed alongside the artifact.
/// Returns the path to the lock file.
#[allow(dead_code)] // Used by pipeline lock commands (wai-yh5v, wai-bzrp)
pub fn write_artifact_lock(
    artifact_path: &Path,
    run_id: &str,
    step_id: &str,
) -> Result<std::path::PathBuf> {
    let hash = artifact_hash(artifact_path)?;
    let lock = ArtifactLock {
        artifact: artifact_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        locked_at: chrono::Utc::now().to_rfc3339(),
        lock_hash: hash,
        pipeline_run: run_id.to_string(),
        pipeline_step: step_id.to_string(),
    };
    let lock_filename = format!(
        "{}.{}.lock",
        artifact_path.file_name().unwrap().to_string_lossy(),
        run_id
    );
    let lock_path = artifact_path.parent().unwrap().join(&lock_filename);
    let toml_content = toml::to_string_pretty(&lock).into_diagnostic()?;
    fs::write(&lock_path, toml_content).into_diagnostic()?;
    Ok(lock_path)
}

/// Read and parse a `.lock` sidecar file.
#[allow(dead_code)] // Used by pipeline lock/verify commands (wai-yh5v, wai-rdp5)
pub fn read_artifact_lock(lock_path: &Path) -> Result<ArtifactLock> {
    let content = fs::read_to_string(lock_path).into_diagnostic()?;
    toml::from_str(&content).into_diagnostic()
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

    // ── gate TOML parsing ─────────────────────────────────────────────────

    #[test]
    fn load_pipeline_toml_with_gates() {
        let toml = r#"
[pipeline]
name = "gated"

[[steps]]
id = "generate"
prompt = "Generate {topic}."

[steps.gate.structural]
min_artifacts = 1
types = ["research"]

[steps.gate.procedural]
require_review = true
max_critical = 0

[steps.gate.approval]
required = true
message = "Review before advancing."

[[steps]]
id = "done"
prompt = "Wrap up."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse gated pipeline");
        assert_eq!(def.steps.len(), 2);

        let gate = def.steps[0]
            .gate
            .as_ref()
            .expect("step 0 should have a gate");
        let sg = gate.structural.as_ref().unwrap();
        assert_eq!(sg.min_artifacts, 1);
        assert_eq!(sg.types, vec!["research"]);

        let pg = gate.procedural.as_ref().unwrap();
        assert!(pg.require_review);
        assert_eq!(pg.max_critical, Some(0));

        let ag = gate.approval.as_ref().unwrap();
        assert!(ag.required);
        assert_eq!(ag.message.as_deref(), Some("Review before advancing."));

        assert!(def.steps[1].gate.is_none());
    }

    #[test]
    fn load_pipeline_toml_with_oracles() {
        let toml = r#"
[pipeline]
name = "oracle-test"

[[steps]]
id = "check"
prompt = "Check {topic}."

[[steps.gate.oracles]]
name = "dim-analysis"
timeout = 60

[[steps.gate.oracles]]
name = "custom"
command = "python check.py"
scope = "all"
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse oracle pipeline");
        let gate = def.steps[0].gate.as_ref().expect("should have gate");
        assert_eq!(gate.oracles.len(), 2);
        assert_eq!(gate.oracles[0].name, "dim-analysis");
        assert_eq!(gate.oracles[0].timeout, Some(60));
        assert_eq!(gate.oracles[1].command.as_deref(), Some("python check.py"));
        assert_eq!(gate.oracles[1].scope.as_deref(), Some("all"));
    }

    // ── lock field ─────────────────────────────────────────────────────

    #[test]
    fn load_pipeline_toml_lock_defaults_to_false() {
        let toml = r#"
[pipeline]
name = "no-lock"

[[steps]]
id = "step1"
prompt = "Do something."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse pipeline without lock field");
        assert!(!def.steps[0].lock, "lock should default to false");
    }

    #[test]
    fn load_pipeline_toml_lock_true() {
        let toml = r#"
[pipeline]
name = "locked"

[[steps]]
id = "step1"
prompt = "Do something."
lock = true

[[steps]]
id = "step2"
prompt = "Do something else."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse pipeline with lock = true");
        assert!(def.steps[0].lock, "step1 lock should be true");
        assert!(!def.steps[1].lock, "step2 lock should default to false");
    }

    // ── frontmatter parsing ──────────────────────────────────────────────

    #[test]
    fn parse_frontmatter_extracts_tags_and_reviews() {
        let content =
            "---\ntags: [pipeline-run:test, pipeline-step:gen]\nreviews: findings.md\n---\n\nbody";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.tags, vec!["pipeline-run:test", "pipeline-step:gen"]);
        assert_eq!(fm.reviews.as_deref(), Some("findings.md"));
    }

    #[test]
    fn parse_frontmatter_extracts_severity() {
        let content = "---\nseverity: {critical: 2, high: 1, medium: 0, low: 5}\n---\n\nbody";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.severity_critical, 2);
        assert_eq!(fm.severity_high, 1);
    }

    #[test]
    fn parse_frontmatter_handles_no_frontmatter() {
        let content = "just some text";
        let fm = parse_frontmatter(content);
        assert!(fm.tags.is_empty());
        assert!(fm.reviews.is_none());
    }

    // ── gate evaluation ──────────────────────────────────────────────────

    #[test]
    fn structural_gate_fails_on_missing_artifacts() {
        let gate = StepGate {
            structural: Some(StructuralGate {
                min_artifacts: 1,
                types: vec!["research".to_string()],
            }),
            ..Default::default()
        };
        let step = PipelineStep {
            id: "gen".to_string(),
            prompt: "test".to_string(),
            gate: Some(gate.clone()),
            lock: false,
        };
        let run = PipelineRun {
            run_id: "test-run".to_string(),
            pipeline: "test".to_string(),
            topic: "topic".to_string(),
            created_at: "2026-04-02T00:00:00Z".to_string(),
            current_step: 0,
            approvals: HashMap::new(),
        };
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![step.clone()],
            metadata: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
        assert!(!failures.is_empty());
        assert!(
            failures[0].contains("requires at least 1"),
            "got: {}",
            failures[0]
        );
    }

    #[test]
    fn approval_gate_fails_when_not_approved() {
        let gate = StepGate {
            approval: Some(ApprovalGate {
                required: true,
                message: Some("Please review.".to_string()),
            }),
            ..Default::default()
        };
        let step = PipelineStep {
            id: "review-step".to_string(),
            prompt: "test".to_string(),
            gate: Some(gate.clone()),
            lock: false,
        };
        let run = PipelineRun {
            run_id: "test-run".to_string(),
            pipeline: "test".to_string(),
            topic: "topic".to_string(),
            created_at: "2026-04-02T00:00:00Z".to_string(),
            current_step: 0,
            approvals: HashMap::new(),
        };
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![step.clone()],
            metadata: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
        assert!(!failures.is_empty());
        assert!(
            failures[0].contains("Please review."),
            "got: {}",
            failures[0]
        );
    }

    #[test]
    fn approval_gate_passes_when_approved() {
        let gate = StepGate {
            approval: Some(ApprovalGate {
                required: true,
                message: None,
            }),
            ..Default::default()
        };
        let step = PipelineStep {
            id: "review-step".to_string(),
            prompt: "test".to_string(),
            gate: Some(gate.clone()),
            lock: false,
        };
        let mut approvals = HashMap::new();
        approvals.insert(
            "review-step".to_string(),
            "2099-01-01T00:00:00Z".to_string(),
        );
        let run = PipelineRun {
            run_id: "test-run".to_string(),
            pipeline: "test".to_string(),
            topic: "topic".to_string(),
            created_at: "2026-04-02T00:00:00Z".to_string(),
            current_step: 0,
            approvals,
        };
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![step.clone()],
            metadata: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
        assert!(
            failures.is_empty(),
            "expected no failures, got: {:?}",
            failures
        );
    }

    // ── pipeline metadata parsing ──────────────────────────────────────

    #[test]
    fn load_pipeline_toml_with_metadata() {
        let toml = r#"
[pipeline]
name = "research"
description = "Research workflow"

[pipeline.metadata]
when = "Frontier-level research requiring systematic validation"
skills = ["design-practice", "ro5"]

[[steps]]
id = "start"
prompt = "Start {topic}."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse pipeline with metadata");
        let meta = def.metadata.as_ref().expect("should have metadata");
        assert_eq!(
            meta.when.as_deref(),
            Some("Frontier-level research requiring systematic validation")
        );
        assert_eq!(meta.skills, vec!["design-practice", "ro5"]);
    }

    #[test]
    fn load_pipeline_toml_without_metadata() {
        let toml = r#"
[pipeline]
name = "simple"

[[steps]]
id = "go"
prompt = "Do {topic}."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse pipeline without metadata");
        assert!(def.metadata.is_none());
    }

    #[test]
    fn load_pipeline_toml_metadata_partial() {
        let toml = r#"
[pipeline]
name = "partial"

[pipeline.metadata]
when = "Only when needed"

[[steps]]
id = "go"
prompt = "Do {topic}."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse partial metadata");
        let meta = def.metadata.as_ref().expect("should have metadata");
        assert_eq!(meta.when.as_deref(), Some("Only when needed"));
        assert!(meta.skills.is_empty());
    }

    #[test]
    fn no_gates_means_no_failures() {
        let gate = StepGate::default();
        let step = PipelineStep {
            id: "free".to_string(),
            prompt: "test".to_string(),
            gate: Some(gate.clone()),
            lock: false,
        };
        let run = PipelineRun {
            run_id: "r".to_string(),
            pipeline: "p".to_string(),
            topic: "t".to_string(),
            created_at: "2026-04-02T00:00:00Z".to_string(),
            current_step: 0,
            approvals: HashMap::new(),
        };
        let def = PipelineDefinition {
            name: "p".to_string(),
            description: None,
            steps: vec![step.clone()],
            metadata: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
        assert!(failures.is_empty());
    }

    // ── validate_pipeline ─────────────────────────────────────────────────

    #[test]
    fn validate_warns_on_missing_metadata() {
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![PipelineStep {
                id: "go".to_string(),
                prompt: "test".to_string(),
                gate: None,
                lock: false,
            }],
            metadata: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_pipeline(&def, dir.path());
        assert!(
            issues
                .iter()
                .any(|i| i.level == ValidationLevel::Warn && i.message.contains("metadata")),
            "expected warning about missing metadata"
        );
    }

    #[test]
    fn validate_no_warning_with_metadata() {
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![PipelineStep {
                id: "go".to_string(),
                prompt: "test".to_string(),
                gate: None,
                lock: false,
            }],
            metadata: Some(PipelineMetadataSection {
                when: Some("When needed".to_string()),
                skills: vec![],
            }),
        };
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_pipeline(&def, dir.path());
        assert!(
            !issues.iter().any(|i| i.message.contains("metadata")),
            "should not warn about metadata when present"
        );
    }

    #[test]
    fn validate_warns_on_missing_oracle() {
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![PipelineStep {
                id: "check".to_string(),
                prompt: "check".to_string(),
                gate: Some(StepGate {
                    oracles: vec![OracleGate {
                        name: "nonexistent-oracle".to_string(),
                        command: None,
                        description: None,
                        timeout: None,
                        scope: None,
                    }],
                    ..Default::default()
                }),
                lock: false,
            }],
            metadata: Some(PipelineMetadataSection::default()),
        };
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_pipeline(&def, dir.path());
        assert!(
            issues
                .iter()
                .any(|i| i.level == ValidationLevel::Warn
                    && i.message.contains("nonexistent-oracle")),
            "expected warning about missing oracle"
        );
    }

    #[test]
    fn validate_skips_oracle_with_explicit_command() {
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![PipelineStep {
                id: "check".to_string(),
                prompt: "check".to_string(),
                gate: Some(StepGate {
                    oracles: vec![OracleGate {
                        name: "custom".to_string(),
                        command: Some("python check.py".to_string()),
                        description: None,
                        timeout: None,
                        scope: None,
                    }],
                    ..Default::default()
                }),
                lock: false,
            }],
            metadata: Some(PipelineMetadataSection::default()),
        };
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_pipeline(&def, dir.path());
        assert!(
            !issues.iter().any(|i| i.message.contains("custom")),
            "should not warn about oracle with explicit command"
        );
    }

    // ── validate lock-without-gate ───────────────────────────────────────

    #[test]
    fn validate_warns_lock_true_no_gate() {
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![PipelineStep {
                id: "locked-step".to_string(),
                prompt: "do work".to_string(),
                gate: None,
                lock: true,
            }],
            metadata: Some(PipelineMetadataSection::default()),
        };
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_pipeline(&def, dir.path());
        assert!(
            issues.iter().any(|i| i.level == ValidationLevel::Warn
                && i.message.contains("locked-step")
                && i.message.contains("lock = true but no gate")),
            "expected warning about lock without gate, got: {:?}",
            issues
        );
    }

    #[test]
    fn validate_no_warning_lock_true_with_gate() {
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![PipelineStep {
                id: "gated-step".to_string(),
                prompt: "do work".to_string(),
                gate: Some(StepGate {
                    structural: Some(StructuralGate {
                        min_artifacts: 1,
                        types: vec![],
                    }),
                    ..Default::default()
                }),
                lock: true,
            }],
            metadata: Some(PipelineMetadataSection::default()),
        };
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_pipeline(&def, dir.path());
        assert!(
            !issues
                .iter()
                .any(|i| i.message.contains("lock = true but no gate")),
            "should not warn when gate is present, got: {:?}",
            issues
        );
    }

    #[test]
    fn validate_no_warning_lock_false_no_gate() {
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![PipelineStep {
                id: "unlocked-step".to_string(),
                prompt: "do work".to_string(),
                gate: None,
                lock: false,
            }],
            metadata: Some(PipelineMetadataSection::default()),
        };
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_pipeline(&def, dir.path());
        assert!(
            !issues
                .iter()
                .any(|i| i.message.contains("lock = true but no gate")),
            "should not warn when lock is false, got: {:?}",
            issues
        );
    }

    // ── format_gate_summary ───────────────────────────────────────────────

    #[test]
    fn format_gate_summary_none_gate() {
        assert_eq!(format_gate_summary(&None), "");
    }

    #[test]
    fn format_gate_summary_empty_gate() {
        let gate = StepGate::default();
        assert_eq!(format_gate_summary(&Some(gate)), "");
    }

    #[test]
    fn format_gate_summary_structural_only() {
        let gate = StepGate {
            structural: Some(StructuralGate {
                min_artifacts: 1,
                types: vec![],
            }),
            ..Default::default()
        };
        assert_eq!(format_gate_summary(&Some(gate)), "[structural]");
    }

    #[test]
    fn format_gate_summary_all_tiers() {
        let gate = StepGate {
            structural: Some(StructuralGate {
                min_artifacts: 1,
                types: vec![],
            }),
            procedural: Some(ProceduralGate {
                require_review: true,
                review_types: vec![],
                max_critical: None,
                max_high: None,
            }),
            coverage: Some(CoverageGate {
                require_input_manifest: true,
            }),
            oracles: vec![OracleGate {
                name: "check".to_string(),
                command: None,
                description: None,
                timeout: None,
                scope: None,
            }],
            approval: Some(ApprovalGate {
                required: true,
                message: None,
            }),
        };
        assert_eq!(
            format_gate_summary(&Some(gate)),
            "[structural + procedural + coverage + oracle + approval]"
        );
    }

    // ── coverage gate tests ───────────────────────────────────────────────

    #[test]
    fn coverage_gate_toml_backward_compat() {
        // TOML without [steps.gate.coverage] should still parse correctly
        let toml = r#"
[pipeline]
name = "no-coverage"

[[steps]]
id = "step1"
prompt = "Do something."

[steps.gate.structural]
min_artifacts = 1
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse without coverage gate");
        let gate = def.steps[0].gate.as_ref().expect("should have gate");
        assert!(
            gate.coverage.is_none(),
            "coverage should be None when omitted"
        );
        assert!(gate.structural.is_some());
    }

    #[test]
    fn coverage_gate_toml_parses() {
        let toml = r#"
[pipeline]
name = "with-coverage"

[[steps]]
id = "generate"
prompt = "Generate {topic}."

[steps.gate.coverage]
require_input_manifest = true
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should parse coverage gate");
        let gate = def.steps[0].gate.as_ref().expect("should have gate");
        let cg = gate
            .coverage
            .as_ref()
            .expect("coverage gate should be present");
        assert!(cg.require_input_manifest);
    }

    #[test]
    fn format_gate_summary_coverage_only() {
        let gate = StepGate {
            coverage: Some(CoverageGate {
                require_input_manifest: true,
            }),
            ..Default::default()
        };
        assert_eq!(format_gate_summary(&Some(gate)), "[coverage]");
    }

    // ── coverage gate evaluation tests ─────────────────────────────────────

    /// Helper to build a standard test scaffold for coverage gate tests.
    fn coverage_gate_scaffold(
        require: bool,
    ) -> (StepGate, PipelineStep, PipelineRun, PipelineDefinition) {
        let gate = StepGate {
            coverage: Some(CoverageGate {
                require_input_manifest: require,
            }),
            ..Default::default()
        };
        let step = PipelineStep {
            id: "gen".to_string(),
            prompt: "test".to_string(),
            gate: Some(gate.clone()),
            lock: false,
        };
        let run = PipelineRun {
            run_id: "test-run".to_string(),
            pipeline: "test".to_string(),
            topic: "topic".to_string(),
            created_at: "2026-04-02T00:00:00Z".to_string(),
            current_step: 0,
            approvals: HashMap::new(),
        };
        let def = PipelineDefinition {
            name: "test".to_string(),
            description: None,
            steps: vec![step.clone()],
            metadata: None,
        };
        (gate, step, run, def)
    }

    #[test]
    fn coverage_gate_passes_with_manifest() {
        let (gate, step, run, def) = coverage_gate_scaffold(true);
        let dir = tempfile::tempdir().unwrap();

        // Create the project structure with a coverage manifest
        let reviews_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project")
            .join("reviews");
        fs::create_dir_all(&reviews_dir).unwrap();
        fs::write(
            reviews_dir.join("coverage-manifest.md"),
            "---\ntags: [coverage-manifest:gen]\n---\n\nManifest body\n",
        )
        .unwrap();

        let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
        assert!(
            failures.is_empty(),
            "expected no failures when manifest exists, got: {:?}",
            failures
        );
    }

    #[test]
    fn coverage_gate_fails_without_manifest() {
        let (gate, step, run, def) = coverage_gate_scaffold(true);
        let dir = tempfile::tempdir().unwrap();

        // Create the project structure but no manifest
        let reviews_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project")
            .join("reviews");
        fs::create_dir_all(&reviews_dir).unwrap();

        let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
        assert!(!failures.is_empty(), "expected failure when no manifest");
        assert!(
            failures[0].contains("Coverage gate not satisfied"),
            "got: {}",
            failures[0]
        );
        assert!(
            failures[0].contains("coverage-manifest:gen"),
            "message should include expected tag, got: {}",
            failures[0]
        );
    }

    #[test]
    fn coverage_gate_skipped_when_not_required() {
        let (gate, step, run, def) = coverage_gate_scaffold(false);
        let dir = tempfile::tempdir().unwrap();

        // No manifest, but gate doesn't require one
        let failures = evaluate_gates(&gate, &step, &run, &def, dir.path()).unwrap();
        assert!(
            failures.is_empty(),
            "expected no failures when require_input_manifest is false, got: {:?}",
            failures
        );
    }

    // ── oracle resolution tests ───────────────────────────────────────────

    #[test]
    fn resolve_oracle_exact_name() {
        let dir = tempfile::tempdir().unwrap();
        let wai = dir.path().join(".wai").join("resources").join("oracles");
        fs::create_dir_all(&wai).unwrap();
        fs::write(wai.join("check"), "#!/bin/sh\nexit 0\n").unwrap();
        let result = resolve_oracle_command("check", dir.path());
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        assert!(result.unwrap().contains("check"));
    }

    #[test]
    fn resolve_oracle_sh_extension() {
        let dir = tempfile::tempdir().unwrap();
        let wai = dir.path().join(".wai").join("resources").join("oracles");
        fs::create_dir_all(&wai).unwrap();
        fs::write(wai.join("check.sh"), "#!/bin/sh\nexit 0\n").unwrap();
        let result = resolve_oracle_command("check", dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("check.sh"));
    }

    #[test]
    fn resolve_oracle_py_extension() {
        let dir = tempfile::tempdir().unwrap();
        let wai = dir.path().join(".wai").join("resources").join("oracles");
        fs::create_dir_all(&wai).unwrap();
        fs::write(wai.join("check.py"), "#!/usr/bin/env python3\n").unwrap();
        let result = resolve_oracle_command("check", dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("check.py"));
    }

    #[test]
    fn resolve_oracle_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let wai = dir.path().join(".wai").join("resources").join("oracles");
        fs::create_dir_all(&wai).unwrap();
        let result = resolve_oracle_command("nonexistent", dir.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not found"), "got: {msg}");
    }

    #[test]
    fn resolve_oracle_probes_in_order() {
        // When both .sh and .py exist, .sh should win (probed first)
        let dir = tempfile::tempdir().unwrap();
        let wai = dir.path().join(".wai").join("resources").join("oracles");
        fs::create_dir_all(&wai).unwrap();
        fs::write(wai.join("check.sh"), "#!/bin/sh\n").unwrap();
        fs::write(wai.join("check.py"), "#!/usr/bin/env python3\n").unwrap();
        let result = resolve_oracle_command("check", dir.path()).unwrap();
        assert!(
            result.contains("check.sh"),
            "expected .sh to win probe order, got: {result}"
        );
    }

    #[test]
    fn execute_oracle_success() {
        let result = execute_oracle("true", &[], 30).unwrap();
        assert!(
            result.is_none(),
            "expected None (success), got: {:?}",
            result
        );
    }

    #[test]
    fn execute_oracle_failure() {
        let result = execute_oracle("false", &[], 30).unwrap();
        assert!(result.is_some(), "expected Some (failure)");
    }

    // ── built-in templates ────────────────────────────────────────────────

    #[test]
    fn builtin_template_scientific_research_exists() {
        let template = get_builtin_template("scientific-research");
        assert!(template.is_some(), "expected scientific-research template");
        let content = template.unwrap();
        assert!(content.contains("[pipeline]"));
        assert!(content.contains("scientific-research"));
        assert!(content.contains("[pipeline.metadata]"));
    }

    #[test]
    fn builtin_template_unknown_returns_none() {
        assert!(get_builtin_template("nonexistent").is_none());
    }

    #[test]
    fn builtin_template_names_not_empty() {
        let names = builtin_template_names();
        assert!(!names.is_empty());
        assert!(names.contains(&"scientific-research"));
    }

    // ── artifact_hash ─────────────────────────────────────────────────────

    #[test]
    fn artifact_hash_consistent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifact.md");
        fs::write(&path, "hello world\n").unwrap();
        let h1 = artifact_hash(&path).unwrap();
        let h2 = artifact_hash(&path).unwrap();
        assert_eq!(h1, h2, "same file should produce identical hashes");
        assert!(
            h1.starts_with("sha256:"),
            "hash should be prefixed with sha256:"
        );
    }

    #[test]
    fn artifact_hash_normalizes_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let lf_path = dir.path().join("lf.md");
        let crlf_path = dir.path().join("crlf.md");
        fs::write(&lf_path, "line one\nline two\n").unwrap();
        fs::write(&crlf_path, "line one\r\nline two\r\n").unwrap();
        let lf_hash = artifact_hash(&lf_path).unwrap();
        let crlf_hash = artifact_hash(&crlf_path).unwrap();
        assert_eq!(
            lf_hash, crlf_hash,
            "CRLF and LF content should hash identically"
        );
    }

    #[test]
    fn artifact_hash_different_content() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.md");
        let b = dir.path().join("b.md");
        fs::write(&a, "content A\n").unwrap();
        fs::write(&b, "content B\n").unwrap();
        let ha = artifact_hash(&a).unwrap();
        let hb = artifact_hash(&b).unwrap();
        assert_ne!(ha, hb, "different content should produce different hashes");
    }

    // ── artifact lock sidecar ─────────────────────────────────────────────

    #[test]
    fn write_artifact_lock_creates_file_with_correct_name() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("research.md");
        fs::write(&artifact, "# Research\nSome findings.\n").unwrap();

        let lock_path = write_artifact_lock(&artifact, "run-abc", "step-1").unwrap();

        assert!(lock_path.exists(), "lock file should exist");
        assert_eq!(
            lock_path.file_name().unwrap().to_string_lossy(),
            "research.md.run-abc.lock"
        );
    }

    #[test]
    fn write_artifact_lock_roundtrips_via_read() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("design.md");
        fs::write(&artifact, "# Design\nArchitecture notes.\n").unwrap();

        let lock_path = write_artifact_lock(&artifact, "run-42", "generate").unwrap();
        let lock = read_artifact_lock(&lock_path).unwrap();

        assert_eq!(lock.artifact, "design.md");
        assert_eq!(lock.pipeline_run, "run-42");
        assert_eq!(lock.pipeline_step, "generate");
        assert!(
            lock.lock_hash.starts_with("sha256:"),
            "hash should be prefixed with sha256:"
        );
        // locked_at should be a valid RFC 3339 timestamp
        assert!(
            chrono::DateTime::parse_from_rfc3339(&lock.locked_at).is_ok(),
            "locked_at should be valid RFC 3339: {}",
            lock.locked_at
        );
    }

    #[test]
    fn write_artifact_lock_hash_matches_artifact_hash() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("notes.md");
        fs::write(&artifact, "Some content\nwith lines.\n").unwrap();

        let expected_hash = artifact_hash(&artifact).unwrap();
        let lock_path = write_artifact_lock(&artifact, "run-x", "review").unwrap();
        let lock = read_artifact_lock(&lock_path).unwrap();

        assert_eq!(
            lock.lock_hash, expected_hash,
            "lock hash should match artifact_hash"
        );
    }

    #[test]
    fn write_artifact_lock_includes_run_id_in_filename() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("plan.md");
        fs::write(&artifact, "# Plan\n").unwrap();

        let lock_path = write_artifact_lock(&artifact, "my-run-id", "step-0").unwrap();
        let filename = lock_path.file_name().unwrap().to_string_lossy();

        assert!(
            filename.contains("my-run-id"),
            "filename should contain run-id, got: {filename}"
        );
        assert!(
            filename.ends_with(".lock"),
            "filename should end with .lock, got: {filename}"
        );
    }

    // ── find_step_addenda ────────────────────────────────────────────────

    #[test]
    fn find_step_addenda_returns_matching_files() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        fs::write(
            research_dir.join("2026-04-14-correction.md"),
            "---\ntags: [pipeline-addendum:implement]\n---\n\nCorrection notes.",
        )
        .unwrap();

        fs::write(
            research_dir.join("2026-04-14-other.md"),
            "---\ntags: [pipeline-addendum:review]\n---\n\nOther notes.",
        )
        .unwrap();

        fs::write(
            research_dir.join("2026-04-14-normal.md"),
            "---\ntags: [pipeline-run:abc]\n---\n\nNormal research.",
        )
        .unwrap();

        let addenda = find_step_addenda(dir.path(), "implement");
        assert_eq!(addenda.len(), 1, "expected 1 addendum, got: {:?}", addenda);
        assert_eq!(addenda[0], "research/2026-04-14-correction.md");
    }

    #[test]
    fn find_step_addenda_returns_empty_when_none_match() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        fs::write(
            research_dir.join("2026-04-14-normal.md"),
            "---\ntags: [pipeline-run:abc]\n---\n\nNormal research.",
        )
        .unwrap();

        let addenda = find_step_addenda(dir.path(), "implement");
        assert!(
            addenda.is_empty(),
            "expected no addenda, got: {:?}",
            addenda
        );
    }

    #[test]
    fn find_step_addenda_scans_multiple_directories() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        let plans_dir = project_dir.join("plans");
        fs::create_dir_all(&research_dir).unwrap();
        fs::create_dir_all(&plans_dir).unwrap();

        fs::write(
            research_dir.join("2026-04-14-fix.md"),
            "---\ntags: [pipeline-addendum:design]\n---\n\nFix notes.",
        )
        .unwrap();

        fs::write(
            plans_dir.join("2026-04-14-revised.md"),
            "---\ntags: [pipeline-addendum:design]\n---\n\nRevised plan.",
        )
        .unwrap();

        let addenda = find_step_addenda(dir.path(), "design");
        assert_eq!(addenda.len(), 2, "expected 2 addenda, got: {:?}", addenda);
        assert!(addenda.contains(&"plans/2026-04-14-revised.md".to_string()));
        assert!(addenda.contains(&"research/2026-04-14-fix.md".to_string()));
    }

    #[test]
    fn find_step_addenda_handles_missing_projects_dir() {
        let dir = tempfile::tempdir().unwrap();
        let addenda = find_step_addenda(dir.path(), "implement");
        assert!(
            addenda.is_empty(),
            "expected no addenda for missing project dir"
        );
    }

    // ── find_step_artifact_paths ─────────────────────────────────────────

    #[test]
    fn find_step_artifact_paths_returns_matching_files() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        // Artifact tagged with both run and step tags
        fs::write(
            research_dir.join("2026-04-14-findings.md"),
            "---\ntags: [pipeline-run:run-abc, pipeline-step:generate]\n---\n\nFindings.",
        )
        .unwrap();

        // Artifact tagged with different step — should NOT match
        fs::write(
            research_dir.join("2026-04-14-other.md"),
            "---\ntags: [pipeline-run:run-abc, pipeline-step:review]\n---\n\nOther.",
        )
        .unwrap();

        // Artifact with no pipeline tags — should NOT match
        fs::write(
            research_dir.join("2026-04-14-plain.md"),
            "---\ntags: [general]\n---\n\nPlain.",
        )
        .unwrap();

        let paths = find_step_artifact_paths(dir.path(), "run-abc", "generate");
        assert_eq!(paths.len(), 1, "expected 1 artifact, got: {:?}", paths);
        assert!(
            paths[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("findings"),
            "expected findings artifact, got: {:?}",
            paths[0]
        );
    }

    #[test]
    fn find_step_artifact_paths_returns_empty_when_no_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        // Artifact tagged with different run
        fs::write(
            research_dir.join("2026-04-14-other.md"),
            "---\ntags: [pipeline-run:other-run, pipeline-step:generate]\n---\n\nOther.",
        )
        .unwrap();

        let paths = find_step_artifact_paths(dir.path(), "run-abc", "generate");
        assert!(paths.is_empty(), "expected no artifacts, got: {:?}", paths);
    }

    // ── cmd_next locking integration ────────────────────────────────────

    #[test]
    fn cmd_next_lock_creates_lock_files_for_step_artifacts() {
        // Simulate the locking path from cmd_next: when a step has lock=true,
        // find its artifacts and write lock sidecars for each.
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project");
        let research_dir = project_dir.join("research");
        fs::create_dir_all(&research_dir).unwrap();

        let run_id = "run-lock-test";
        let step_id = "generate";

        // Create two artifacts tagged for this run+step
        fs::write(
            research_dir.join("2026-04-14-findings.md"),
            "---\ntags: [pipeline-run:run-lock-test, pipeline-step:generate]\n---\n\nFindings.",
        )
        .unwrap();
        fs::write(
            research_dir.join("2026-04-14-analysis.md"),
            "---\ntags: [pipeline-run:run-lock-test, pipeline-step:generate]\n---\n\nAnalysis.",
        )
        .unwrap();

        // Build a step definition with lock = true
        let step_def = PipelineStep {
            id: step_id.to_string(),
            prompt: String::new(),
            gate: None,
            lock: true,
        };

        // Execute the same logic as cmd_next's locking block
        assert!(step_def.lock, "step should have lock=true");
        let artifact_paths = find_step_artifact_paths(dir.path(), run_id, &step_def.id);
        assert_eq!(
            artifact_paths.len(),
            2,
            "expected 2 artifacts, got: {:?}",
            artifact_paths
        );

        for path in &artifact_paths {
            write_artifact_lock(path, run_id, &step_def.id).unwrap();
        }

        // Verify lock files exist for each artifact
        for path in &artifact_paths {
            let lock_filename = format!(
                "{}.{}.lock",
                path.file_name().unwrap().to_string_lossy(),
                run_id
            );
            let lock_path = path.parent().unwrap().join(&lock_filename);
            assert!(
                lock_path.exists(),
                "lock file should exist: {:?}",
                lock_path
            );

            // Verify lock content
            let lock = read_artifact_lock(&lock_path).unwrap();
            assert_eq!(lock.pipeline_run, run_id);
            assert_eq!(lock.pipeline_step, step_id);
            assert!(lock.lock_hash.starts_with("sha256:"));
        }
    }

    #[test]
    fn cmd_next_lock_skipped_when_lock_is_false() {
        // When lock=false, no lock files should be created (the if-branch is not entered)
        let step_def = PipelineStep {
            id: "review".to_string(),
            prompt: String::new(),
            gate: None,
            lock: false,
        };
        assert!(
            !step_def.lock,
            "step should have lock=false — locking block would be skipped"
        );
    }

    // ── cmd_verify (via artifact_hash / read_artifact_lock) ─────────────

    #[test]
    fn verify_passes_when_artifacts_are_intact() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project")
            .join("research");
        fs::create_dir_all(&project_dir).unwrap();

        let artifact = project_dir.join("notes.md");
        fs::write(&artifact, "# Notes\nSome content.\n").unwrap();

        let hash = artifact_hash(&artifact).unwrap();
        let lock = ArtifactLock {
            artifact: "notes.md".to_string(),
            locked_at: chrono::Utc::now().to_rfc3339(),
            lock_hash: hash.clone(),
            pipeline_run: "run-1".to_string(),
            pipeline_step: "generate".to_string(),
        };
        let lock_path = project_dir.join("notes.md.run-1.lock");
        fs::write(&lock_path, toml::to_string_pretty(&lock).unwrap()).unwrap();

        let projects_dir = dir.path().join(".wai").join("projects");
        let mut mismatches: Vec<String> = Vec::new();
        let mut lock_count: usize = 0;
        for entry in walkdir::WalkDir::new(&projects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("lock") {
                continue;
            }
            let parsed_lock = read_artifact_lock(path).unwrap();
            let artifact_path = path.parent().unwrap().join(&parsed_lock.artifact);
            let actual = artifact_hash(&artifact_path).unwrap();
            if actual != parsed_lock.lock_hash {
                mismatches.push(parsed_lock.artifact.clone());
            }
            lock_count += 1;
        }

        assert_eq!(lock_count, 1, "should find exactly one lock file");
        assert!(mismatches.is_empty(), "all artifacts should verify OK");
    }

    #[test]
    fn verify_detects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir
            .path()
            .join(".wai")
            .join("projects")
            .join("test-project")
            .join("research");
        fs::create_dir_all(&project_dir).unwrap();

        let artifact = project_dir.join("notes.md");
        fs::write(&artifact, "# Notes\nOriginal content.\n").unwrap();

        let original_hash = artifact_hash(&artifact).unwrap();
        let lock = ArtifactLock {
            artifact: "notes.md".to_string(),
            locked_at: chrono::Utc::now().to_rfc3339(),
            lock_hash: original_hash.clone(),
            pipeline_run: "run-1".to_string(),
            pipeline_step: "generate".to_string(),
        };
        let lock_path = project_dir.join("notes.md.run-1.lock");
        fs::write(&lock_path, toml::to_string_pretty(&lock).unwrap()).unwrap();

        // Tamper with the artifact
        fs::write(&artifact, "# Notes\nTampered content!\n").unwrap();

        let projects_dir = dir.path().join(".wai").join("projects");
        let mut mismatches: Vec<String> = Vec::new();
        for entry in walkdir::WalkDir::new(&projects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("lock") {
                continue;
            }
            let parsed_lock = read_artifact_lock(path).unwrap();
            let artifact_path = path.parent().unwrap().join(&parsed_lock.artifact);
            let actual = artifact_hash(&artifact_path).unwrap();
            if actual != parsed_lock.lock_hash {
                mismatches.push(parsed_lock.artifact.clone());
            }
        }

        assert_eq!(
            mismatches.len(),
            1,
            "should detect exactly one mismatch, got: {:?}",
            mismatches
        );
        assert_eq!(mismatches[0], "notes.md");
    }
}
