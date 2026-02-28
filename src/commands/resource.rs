use cliclack::log;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::cli::{
    ResourceAddCommands, ResourceExportArgs, ResourceImportCommands, ResourceInstallArgs,
    ResourceListCommands,
};
use crate::config::{SKILLS_DIR, agent_config_dir, global_skills_dir};
use crate::context::{current_context, require_safe_mode};
use crate::error::WaiError;

use super::require_project;

/// Validates a skill name according to the following rules:
/// - Only lowercase a-z, digits 0-9, hyphens, and at most one '/' allowed
/// - '/' separates category from action (e.g. "issue/gather")
/// - Neither segment may be empty, start/end with a hyphen, or contain invalid characters
/// - No leading, trailing, or consecutive hyphens within each segment
/// - Maximum 64 characters (total, including '/')
pub fn validate_skill_name(name: &str) -> Result<(), WaiError> {
    // Check for empty string
    if name.is_empty() {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot be empty".to_string(),
        });
    }

    // At most one '/' allowed
    let slash_count = name.chars().filter(|&c| c == '/').count();
    if slash_count > 1 {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name can contain at most one '/' separator (e.g. category/action)"
                .to_string(),
        });
    }

    // Check length
    if name.len() > 64 {
        return Err(WaiError::InvalidSkillName {
            message: format!("Skill name too long ({} chars, max 64)", name.len()),
        });
    }

    // Validate each segment individually
    for segment in name.splitn(2, '/') {
        validate_skill_name_segment(segment)?;
    }

    Ok(())
}

/// Validates a single segment of a skill name (the part before or after '/').
fn validate_skill_name_segment(segment: &str) -> Result<(), WaiError> {
    if segment.is_empty() {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name segments cannot be empty (check for leading or trailing '/')"
                .to_string(),
        });
    }

    if segment == "." || segment == ".." {
        return Err(WaiError::InvalidSkillName {
            message: format!("'{}' is not a valid skill name segment", segment),
        });
    }

    if segment.starts_with('.') {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot start with '.'".to_string(),
        });
    }

    if segment.starts_with('-') {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot start with a hyphen".to_string(),
        });
    }

    if segment.ends_with('-') {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot end with a hyphen".to_string(),
        });
    }

    if segment.contains("--") {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot contain consecutive hyphens".to_string(),
        });
    }

    for (idx, ch) in segment.chars().enumerate() {
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '-' {
            return Err(WaiError::InvalidSkillName {
                message: format!(
                    "Invalid character '{}' at position {}. Only lowercase letters, digits, and hyphens allowed",
                    ch, idx
                ),
            });
        }
    }

    Ok(())
}

/// Metadata extracted from a SKILL.md frontmatter
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// Source of a listed skill
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum SkillSource {
    Local,
    Global,
}

/// Skill entry for listing
#[derive(Debug, Clone, Serialize)]
struct SkillEntry {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    description: String,
    path: String,
    source: SkillSource,
}

/// JSON payload for skill list
#[derive(Debug, Serialize)]
struct SkillListPayload {
    skills: Vec<SkillEntry>,
}

/// Parses YAML frontmatter from a SKILL.md file.
///
/// Expects frontmatter between opening and closing `---` delimiters.
/// Returns `None` if:
/// - File cannot be read
/// - No frontmatter delimiters found
/// - YAML is malformed
/// - Required fields (`name` or `description`) are missing
///
/// Never panics - all errors are handled gracefully by returning `None`.
pub fn parse_skill_frontmatter(path: &Path) -> Option<SkillMetadata> {
    // Read file contents
    let contents = fs::read_to_string(path).ok()?;

    // Find frontmatter delimiters
    let mut lines = contents.lines();

    // Check if first line is opening delimiter
    let first_line = lines.next()?.trim();
    if first_line != "---" {
        return None;
    }

    // Collect lines until closing delimiter
    let mut frontmatter_lines = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        frontmatter_lines.push(line);
    }

    // If no lines were collected, frontmatter is empty
    if frontmatter_lines.is_empty() {
        return None;
    }

    // Join lines and parse YAML
    let yaml_content = frontmatter_lines.join("\n");
    let metadata: SkillMetadata = serde_yml::from_str(&yaml_content).ok()?;

    // Validate that required fields are not empty
    if metadata.name.trim().is_empty() || metadata.description.trim().is_empty() {
        return None;
    }

    Some(metadata)
}

pub fn run_add(cmd: ResourceAddCommands) -> Result<()> {
    match cmd {
        ResourceAddCommands::Skill { name, template } => add_skill(&name, template.as_deref()),
    }
}

pub fn run_list(cmd: ResourceListCommands) -> Result<()> {
    match cmd {
        ResourceListCommands::Skills { json } => list_skills(json),
    }
}

pub fn run_import(cmd: ResourceImportCommands) -> Result<()> {
    match cmd {
        ResourceImportCommands::Skills { from } => import_skills(from),
        ResourceImportCommands::Archive { file, yes } => import_skills_archive(&file, yes),
    }
}

pub fn run_install(args: ResourceInstallArgs) -> Result<()> {
    if args.global {
        install_skill_global(&args.skill)
    } else if let Some(repo_path) = args.from_repo {
        install_skill_from_repo(&args.skill, &repo_path)
    } else {
        miette::bail!(
            "Specify either --global (to install globally) or --from-repo <path> (to install from another repository)"
        )
    }
}

pub fn run_export(args: ResourceExportArgs) -> Result<()> {
    export_skills(&args.skills, &args.output)
}

fn add_skill(name: &str, template_name: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("add skill")?;

    // Validate skill name
    validate_skill_name(name)?;

    // Resolve template body
    let body = match template_name {
        Some(t) => skill_template_body(t)?,
        None => "Instructions go here.\n".to_string(),
    };

    // Build path to skills directory
    let skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    // Path::join handles the '/' in hierarchical names correctly on all platforms
    let skill_dir = skills_dir.join(name);

    // Check if skill already exists
    if skill_dir.exists() {
        miette::bail!("Skill '{}' already exists at {}", name, skill_dir.display());
    }

    // Conflict detection for hierarchical names
    if name.contains('/') {
        // Hierarchical: check that the category segment isn't already a flat skill
        let category = name.split('/').next().unwrap();
        let category_dir = skills_dir.join(category);
        if category_dir.join("SKILL.md").exists() {
            miette::bail!(
                "Cannot create '{}': '{}' already exists as a flat skill",
                name,
                category
            );
        }
    } else {
        // Flat: check that the name isn't already used as a category directory
        let candidate = skills_dir.join(name);
        if candidate.is_dir() && !candidate.join("SKILL.md").exists() {
            miette::bail!(
                "Cannot create '{}': it is already used as a category. Use a hierarchical name like '{}/...' instead.",
                name,
                name
            );
        }
    }

    // Create skill directory (create_dir_all handles intermediate category dirs)
    fs::create_dir_all(&skill_dir).into_diagnostic()?;

    // Create SKILL.md with frontmatter + body
    let skill_file = skill_dir.join("SKILL.md");
    let title = kebab_to_title_case(name);
    let content = format!(
        "---\nname: {}\ndescription: \"\"\n---\n\n# {}\n\n{}",
        name, title, body
    );

    fs::write(&skill_file, content).into_diagnostic()?;

    log::success(format!("Created skill '{}'", name)).into_diagnostic()?;
    log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;

    Ok(())
}

const VALID_TEMPLATES: &[&str] = &["gather", "create", "tdd", "rule-of-5"];

fn skill_template_body(name: &str) -> Result<String> {
    match name {
        "gather" => Ok(r#"## Instructions

Use this skill to research $ARGUMENTS in the $PROJECT project.
Repository root: $REPO_ROOT

1. Search existing artifacts:
   ```
   wai search "$ARGUMENTS"
   ```
2. Explore the codebase at $REPO_ROOT relevant to $ARGUMENTS.
3. Gather findings and record them:
   ```
   wai add research "findings about $ARGUMENTS"
   ```
4. Summary: list key facts, open questions, and recommended next steps.
"#
        .to_string()),

        "create" => Ok(r#"## Instructions

Use this skill to create items for $ARGUMENTS in the $PROJECT project.
Repository root: $REPO_ROOT

1. Retrieve the latest plan:
   ```
   wai search "$ARGUMENTS" --type plan --latest
   ```
2. For each item in the plan:
   - Create a tracking issue:
     ```
     bd create --title="..." --type=task
     ```
   - Wire dependencies with `bd dep add <blocked> <blocker>` as needed.
3. Confirm all items are created and visible with `bd ready`.
"#
        .to_string()),

        "tdd" => Ok(r#"## Instructions

Use this skill to implement $ARGUMENTS in the $PROJECT project using TDD.
Repository root: $REPO_ROOT

### RED
1. Write a failing test for the next behaviour.
2. Run tests — confirm the new test fails:
   ```
   cd $REPO_ROOT && cargo test
   ```

### GREEN
3. Write the minimum code to make the test pass.
4. Run tests — confirm all tests pass.

### REFACTOR
5. Clean up without changing behaviour.
6. Run tests — confirm all tests still pass.

Commit after each GREEN/REFACTOR cycle:
```
git add <files>
git commit -m "..."
```
"#
        .to_string()),

        "rule-of-5" => Ok(r#"## Instructions

Use this skill to review $ARGUMENTS in the $PROJECT project using 5 passes.
Repository root: $REPO_ROOT

Run 5 independent review passes. After each pass, record findings.
Check for convergence: if passes 4 and 5 agree, stop early.

### Pass structure
- What is the purpose of this component?
- Does the implementation match the intent?
- Are there edge cases, error paths, or missing tests?
- Is the interface clear and consistent?

### Verdict

After all passes, output one of:
- **APPROVED** — ready to merge/deploy
- **NEEDS_CHANGES** — list specific required changes
- **NEEDS_HUMAN** — ambiguous or high-risk; escalate to a human reviewer
"#
        .to_string()),

        other => {
            miette::bail!(
                "Unknown template '{}'. Valid templates: {}",
                other,
                VALID_TEMPLATES.join(", ")
            )
        }
    }
}

/// Converts a skill name to Title Case for use in templates.
/// Handles both flat ("my-cool-skill" -> "My Cool Skill") and
/// hierarchical ("issue/gather" -> "Issue / Gather") names.
fn kebab_to_title_case(s: &str) -> String {
    let title_segment = |seg: &str| -> String {
        seg.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    };

    s.splitn(2, '/')
        .map(title_segment)
        .collect::<Vec<_>>()
        .join(" / ")
}

fn list_skills(json: bool) -> Result<()> {
    // Merge local --json with global --json so both
    // `wai resource list skills --json` and `wai --json resource list skills` work.
    let json = json || current_context().json;
    let project_root = require_project()?;
    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    let global_dir = global_skills_dir();

    // Collect local skills (keyed by canonical name for deduplication)
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut entries = Vec::new();

    if local_skills_dir.exists() {
        scan_skills_dir(
            &local_skills_dir,
            &local_skills_dir,
            SkillSource::Local,
            &mut entries,
        );
    }
    // Track names from local skills to suppress global duplicates
    for e in &entries {
        seen_names.insert(e.name.clone());
    }

    // Also collect global skills that aren't shadowed by local ones
    if global_dir.exists() {
        let mut global_entries = Vec::new();
        scan_skills_dir(
            &global_dir,
            &global_dir,
            SkillSource::Global,
            &mut global_entries,
        );
        for e in global_entries {
            if !seen_names.contains(&e.name) {
                entries.push(e);
            }
        }
    }

    // Sort alphabetically by name
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    // Output
    if json {
        let payload = SkillListPayload { skills: entries };
        crate::output::print_json(&payload)?;
    } else if entries.is_empty() {
        println!();
        println!("  {} No skills found", "○".dimmed());
        println!(
            "  {} Run 'wai resource add skill <name>' to create one",
            "→".cyan()
        );
        println!();
    } else {
        println!();
        println!("  {} Skills", "◆".cyan());
        println!();
        for entry in entries {
            let desc = if entry.description.len() > 60 {
                format!("{}...", &entry.description[..57])
            } else {
                entry.description.clone()
            };
            let global_tag = if entry.source == SkillSource::Global {
                " [global]".dimmed().to_string()
            } else {
                String::new()
            };

            if entry.description == "(no metadata)" {
                println!(
                    "    {} {}{}  {}",
                    "•".dimmed(),
                    entry.name,
                    global_tag,
                    desc.dimmed()
                );
            } else {
                println!(
                    "    {} {}{}  {}",
                    "•".dimmed(),
                    entry.name.bold(),
                    global_tag,
                    desc.dimmed()
                );
            }
        }
        println!();
    }

    Ok(())
}

/// Scan a skills directory and collect SkillEntry records.
///
/// Handles both flat skills (skills_dir/<name>/SKILL.md) and hierarchical
/// skills (skills_dir/<category>/<name>/SKILL.md).
fn scan_skills_dir(
    skills_dir: &Path,
    display_root: &Path,
    source: SkillSource,
    entries: &mut Vec<SkillEntry>,
) {
    let Ok(read_dir) = fs::read_dir(skills_dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let skill_file = path.join("SKILL.md");

        if skill_file.exists() {
            // Flat skill
            let relative_path = path
                .strip_prefix(display_root)
                .unwrap_or(&path)
                .display()
                .to_string();
            if let Some(metadata) = parse_skill_frontmatter(&skill_file) {
                entries.push(SkillEntry {
                    name: metadata.name,
                    category: None,
                    description: metadata.description,
                    path: relative_path,
                    source: source.clone(),
                });
            } else {
                entries.push(SkillEntry {
                    name: dir_name,
                    category: None,
                    description: "(no metadata)".to_string(),
                    path: relative_path,
                    source: source.clone(),
                });
            }
        } else {
            // Category directory: recurse one level
            let Ok(sub_read) = fs::read_dir(&path) else {
                continue;
            };
            for sub_entry in sub_read.flatten() {
                let sub_path = sub_entry.path();
                if !sub_path.is_dir() {
                    continue;
                }

                let sub_name = sub_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let sub_skill_file = sub_path.join("SKILL.md");
                let hierarchical_name = format!("{}/{}", dir_name, sub_name);
                let relative_path = sub_path
                    .strip_prefix(display_root)
                    .unwrap_or(&sub_path)
                    .display()
                    .to_string();

                if let Some(metadata) = parse_skill_frontmatter(&sub_skill_file) {
                    entries.push(SkillEntry {
                        name: metadata.name,
                        category: Some(dir_name.clone()),
                        description: metadata.description,
                        path: relative_path,
                        source: source.clone(),
                    });
                } else {
                    entries.push(SkillEntry {
                        name: hierarchical_name,
                        category: Some(dir_name.clone()),
                        description: "(no metadata)".to_string(),
                        path: relative_path,
                        source: source.clone(),
                    });
                }
            }
        }
    }
}

fn import_skills(from: Option<String>) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("import skills")?;

    // Determine source path
    let source_path = if let Some(path) = from {
        Path::new(&path).to_path_buf()
    } else {
        project_root.join(".agents").join("skills")
    };

    // Check if source exists
    if !source_path.exists() {
        miette::bail!(
            "Source path not found: {}\n  Specify a different path with --from <path>",
            source_path.display()
        );
    }

    if !source_path.is_dir() {
        miette::bail!("Source path is not a directory: {}", source_path.display());
    }

    let target_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    fs::create_dir_all(&target_dir).into_diagnostic()?;

    let mut imported = 0;
    let mut skipped = 0;

    // Scan source directory
    for entry in fs::read_dir(&source_path).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let source_skill_dir = entry.path();

        // Only process directories
        if !source_skill_dir.is_dir() {
            continue;
        }

        // Check if SKILL.md exists
        let skill_md = source_skill_dir.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let skill_name = source_skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| miette::miette!("Invalid skill directory name"))?;

        let target_skill_dir = target_dir.join(skill_name);

        // Skip if target already exists
        if target_skill_dir.exists() {
            log::warning(format!("Skipped '{}' (already exists)", skill_name)).into_diagnostic()?;
            skipped += 1;
            continue;
        }

        // Copy entire directory tree
        copy_dir_all(&source_skill_dir, &target_skill_dir).into_diagnostic()?;
        imported += 1;
    }

    // Report results
    if imported == 0 && skipped == 0 {
        println!();
        println!(
            "  {} No skills found in {}",
            "○".dimmed(),
            source_path.display()
        );
        println!();
    } else {
        println!();
        if imported > 0 {
            log::success(format!(
                "Imported {} skill{}",
                imported,
                if imported == 1 { "" } else { "s" }
            ))
            .into_diagnostic()?;
        }
        if skipped > 0 {
            log::info(format!(
                "Skipped {} skill{} (already exist{})",
                skipped,
                if skipped == 1 { "" } else { "s" },
                if skipped == 1 { "s" } else { "" }
            ))
            .into_diagnostic()?;
        }
        log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;
        println!();
    }

    Ok(())
}

/// Recursively copy a directory and all its contents
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Resolve the SKILL.md path for a named skill within a skills directory.
///
/// Handles both flat (`<skills_dir>/<name>/SKILL.md`) and hierarchical
/// (`<skills_dir>/<cat>/<action>/SKILL.md`) layouts.
fn skill_md_path(skills_dir: &Path, name: &str) -> PathBuf {
    skills_dir.join(name).join("SKILL.md")
}

/// Warn if a skill file contains content that looks like hardcoded project names
/// or absolute paths (should use $PROJECT, $REPO_ROOT, $ARGUMENTS instead).
fn warn_hardcoded_content(skill_path: &Path, project_name: &str, project_root: &Path) {
    let Ok(content) = fs::read_to_string(skill_path) else {
        return;
    };
    let root_str = project_root.to_string_lossy();
    let mut warnings: Vec<String> = Vec::new();

    // Check for the literal project name (if non-empty)
    if !project_name.is_empty() && content.contains(project_name) {
        warnings.push(format!("contains project name '{}'", project_name));
    }
    // Check for absolute paths starting with the repo root
    if root_str.len() > 1 && content.contains(root_str.as_ref()) {
        warnings.push(format!("contains absolute path '{}'", root_str));
    }

    if !warnings.is_empty() {
        let _ = log::warning(format!(
            "Skill may have hardcoded content ({}). Consider using $PROJECT, $REPO_ROOT, $ARGUMENTS instead.",
            warnings.join("; ")
        ));
    }
}

/// Install a skill from the current project into the global skills library
/// at `~/.wai/resources/skills/`.
fn install_skill_global(name: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("install skill globally")?;

    validate_skill_name(name)?;

    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    let src_skill_dir = local_skills_dir.join(name);
    let src_skill_md = src_skill_dir.join("SKILL.md");

    if !src_skill_md.exists() {
        miette::bail!(
            "Skill '{}' not found in current project (looked in {})",
            name,
            src_skill_dir.display()
        );
    }

    let global_dir = global_skills_dir();
    let dst_skill_dir = global_dir.join(name);

    if dst_skill_dir.exists() {
        miette::bail!(
            "Skill '{}' already exists in global library at {}",
            name,
            dst_skill_dir.display()
        );
    }

    // Warn about hardcoded content
    let project_name = crate::config::ProjectConfig::load(&project_root)
        .map(|c| c.project.name)
        .unwrap_or_default();
    warn_hardcoded_content(&src_skill_md, &project_name, &project_root);

    fs::create_dir_all(&dst_skill_dir).into_diagnostic()?;
    copy_dir_all(&src_skill_dir, &dst_skill_dir).into_diagnostic()?;

    log::success(format!(
        "Installed '{}' globally to {}",
        name,
        dst_skill_dir.display()
    ))
    .into_diagnostic()?;

    Ok(())
}

/// Install a skill from another repository's `.wai` directory into the current project.
fn install_skill_from_repo(name: &str, repo_path: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("install skill from repo")?;

    validate_skill_name(name)?;

    let repo = PathBuf::from(repo_path);
    let src_skills_dir = crate::config::agent_config_dir(&repo).join(SKILLS_DIR);
    let src_skill_dir = src_skills_dir.join(name);
    let src_skill_md = src_skill_dir.join("SKILL.md");

    if !src_skill_md.exists() {
        miette::bail!(
            "Skill '{}' not found in '{}' (looked in {})",
            name,
            repo_path,
            src_skill_dir.display()
        );
    }

    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    let dst_skill_dir = local_skills_dir.join(name);

    if dst_skill_dir.exists() {
        miette::bail!("Skill '{}' already exists in current project", name);
    }

    // Warn about hardcoded content (use current project's name as the suspect)
    let project_name = crate::config::ProjectConfig::load(&project_root)
        .map(|c| c.project.name)
        .unwrap_or_default();
    warn_hardcoded_content(&src_skill_md, &project_name, &project_root);

    fs::create_dir_all(&dst_skill_dir).into_diagnostic()?;
    copy_dir_all(&src_skill_dir, &dst_skill_dir).into_diagnostic()?;

    log::success(format!(
        "Installed '{}' from '{}' into current project ({})",
        name,
        repo_path,
        dst_skill_dir.display()
    ))
    .into_diagnostic()?;
    log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;

    Ok(())
}

/// Export named skills to a tar.gz archive.
fn export_skills(skill_names: &[String], output_path: &str) -> Result<()> {
    let project_root = require_project()?;

    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    let global_dir = global_skills_dir();

    let out_file = fs::File::create(output_path).into_diagnostic()?;
    let gz = GzEncoder::new(out_file, Compression::default());
    let mut archive = tar::Builder::new(gz);

    for name in skill_names {
        validate_skill_name(name)?;

        // Find the skill (local first, then global)
        let local_path = skill_md_path(&local_skills_dir, name);
        let global_path = skill_md_path(&global_dir, name);

        let skill_md_file = if local_path.exists() {
            local_path
        } else if global_path.exists() {
            global_path
        } else {
            miette::bail!(
                "Skill '{}' not found in local project or global library",
                name
            );
        };

        // Archive path: preserve <name>/SKILL.md or <cat>/<action>/SKILL.md structure
        let archive_path = format!("{}/SKILL.md", name);

        archive
            .append_path_with_name(&skill_md_file, &archive_path)
            .into_diagnostic()?;
    }

    archive.finish().into_diagnostic()?;
    let gz_encoder = archive.into_inner().into_diagnostic()?;
    gz_encoder.finish().into_diagnostic()?;

    log::success(format!(
        "Exported {} skill{} to '{}'",
        skill_names.len(),
        if skill_names.len() == 1 { "" } else { "s" },
        output_path
    ))
    .into_diagnostic()?;

    Ok(())
}

/// Validate an archive entry path.
///
/// Valid patterns:
///   `<name>/SKILL.md`           (flat skill)
///   `<cat>/<action>/SKILL.md`   (hierarchical skill)
///
/// Rejects path traversal (components starting with `..` or absolute paths).
fn validate_archive_entry_path(path_str: &str) -> Result<()> {
    // Must end with /SKILL.md
    if !path_str.ends_with("/SKILL.md") {
        miette::bail!(
            "Invalid archive entry '{}': entries must end with '/SKILL.md'",
            path_str
        );
    }

    let components: Vec<&str> = path_str.split('/').collect();
    // Valid: ["name", "SKILL.md"] or ["cat", "action", "SKILL.md"]
    let depth = components.len();
    if !(2..=3).contains(&depth) {
        miette::bail!(
            "Invalid archive entry '{}': expected 'name/SKILL.md' or 'category/action/SKILL.md'",
            path_str
        );
    }

    // Check for path traversal in each component
    for component in &components {
        if *component == ".." || *component == "." {
            miette::bail!(
                "Invalid archive entry '{}': path traversal detected",
                path_str
            );
        }
        if component.starts_with('/') {
            miette::bail!(
                "Invalid archive entry '{}': absolute paths not allowed",
                path_str
            );
        }
    }

    // Validate skill name components (everything except the final SKILL.md)
    let name_part = components[..depth - 1].join("/");
    validate_skill_name(&name_part)?;

    Ok(())
}

/// Import skills from a tar.gz archive into the current project.
fn import_skills_archive(archive_path: &str, yes: bool) -> Result<()> {
    // Merge local --yes with global --yes so both
    // `wai resource import archive f --yes` and `wai --yes resource import archive f` work.
    let yes = yes || current_context().yes;
    let project_root = require_project()?;
    require_safe_mode("import skills from archive")?;

    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    fs::create_dir_all(&local_skills_dir).into_diagnostic()?;

    import_archive_into_dir(archive_path, &local_skills_dir, yes)
}

/// Extract validated skill entries from a tar.gz archive.
///
/// Returns a list of `(archive_path, content)` pairs. Validates all paths
/// before returning — callers can trust the results are safe to write.
pub(crate) fn read_archive_entries(archive_path: &str) -> Result<Vec<(String, Vec<u8>)>> {
    let archive_file = fs::File::open(archive_path).into_diagnostic()?;
    let gz = GzDecoder::new(BufReader::new(archive_file));
    let mut archive = tar::Archive::new(gz);

    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    for entry in archive.entries().into_diagnostic()? {
        let mut entry = entry.into_diagnostic()?;
        let entry_path = entry.path().into_diagnostic()?;
        let path_str = entry_path.to_string_lossy().to_string();

        if entry.header().entry_type().is_dir() {
            continue;
        }

        validate_archive_entry_path(&path_str)?;

        let mut content = Vec::new();
        std::io::copy(&mut entry, &mut content).into_diagnostic()?;
        entries.push((path_str, content));
    }
    Ok(entries)
}

/// Write validated archive entries into a skills directory.
///
/// With `yes=true` overwrites without prompting; otherwise prompts per skill.
/// Returns `(imported, overwritten, skipped)` counts.
pub(crate) fn write_archive_entries(
    entries: Vec<(String, Vec<u8>)>,
    skills_dir: &Path,
    yes: bool,
) -> Result<(usize, usize, usize)> {
    let mut imported = 0;
    let mut overwritten = 0;
    let mut skipped = 0;

    for (path_str, content) in entries {
        let skill_name = path_str.trim_end_matches("/SKILL.md");
        let dst_dir = skills_dir.join(skill_name);
        let dst_file = dst_dir.join("SKILL.md");

        if dst_file.exists() {
            if yes {
                fs::create_dir_all(&dst_dir).into_diagnostic()?;
                fs::write(&dst_file, &content).into_diagnostic()?;
                overwritten += 1;
            } else {
                let confirmed =
                    cliclack::confirm(format!("Skill '{}' already exists. Overwrite?", skill_name))
                        .interact()
                        .into_diagnostic()?;

                if confirmed {
                    fs::create_dir_all(&dst_dir).into_diagnostic()?;
                    fs::write(&dst_file, &content).into_diagnostic()?;
                    overwritten += 1;
                } else {
                    skipped += 1;
                }
            }
        } else {
            fs::create_dir_all(&dst_dir).into_diagnostic()?;
            fs::write(&dst_file, &content).into_diagnostic()?;
            imported += 1;
        }
    }
    Ok((imported, overwritten, skipped))
}

/// Core implementation: import from archive into a given skills directory.
pub(crate) fn import_archive_into_dir(
    archive_path: &str,
    skills_dir: &Path,
    yes: bool,
) -> Result<()> {
    let entries = read_archive_entries(archive_path)?;

    if entries.is_empty() {
        println!();
        println!("  {} No skills found in archive", "○".dimmed());
        println!();
        return Ok(());
    }

    let (imported, overwritten, skipped) = write_archive_entries(entries, skills_dir, yes)?;

    println!();
    if imported > 0 {
        log::success(format!(
            "Imported {} new skill{}",
            imported,
            if imported == 1 { "" } else { "s" }
        ))
        .into_diagnostic()?;
    }
    if overwritten > 0 {
        log::success(format!(
            "Overwrote {} existing skill{}",
            overwritten,
            if overwritten == 1 { "" } else { "s" }
        ))
        .into_diagnostic()?;
    }
    if skipped > 0 {
        log::info(format!(
            "Skipped {} skill{} (kept existing)",
            skipped,
            if skipped == 1 { "" } else { "s" }
        ))
        .into_diagnostic()?;
    }
    if imported > 0 || overwritten > 0 {
        log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_skill_names() {
        // Valid flat names should pass
        assert!(validate_skill_name("my-skill").is_ok());
        assert!(validate_skill_name("skill123").is_ok());
        assert!(validate_skill_name("a").is_ok());
        assert!(validate_skill_name("my-cool-skill-2").is_ok());
        assert!(validate_skill_name("abc-123-xyz").is_ok());
        // Valid hierarchical names
        assert!(validate_skill_name("issue/gather").is_ok());
        assert!(validate_skill_name("impl/run").is_ok());
        assert!(validate_skill_name("my-cat/my-action").is_ok());
    }

    #[test]
    fn test_hierarchical_invalid() {
        // Two slashes: invalid
        assert!(validate_skill_name("a/b/c").is_err());
        // Leading slash: empty first segment
        assert!(validate_skill_name("/gather").is_err());
        // Trailing slash: empty second segment
        assert!(validate_skill_name("issue/").is_err());
        // Hyphen rules apply to segments too
        assert!(validate_skill_name("issue/-gather").is_err());
        assert!(validate_skill_name("issue/gather-").is_err());
        assert!(validate_skill_name("-cat/gather").is_err());
    }

    #[test]
    fn test_empty_name() {
        let result = validate_skill_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_special_names() {
        assert!(validate_skill_name(".").is_err());
        assert!(validate_skill_name("..").is_err());
    }

    #[test]
    fn test_starts_with_dot() {
        let result = validate_skill_name(".hidden");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot start with '.'")
        );
    }

    #[test]
    fn test_max_length() {
        // 64 characters should be valid
        let valid_64 = "a".repeat(64);
        assert!(validate_skill_name(&valid_64).is_ok());

        // 65 characters should fail
        let invalid_65 = "a".repeat(65);
        let result = validate_skill_name(&invalid_65);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_leading_hyphen() {
        let result = validate_skill_name("-skill");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot start with a hyphen")
        );
    }

    #[test]
    fn test_trailing_hyphen() {
        let result = validate_skill_name("skill-");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot end with a hyphen")
        );
    }

    #[test]
    fn test_consecutive_hyphens() {
        let result = validate_skill_name("my--skill");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("consecutive hyphens")
        );
    }

    #[test]
    fn test_invalid_characters() {
        // Uppercase
        let result = validate_skill_name("MySkill");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid character")
        );

        // Underscore
        let result = validate_skill_name("my_skill");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid character")
        );

        // Space
        let result = validate_skill_name("my skill");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid character")
        );

        // Special characters
        let result = validate_skill_name("my@skill");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid character")
        );
    }

    // Frontmatter parser tests
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_parse_valid_frontmatter() {
        let content = r#"---
name: my-skill
description: A test skill
---
# Content here
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_some());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "my-skill");
        assert_eq!(metadata.description, "A test skill");
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a regular markdown file\nNo frontmatter here";
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_missing_opening_delimiter() {
        let content = r#"name: my-skill
description: A test skill
---
# Content
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_missing_closing_delimiter() {
        let content = r#"---
name: my-skill
description: A test skill
# Content without closing delimiter
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        // Should still parse if we reach EOF
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_malformed_yaml() {
        let content = r#"---
name: my-skill
description: [invalid yaml structure
  missing closing bracket
---
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_missing_name_field() {
        let content = r#"---
description: A test skill
---
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_missing_description_field() {
        let content = r#"---
name: my-skill
---
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_name() {
        let content = r#"---
name: ""
description: A test skill
---
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_description() {
        let content = r#"---
name: my-skill
description: ""
---
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_whitespace_only_fields() {
        let content = r#"---
name: "   "
description: "  "
---
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_skill_frontmatter(Path::new("/nonexistent/path/to/file.md"));
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_frontmatter() {
        let content = r#"---
---
# Content
"#;
        let file = create_temp_file(content);
        let result = parse_skill_frontmatter(file.path());

        assert!(result.is_none());
    }

    // Title case conversion tests
    #[test]
    fn test_kebab_to_title_case() {
        assert_eq!(kebab_to_title_case("my-skill"), "My Skill");
        assert_eq!(kebab_to_title_case("single"), "Single");
        assert_eq!(kebab_to_title_case("a-b-c"), "A B C");
        assert_eq!(kebab_to_title_case("my-cool-skill-2"), "My Cool Skill 2");
        assert_eq!(kebab_to_title_case(""), "");
        // Hierarchical names
        assert_eq!(kebab_to_title_case("issue/gather"), "Issue / Gather");
        assert_eq!(
            kebab_to_title_case("my-cat/my-action"),
            "My Cat / My Action"
        );
    }

    // ── archive path validation ──────────────────────────────────────────────

    #[test]
    fn test_valid_archive_paths() {
        assert!(validate_archive_entry_path("my-skill/SKILL.md").is_ok());
        assert!(validate_archive_entry_path("issue/gather/SKILL.md").is_ok());
        assert!(validate_archive_entry_path("a/SKILL.md").is_ok());
    }

    #[test]
    fn test_archive_path_traversal_rejected() {
        let result = validate_archive_entry_path("../etc/passwd");
        assert!(result.is_err());
        let result = validate_archive_entry_path("../bad/SKILL.md");
        assert!(result.is_err());
        let result = validate_archive_entry_path("good/../SKILL.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_path_must_end_with_skill_md() {
        let result = validate_archive_entry_path("my-skill/README.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SKILL.md"));
    }

    #[test]
    fn test_archive_path_depth_too_deep() {
        let result = validate_archive_entry_path("a/b/c/SKILL.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_path_absolute() {
        // An absolute path component should be rejected via skill name validation
        let result = validate_archive_entry_path("/etc/SKILL.md");
        assert!(result.is_err());
    }

    // ── export → import round-trip ───────────────────────────────────────────

    fn make_skill_dir(dir: &Path, name: &str, content: &str) {
        // name may be hierarchical (cat/action)
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn test_export_import_round_trip() {
        let src_skills = tempfile::tempdir().unwrap();
        let dst_skills = tempfile::tempdir().unwrap();
        let archive_dir = tempfile::tempdir().unwrap();

        let content = "---\nname: gather\ndescription: Gather skill\n---\n\nInstructions.\n";
        make_skill_dir(src_skills.path(), "gather", content);

        // Export
        let archive_path = archive_dir.path().join("skills.tar.gz");
        let archive_path_str = archive_path.to_string_lossy().to_string();

        let out_file = fs::File::create(&archive_path).unwrap();
        let gz = GzEncoder::new(out_file, Compression::default());
        let mut builder = tar::Builder::new(gz);
        let skill_md = src_skills.path().join("gather").join("SKILL.md");
        builder
            .append_path_with_name(&skill_md, "gather/SKILL.md")
            .unwrap();
        builder.finish().unwrap();
        let gz_encoder = builder.into_inner().unwrap();
        gz_encoder.finish().unwrap();

        // Validate paths in archive
        assert!(validate_archive_entry_path("gather/SKILL.md").is_ok());

        // Import (simulate)
        let in_file = fs::File::open(&archive_path).unwrap();
        let gz = GzDecoder::new(BufReader::new(in_file));
        let mut archive = tar::Archive::new(gz);
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let entry_path = entry.path().unwrap().to_string_lossy().to_string();
            if entry.header().entry_type().is_dir() {
                continue;
            }
            validate_archive_entry_path(&entry_path).unwrap();
            let skill_name = entry_path.trim_end_matches("/SKILL.md");
            let dst_dir = dst_skills.path().join(skill_name);
            fs::create_dir_all(&dst_dir).unwrap();
            let mut content_bytes = Vec::new();
            std::io::copy(&mut entry, &mut content_bytes).unwrap();
            fs::write(dst_dir.join("SKILL.md"), &content_bytes).unwrap();
        }

        // Verify content preserved
        let imported_content =
            fs::read_to_string(dst_skills.path().join("gather").join("SKILL.md")).unwrap();
        assert_eq!(imported_content, content);
        let _ = archive_path_str;
    }

    #[test]
    fn test_export_import_hierarchical_round_trip() {
        let src_skills = tempfile::tempdir().unwrap();
        let dst_skills = tempfile::tempdir().unwrap();
        let archive_dir = tempfile::tempdir().unwrap();

        let content =
            "---\nname: issue/gather\ndescription: Issue gather skill\n---\n\nInstructions.\n";
        make_skill_dir(src_skills.path(), "issue/gather", content);

        let archive_path = archive_dir.path().join("skills.tar.gz");

        let out_file = fs::File::create(&archive_path).unwrap();
        let gz = GzEncoder::new(out_file, Compression::default());
        let mut builder = tar::Builder::new(gz);
        let skill_md = src_skills
            .path()
            .join("issue")
            .join("gather")
            .join("SKILL.md");
        builder
            .append_path_with_name(&skill_md, "issue/gather/SKILL.md")
            .unwrap();
        builder.finish().unwrap();
        let gz_encoder = builder.into_inner().unwrap();
        gz_encoder.finish().unwrap();

        // Validate hierarchical path
        assert!(validate_archive_entry_path("issue/gather/SKILL.md").is_ok());

        // Import
        let in_file = fs::File::open(&archive_path).unwrap();
        let gz = GzDecoder::new(BufReader::new(in_file));
        let mut archive = tar::Archive::new(gz);
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let entry_path = entry.path().unwrap().to_string_lossy().to_string();
            if entry.header().entry_type().is_dir() {
                continue;
            }
            validate_archive_entry_path(&entry_path).unwrap();
            let skill_name = entry_path.trim_end_matches("/SKILL.md");
            let dst_dir = dst_skills.path().join(skill_name);
            fs::create_dir_all(&dst_dir).unwrap();
            let mut content_bytes = Vec::new();
            std::io::copy(&mut entry, &mut content_bytes).unwrap();
            fs::write(dst_dir.join("SKILL.md"), &content_bytes).unwrap();
        }

        let imported = fs::read_to_string(
            dst_skills
                .path()
                .join("issue")
                .join("gather")
                .join("SKILL.md"),
        )
        .unwrap();
        assert_eq!(imported, content);
    }

    // ── local overrides global priority ─────────────────────────────────────

    #[test]
    fn test_local_skill_overrides_global_in_list() {
        let local_dir = tempfile::tempdir().unwrap();
        let global_dir = tempfile::tempdir().unwrap();

        // Same skill name in both
        let local_content = "---\nname: my-skill\ndescription: Local version\n---\n\nLocal.\n";
        let global_content = "---\nname: my-skill\ndescription: Global version\n---\n\nGlobal.\n";
        make_skill_dir(local_dir.path(), "my-skill", local_content);
        make_skill_dir(global_dir.path(), "my-skill", global_content);

        // Scan local
        let mut entries: Vec<SkillEntry> = Vec::new();
        scan_skills_dir(
            local_dir.path(),
            local_dir.path(),
            SkillSource::Local,
            &mut entries,
        );
        let mut seen: std::collections::HashSet<String> =
            entries.iter().map(|e| e.name.clone()).collect();

        // Scan global, deduplicate
        let mut global_entries: Vec<SkillEntry> = Vec::new();
        scan_skills_dir(
            global_dir.path(),
            global_dir.path(),
            SkillSource::Global,
            &mut global_entries,
        );
        for e in global_entries {
            if !seen.contains(&e.name) {
                let name = e.name.clone();
                entries.push(e);
                seen.insert(name);
            }
        }

        // Should only have one entry, and it's the local one
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].source, SkillSource::Local);
        assert_eq!(entries[0].description, "Local version");
    }

    // ── install: copy between directories ───────────────────────────────────

    #[test]
    fn test_install_global_copies_skill() {
        let local_skills = tempfile::tempdir().unwrap();
        let global_skills = tempfile::tempdir().unwrap();

        let content = "---\nname: gather\ndescription: Gather\n---\n\nInstructions.\n";
        make_skill_dir(local_skills.path(), "gather", content);

        // Simulate the copy that install --global does
        let src = local_skills.path().join("gather");
        let dst = global_skills.path().join("gather");
        copy_dir_all(&src, &dst).unwrap();

        let installed = fs::read_to_string(dst.join("SKILL.md")).unwrap();
        assert_eq!(installed, content);
    }

    #[test]
    fn test_install_from_repo_copies_skill() {
        let repo_skills = tempfile::tempdir().unwrap();
        let local_skills = tempfile::tempdir().unwrap();

        let content =
            "---\nname: impl/run\ndescription: Implementation run\n---\n\nInstructions.\n";
        make_skill_dir(repo_skills.path(), "impl/run", content);

        // Simulate the copy that install --from-repo does
        let src = repo_skills.path().join("impl").join("run");
        let dst = local_skills.path().join("impl").join("run");
        copy_dir_all(&src, &dst).unwrap();

        let installed = fs::read_to_string(dst.join("SKILL.md")).unwrap();
        assert_eq!(installed, content);
    }

    // ── import archive --yes overwrites ──────────────────────────────────────

    #[test]
    fn test_import_yes_overwrites_existing_skill() {
        let src_skills = tempfile::tempdir().unwrap();
        let dst_skills = tempfile::tempdir().unwrap();
        let archive_dir = tempfile::tempdir().unwrap();

        let original = "---\nname: gather\ndescription: Original\n---\n\nOriginal.\n";
        let updated = "---\nname: gather\ndescription: Updated\n---\n\nUpdated.\n";

        // Write original to destination
        make_skill_dir(dst_skills.path(), "gather", original);

        // Create archive with updated content
        make_skill_dir(src_skills.path(), "gather", updated);
        let archive_path = archive_dir.path().join("skills.tar.gz");
        let out_file = fs::File::create(&archive_path).unwrap();
        let gz = GzEncoder::new(out_file, Compression::default());
        let mut builder = tar::Builder::new(gz);
        let skill_md = src_skills.path().join("gather").join("SKILL.md");
        builder
            .append_path_with_name(&skill_md, "gather/SKILL.md")
            .unwrap();
        builder.finish().unwrap();
        let gz_encoder = builder.into_inner().unwrap();
        gz_encoder.finish().unwrap();

        // Import with yes=true
        let entries = read_archive_entries(archive_path.to_str().unwrap()).unwrap();
        let (imported, overwritten, skipped) =
            write_archive_entries(entries, dst_skills.path(), true).unwrap();

        assert_eq!(imported, 0);
        assert_eq!(overwritten, 1);
        assert_eq!(skipped, 0);

        let result = fs::read_to_string(dst_skills.path().join("gather").join("SKILL.md")).unwrap();
        assert_eq!(result, updated);
    }

    #[test]
    fn test_global_skill_visible_when_no_local() {
        let local_dir = tempfile::tempdir().unwrap();
        let global_dir = tempfile::tempdir().unwrap();

        // Only in global
        let global_content =
            "---\nname: shared-skill\ndescription: Shared globally\n---\n\nGlobal.\n";
        make_skill_dir(global_dir.path(), "shared-skill", global_content);

        let mut entries: Vec<SkillEntry> = Vec::new();
        scan_skills_dir(
            local_dir.path(),
            local_dir.path(),
            SkillSource::Local,
            &mut entries,
        );
        let seen: std::collections::HashSet<String> =
            entries.iter().map(|e| e.name.clone()).collect();

        let mut global_entries: Vec<SkillEntry> = Vec::new();
        scan_skills_dir(
            global_dir.path(),
            global_dir.path(),
            SkillSource::Global,
            &mut global_entries,
        );
        for e in global_entries {
            if !seen.contains(&e.name) {
                entries.push(e);
            }
        }

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].source, SkillSource::Global);
    }
}
