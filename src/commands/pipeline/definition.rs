use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::config::pipelines_dir;

use super::{
    PipelineDefinition, PipelineMetadataSection, PipelineStep, ValidationIssue, ValidationLevel,
};

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

/// Validate that a pipeline name is non-empty, lowercase, alphanumeric + hyphens.
pub(super) fn validate_pipeline_name(name: &str) -> Result<()> {
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

/// List all pipeline names found in the pipelines directory.
pub(super) fn list_pipeline_names(project_root: &Path) -> Vec<String> {
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
