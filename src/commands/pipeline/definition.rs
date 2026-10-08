use miette::{IntoDiagnostic, Result};
use std::collections::{HashMap, HashSet};
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
    #[serde(default)]
    blocks: Vec<SharedBlock>,
}

/// One reusable content block from the `[[blocks]]` TOML table (wai-vsn6).
///
/// Step prompts reference a block as `{{block:<name>}}`; the reference is
/// replaced by the block's `content` when the definition is loaded, so all
/// downstream rendering paths are unaffected.
#[derive(serde::Deserialize)]
struct SharedBlock {
    name: String,
    content: String,
}

/// Marker that starts a shared-block reference in a step prompt.
const BLOCK_REF_START: &str = "{{block:";
/// Marker that ends a shared-block reference in a step prompt.
const BLOCK_REF_END: &str = "}}";
/// Maximum expansion depth for (possibly nested) block references.
/// A depth beyond this indicates a reference cycle.
const MAX_BLOCK_DEPTH: usize = 32;

/// Replace every `{{block:<name>}}` reference in `text` with the content of
/// the block called `<name>` from `blocks`.
///
/// Nested references (a block's content referencing another block) are
/// resolved by repeated passes. Unknown names and malformed markers are hard
/// errors; a cycle exceeds [`MAX_BLOCK_DEPTH`] and errors as well.
fn resolve_block_refs(
    text: &str,
    context: &str,
    blocks: &HashMap<String, String>,
) -> Result<String> {
    let mut current = text.to_string();
    for _ in 0..=MAX_BLOCK_DEPTH {
        if !current.contains(BLOCK_REF_START) {
            return Ok(current);
        }
        current = expand_block_refs_once(&current, context, blocks)?;
    }
    Err(miette::miette!(
        "cyclic or too deeply nested block reference in {}",
        context
    ))
}

/// Perform one expansion pass over `text`, replacing every well-formed
/// `{{block:<name>}}` marker with the referenced block's content.
fn expand_block_refs_once(
    text: &str,
    context: &str,
    blocks: &HashMap<String, String>,
) -> Result<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(BLOCK_REF_START) {
        out.push_str(&rest[..start]);
        let after_marker = &rest[start + BLOCK_REF_START.len()..];
        let Some(end) = after_marker.find(BLOCK_REF_END) else {
            miette::bail!(
                "malformed block reference (missing '{}') in {}",
                BLOCK_REF_END,
                context
            );
        };
        let name = &after_marker[..end];
        let content = blocks
            .get(name)
            .ok_or_else(|| miette::miette!("unknown block '{}' referenced in {}", name, context))?;
        out.push_str(content);
        rest = &after_marker[end + BLOCK_REF_END.len()..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Build the block name → content map from the `[[blocks]]` entries,
/// rejecting duplicate names.
fn build_block_map(blocks: &[SharedBlock]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for block in blocks {
        if map
            .insert(block.name.clone(), block.content.clone())
            .is_some()
        {
            miette::bail!("duplicate block name: {}", block.name);
        }
    }
    Ok(map)
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

    // Shared-block resolution (wai-vsn6): expand `{{block:<name>}}`
    // references in step prompts at load time so every downstream rendering
    // path is unchanged. Unknown/malformed/cyclic references fail loudly.
    let blocks = build_block_map(&file.blocks)?;
    let mut steps = file.steps;
    for step in &mut steps {
        step.prompt = resolve_block_refs(&step.prompt, &format!("step '{}'", step.id), &blocks)?;
    }

    let def = PipelineDefinition {
        name: file.pipeline.name,
        description: file.pipeline.description,
        steps,
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

// ── Shared-block tests (wai-vsn6) ─────────────────────────────────────────────

#[cfg(test)]
mod blocks_tests {
    use super::super::{render_prompt, setup};
    use super::{SharedBlock, build_block_map, load_pipeline_toml, resolve_block_refs};
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_toml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("create tempfile");
        f.write_all(content.as_bytes()).expect("write toml");
        f
    }

    /// Rendered prompts for every built-in template must be byte-identical
    /// to the pre-mechanism baseline (hard constraint of wai-vsn6). The
    /// baseline was captured with the pre-change engine; any drift in
    /// rendering — from block expansion or render_prompt itself — fails here.
    #[test]
    fn builtin_templates_render_byte_identical_to_baseline() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pipeline-rendered-baseline-vsn6");
        assert!(
            dir.is_dir(),
            "baseline fixtures missing at {}",
            dir.display()
        );
        for name in setup::builtin_template_names() {
            let baseline = std::fs::read_to_string(dir.join(format!("{}.txt", name)))
                .unwrap_or_else(|e| panic!("baseline for {name}: {e}"));
            let content = setup::get_builtin_template(name)
                .unwrap_or_else(|| panic!("no built-in template {name}"));
            let f = write_toml(content);
            let def =
                load_pipeline_toml(f.path()).unwrap_or_else(|e| panic!("{name}: load failed: {e}"));
            let mut rendered = String::new();
            for step in &def.steps {
                rendered.push_str(&format!("=== step {} ===\n", step.id));
                rendered.push_str(&render_prompt(&step.prompt, "SNAPSHOT_TOPIC"));
                rendered.push('\n');
            }
            assert_eq!(
                rendered, baseline,
                "rendered prompts for '{name}' drifted from the wai-vsn6 baseline"
            );
        }
    }

    #[test]
    fn block_ref_expands_from_blocks_table() {
        let toml = r#"
[pipeline]
name = "blocks"

[[steps]]
id = "step1"
prompt = """
Do the thing.
{{block:trail}}
"""

[[blocks]]
name = "trail"
content = "Advance: `wai pipeline next`"
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should load with block ref");
        assert_eq!(
            def.steps[0].prompt, "Do the thing.\nAdvance: `wai pipeline next`\n",
            "block ref must expand to the block content"
        );
    }

    #[test]
    fn block_content_participates_in_topic_rendering() {
        let toml = r#"
[pipeline]
name = "blocks"

[[steps]]
id = "step1"
prompt = "{{block:line}}\n"

[[blocks]]
name = "line"
content = "Work on {topic} now."
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should load");
        assert_eq!(
            render_prompt(&def.steps[0].prompt, "auth"),
            "Work on auth now.\n"
        );
    }

    #[test]
    fn load_pipeline_toml_rejects_unknown_block() {
        let toml = r#"
[pipeline]
name = "blocks"

[[steps]]
id = "step1"
prompt = "{{block:does-not-exist}}\n"
"#;
        let f = write_toml(toml);
        let err = load_pipeline_toml(f.path()).expect_err("unknown block must fail");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("does-not-exist"),
            "error must name the missing block: {msg}"
        );
        assert!(
            msg.contains("unknown block"),
            "error must say 'unknown block': {msg}"
        );
    }

    #[test]
    fn load_pipeline_toml_rejects_malformed_block_ref() {
        let toml = r#"
[pipeline]
name = "blocks"

[[steps]]
id = "step1"
prompt = "{{block:no-closing-braces\n"
"#;
        let f = write_toml(toml);
        let err = load_pipeline_toml(f.path()).expect_err("malformed ref must fail");
        let msg = format!("{err:?}").to_lowercase();
        assert!(
            msg.contains("malformed"),
            "error must flag malformed reference: {msg}"
        );
    }

    #[test]
    fn load_pipeline_toml_rejects_duplicate_block_names() {
        let toml = r#"
[pipeline]
name = "blocks"

[[steps]]
id = "step1"
prompt = "{{block:trail}}\n"

[[blocks]]
name = "trail"
content = "A"

[[blocks]]
name = "trail"
content = "B"
"#;
        let f = write_toml(toml);
        let err = load_pipeline_toml(f.path()).expect_err("duplicate block names must fail");
        assert!(
            format!("{err:?}").contains("duplicate block"),
            "error must mention duplicate block"
        );
    }

    #[test]
    fn block_refs_resolve_recursively() {
        let toml = r#"
[pipeline]
name = "blocks"

[[steps]]
id = "step1"
prompt = "{{block:outer}}\n"

[[blocks]]
name = "outer"
content = "[{{block:inner}}]"

[[blocks]]
name = "inner"
content = "core"
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("nested refs should resolve");
        assert_eq!(def.steps[0].prompt, "[core]\n");
    }

    #[test]
    fn load_pipeline_toml_rejects_cyclic_block_refs() {
        let toml = r#"
[pipeline]
name = "blocks"

[[steps]]
id = "step1"
prompt = "{{block:a}}\n"

[[blocks]]
name = "a"
content = "{{block:b}}"

[[blocks]]
name = "b"
content = "{{block:a}}"
"#;
        let f = write_toml(toml);
        let err = load_pipeline_toml(f.path()).expect_err("cyclic refs must fail");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("cyclic") || msg.contains("deeply nested"),
            "error must flag cycle: {msg}"
        );
    }

    #[test]
    fn prompt_without_block_refs_is_unchanged() {
        let toml = r#"
[pipeline]
name = "blocks"

[[steps]]
id = "step1"
prompt = "Plain prompt with {topic} only.\n"
"#;
        let f = write_toml(toml);
        let def = load_pipeline_toml(f.path()).expect("should load");
        assert_eq!(def.steps[0].prompt, "Plain prompt with {topic} only.\n");
    }

    #[test]
    fn resolve_block_refs_direct_expansion_and_errors() {
        let pairs = vec![
            SharedBlock {
                name: "trail".to_string(),
                content: "Advance".to_string(),
            },
            SharedBlock {
                name: "trail".to_string(),
                content: "Duplicate".to_string(),
            },
        ];
        let err = build_block_map(&pairs).expect_err("duplicate names must fail");
        assert!(format!("{err:?}").contains("duplicate block"));

        let mut blocks: HashMap<String, String> = HashMap::new();
        blocks.insert("trail".to_string(), "Advance".to_string());
        assert_eq!(
            resolve_block_refs("{{block:trail}}", "t", &blocks).expect("resolves"),
            "Advance"
        );
        let err = resolve_block_refs("{{block:missing}}", "t", &blocks).expect_err("unknown");
        assert!(format!("{err:?}").contains("unknown block"));
        let err = resolve_block_refs("{{block:trail", "t", &blocks).expect_err("malformed");
        assert!(format!("{err:?}").to_lowercase().contains("malformed"));
    }
}
