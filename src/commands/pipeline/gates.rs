use crate::commands::pipeline::{ApprovalGate, ProceduralGate, StructuralGate};
use miette::Result;
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;

use super::{OracleGate, PipelineDefinition, PipelineRun, PipelineStep, StepGate};

// ─── Artifact scanning helpers ────────────────────────────────────────────────

/// Metadata about an artifact found in the project.
#[derive(Debug, Clone)]
pub(super) struct ArtifactInfo {
    pub(super) filename: String,
    pub(super) artifact_type: String,
    pub(super) reviews_target: Option<String>,
    pub(super) severity_critical: u32,
    pub(super) severity_high: u32,
    pub(super) created_at: Option<String>,
}

/// Parsed frontmatter fields relevant to gate evaluation.
#[derive(Default)]
pub(super) struct Frontmatter {
    pub(super) tags: Vec<String>,
    pub(super) reviews: Option<String>,
    pub(super) severity_critical: u32,
    pub(super) severity_high: u32,
}

/// Parse frontmatter fields from artifact content.
pub(super) fn parse_frontmatter(content: &str) -> Frontmatter {
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
            parse_tags(&mut fm, value);
        } else if let Some(value) = line.strip_prefix("reviews:") {
            fm.reviews = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("severity:") {
            parse_severity(&mut fm, value);
        }
    }
    fm
}

/// Parse a `tags:` flow sequence value: `[a, b]`.
fn parse_tags(fm: &mut Frontmatter, value: &str) {
    let value = value.trim();
    if !value.starts_with('[') {
        return;
    }
    let inner = value.trim_start_matches('[').trim_end_matches(']');
    for tag in inner.split(',') {
        let t = tag.trim().to_string();
        if !t.is_empty() {
            fm.tags.push(t);
        }
    }
}

/// Parse a `severity:` flow mapping value: `{critical: 0, high: 1}`.
fn parse_severity(fm: &mut Frontmatter, value: &str) {
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

/// Find all artifacts in the project tagged with the given run ID and step ID.
pub(super) fn find_step_artifacts(
    project_root: &Path,
    run_id: &str,
    step_id: &str,
) -> Vec<ArtifactInfo> {
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
        if project_dir.is_dir() {
            collect_artifacts_from_project(&project_dir, &run_tag, &step_tag, &mut artifacts);
        }
    }
    artifacts
}

/// Artifact-type directories scanned by [`find_step_artifacts`].
const ARTIFACT_DIRS: [(&str, &str); 5] = [
    ("research", "research"),
    ("plans", "plan"),
    ("designs", "design"),
    ("handoffs", "handoff"),
    ("reviews", "review"),
];

fn collect_artifacts_from_project(
    project_dir: &Path,
    run_tag: &str,
    step_tag: &str,
    artifacts: &mut Vec<ArtifactInfo>,
) {
    // Check each artifact type directory
    for (dir_name, art_type) in &ARTIFACT_DIRS {
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
            if fm.tags.contains(&run_tag.to_string()) && fm.tags.contains(&step_tag.to_string()) {
                artifacts.push(artifact_info(&path, art_type, &fm));
            }
        }
    }
}

fn artifact_info(path: &Path, art_type: &str, fm: &Frontmatter) -> ArtifactInfo {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    // Use file modification time as creation proxy
    let created_at = fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

    ArtifactInfo {
        filename,
        artifact_type: art_type.to_string(),
        reviews_target: fm.reviews.clone(),
        severity_critical: fm.severity_critical,
        severity_high: fm.severity_high,
        created_at,
    }
}

/// Find all artifact file paths tagged with the given run ID and step ID.
///
/// Similar to [`find_step_artifacts`] but returns full `PathBuf`s suitable for
/// passing to [`write_artifact_lock`].
pub(super) fn find_step_artifact_paths(
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

/// Check whether a coverage manifest artifact exists for the given step.
///
/// A coverage manifest is a `.md` file in any project's `reviews/` directory
/// whose frontmatter contains the tag `coverage-manifest:<step_id>`.
pub(super) fn has_coverage_manifest(project_root: &Path, step_id: &str) -> bool {
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

// ─── Gate evaluation ──────────────────────────────────────────────────────────

/// Evaluate all configured gates for the current step. Returns a list of
/// failure messages. Empty list means all gates passed.
pub(super) fn evaluate_gates(
    gate: &StepGate,
    step: &PipelineStep,
    run: &PipelineRun,
    _definition: &PipelineDefinition,
    project_root: &Path,
) -> Result<Vec<String>> {
    let step_artifacts = find_step_artifacts(project_root, &run.run_id, &step.id);

    // Tier 1: Structural
    if let Some(ref sg) = gate.structural {
        let failures = structural_failures(sg, step, &step_artifacts);
        if !failures.is_empty() {
            return Ok(failures);
        }
    }

    // Tier 2: Procedural
    if let Some(ref pg) = gate.procedural
        && pg.require_review
    {
        let failures = procedural_review_failures(pg, &step_artifacts);
        if !failures.is_empty() {
            return Ok(failures);
        }
    }

    // Tier 3: Coverage
    if let Some(ref cg) = gate.coverage
        && cg.require_input_manifest
        && !has_coverage_manifest(project_root, &step.id)
    {
        return Ok(vec![format!(
            "Coverage gate not satisfied. Create a coverage manifest (type: review, tag: coverage-manifest:{}) listing all inputs addressed.",
            step.id
        )]);
    }

    // Tier 4: Oracles
    let mut failures = Vec::new();
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
        return Ok(approval_failures(ag, run, step, &step_artifacts));
    }

    Ok(failures)
}

/// Tier 1 — structural: minimum artifact count per type.
fn structural_failures(
    sg: &StructuralGate,
    step: &PipelineStep,
    step_artifacts: &[ArtifactInfo],
) -> Vec<String> {
    let matching: Vec<_> = if sg.types.is_empty() {
        step_artifacts.to_vec()
    } else {
        step_artifacts
            .iter()
            .filter(|a| sg.types.contains(&a.artifact_type))
            .cloned()
            .collect()
    };
    if matching.len() >= sg.min_artifacts {
        return Vec::new();
    }
    let type_desc = if sg.types.is_empty() {
        String::new()
    } else {
        format!(" {} ", sg.types.join("/"))
    };
    vec![format!(
        "Step '{}' requires at least {} {}artifact(s). Found {}.",
        step.id,
        sg.min_artifacts,
        type_desc,
        matching.len()
    )]
}

/// Tier 2 — procedural: every reviewable artifact must have a review whose
/// critical/high finding counts stay within the configured maxima.
fn procedural_review_failures(pg: &ProceduralGate, step_artifacts: &[ArtifactInfo]) -> Vec<String> {
    let mut failures = Vec::new();
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
    failures
}

/// Tier 5 — approval: an approval must exist and predate every artifact.
fn approval_failures(
    ag: &ApprovalGate,
    run: &PipelineRun,
    step: &PipelineStep,
    step_artifacts: &[ArtifactInfo],
) -> Vec<String> {
    let step_id = &step.id;
    match run.approvals.get(step_id) {
        None => {
            let msg = ag
                .message
                .as_deref()
                .unwrap_or("This step requires human approval.");
            vec![format!("{} Run 'wai pipeline approve' when ready.", msg)]
        }
        Some(approval_ts) => {
            // Check if any artifact was created after approval
            for artifact in step_artifacts {
                if let Some(ref created) = artifact.created_at
                    && created.as_str() > approval_ts.as_str()
                {
                    return vec![format!(
                        "Approval invalidated — artifact '{}' created after approval. Run 'wai pipeline approve' again.",
                        artifact.filename
                    )];
                }
            }
            Vec::new()
        }
    }
}

// ─── Oracle helpers ───────────────────────────────────────────────────────────

/// Run an oracle gate check. Returns failure messages (empty = passed).
pub(super) fn run_oracle(
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
pub(super) fn resolve_oracle_command(name: &str, project_root: &Path) -> Result<String> {
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
    Err(miette::miette!(
        "Oracle '{}' not found in {}",
        name,
        oracles_dir.display()
    ))
}

/// Execute an oracle command with arguments. Returns None on success (exit 0),
/// or Some(stderr) on failure.
pub(super) fn execute_oracle(
    command: &str,
    args: &[&str],
    timeout_secs: u64,
) -> Result<Option<String>> {
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

// ─── Display helpers ──────────────────────────────────────────────────────────

/// Format a one-line gate summary for a step.
pub(super) fn format_gate_summary(gate: &Option<StepGate>) -> String {
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

/// Print gate status for a single step.
pub(super) fn print_gate_status(
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

    if let Some(ref sg) = gate.structural {
        print_structural_gate(sg, live, &step_artifacts);
    }
    if let Some(ref pg) = gate.procedural
        && pg.require_review
    {
        print_procedural_gate(pg, live, &step_artifacts);
    }
    if let Some(ref cg) = gate.coverage
        && cg.require_input_manifest
    {
        println!("    {} Coverage: require input manifest", "•".dimmed(),);
    }
    print_oracle_gates(&gate.oracles);
    if let Some(ref ag) = gate.approval
        && ag.required
    {
        print_approval_gate(ag, run, &step.id);
    }

    println!();
    Ok(())
}

/// ✓ when live and satisfied, ✗ when live and unsatisfied, • when dry.
fn pass_icon(passed: bool, live: bool) -> String {
    match (live, passed) {
        (true, true) => "✓".green().to_string(),
        (true, false) => "✗".red().to_string(),
        (false, _) => "•".dimmed().to_string(),
    }
}

fn print_structural_gate(sg: &StructuralGate, live: bool, step_artifacts: &[ArtifactInfo]) {
    let type_desc = if sg.types.is_empty() {
        "any".to_string()
    } else {
        sg.types.join("/")
    };
    let matching: Vec<_> = if sg.types.is_empty() {
        step_artifacts.to_vec()
    } else {
        step_artifacts
            .iter()
            .filter(|a| sg.types.contains(&a.artifact_type))
            .cloned()
            .collect()
    };
    let icon = pass_icon(matching.len() >= sg.min_artifacts, live);
    let found = if live {
        format!(" — found {}", matching.len())
    } else {
        String::new()
    };
    println!(
        "    {} Structural: min {} {} artifact(s){}",
        icon, sg.min_artifacts, type_desc, found
    );
}

fn print_procedural_gate(pg: &ProceduralGate, live: bool, step_artifacts: &[ArtifactInfo]) {
    let type_desc = if pg.review_types.is_empty() {
        "all (except review)".to_string()
    } else {
        pg.review_types.join("/")
    };
    if live {
        let reviews: Vec<_> = step_artifacts
            .iter()
            .filter(|a| a.artifact_type == "review")
            .collect();
        let missing: Vec<_> = step_artifacts
            .iter()
            .filter(|a| {
                if a.artifact_type == "review" {
                    return false;
                }
                if !pg.review_types.is_empty() && !pg.review_types.contains(&a.artifact_type) {
                    return false;
                }
                !reviews
                    .iter()
                    .any(|r| r.reviews_target.as_deref() == Some(&a.filename))
            })
            .collect();
        let icon = pass_icon(missing.is_empty(), live);
        println!(
            "    {} Procedural: require review for {} types — {} unreviewed",
            icon,
            type_desc,
            missing.len()
        );
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

fn print_oracle_gates(oracles: &[OracleGate]) {
    for oracle in oracles {
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
}

fn print_approval_gate(ag: &ApprovalGate, run: Option<&PipelineRun>, step_id: &str) {
    let msg = ag.message.as_deref().unwrap_or("required");
    if let Some(r) = run {
        let icon = pass_icon(r.approvals.contains_key(step_id), true);
        let state = if r.approvals.contains_key(step_id) {
            "approved"
        } else {
            "pending"
        };
        println!("    {} Approval: {} ({})", icon, msg, state);
    } else {
        println!("    {} Approval: {}", "•".dimmed(), msg);
    }
}
