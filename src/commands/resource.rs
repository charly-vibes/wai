use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::cli::{ResourceAddCommands, ResourceImportCommands, ResourceListCommands};
use crate::config::{SKILLS_DIR, agent_config_dir};
use crate::context::require_safe_mode;
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

/// Skill entry for listing
#[derive(Debug, Clone, Serialize)]
struct SkillEntry {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    description: String,
    path: String,
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
    let metadata: SkillMetadata = serde_yaml::from_str(&yaml_content).ok()?;

    // Validate that required fields are not empty
    if metadata.name.trim().is_empty() || metadata.description.trim().is_empty() {
        return None;
    }

    Some(metadata)
}

pub fn run_add(cmd: ResourceAddCommands) -> Result<()> {
    match cmd {
        ResourceAddCommands::Skill { name } => add_skill(&name),
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
    }
}

fn add_skill(name: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("add skill")?;

    // Validate skill name
    validate_skill_name(name)?;

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

    // Create SKILL.md with template
    let skill_file = skill_dir.join("SKILL.md");
    let title = kebab_to_title_case(name);
    let template = format!(
        r#"---
name: {}
description: ""
---

# {}

Instructions go here.
"#,
        name, title
    );

    fs::write(&skill_file, template).into_diagnostic()?;

    log::success(format!("Created skill '{}'", name)).into_diagnostic()?;
    log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;

    Ok(())
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
    let project_root = require_project()?;
    let skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);

    // Check if skills directory exists
    if !skills_dir.exists() {
        if json {
            let payload = SkillListPayload { skills: vec![] };
            crate::output::print_json(&payload)?;
        } else {
            println!();
            println!("  {} No skills found", "○".dimmed());
            println!(
                "  {} Run 'wai resource add skill <name>' to create one",
                "→".cyan()
            );
            println!();
        }
        return Ok(());
    }

    // Scan skills directory (flat skills and one level of category subdirectories)
    let mut entries = Vec::new();
    for entry in fs::read_dir(&skills_dir).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();

        // Only process directories
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
            // Flat skill: directory contains SKILL.md directly
            let relative_path = path
                .strip_prefix(&project_root)
                .unwrap_or(&path)
                .display()
                .to_string();
            if let Some(metadata) = parse_skill_frontmatter(&skill_file) {
                entries.push(SkillEntry {
                    name: metadata.name,
                    category: None,
                    description: metadata.description,
                    path: relative_path,
                });
            } else {
                entries.push(SkillEntry {
                    name: dir_name,
                    category: None,
                    description: "(no metadata)".to_string(),
                    path: relative_path,
                });
            }
        } else {
            // Category directory: recurse one level for hierarchical skills
            for sub_entry in fs::read_dir(&path).into_diagnostic()? {
                let sub_entry = sub_entry.into_diagnostic()?;
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
                    .strip_prefix(&project_root)
                    .unwrap_or(&sub_path)
                    .display()
                    .to_string();

                if let Some(metadata) = parse_skill_frontmatter(&sub_skill_file) {
                    entries.push(SkillEntry {
                        name: metadata.name,
                        category: Some(dir_name.clone()),
                        description: metadata.description,
                        path: relative_path,
                    });
                } else {
                    entries.push(SkillEntry {
                        name: hierarchical_name,
                        category: Some(dir_name.clone()),
                        description: "(no metadata)".to_string(),
                        path: relative_path,
                    });
                }
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

            if entry.description == "(no metadata)" {
                println!("    {} {} {}", "•".dimmed(), entry.name, desc.dimmed());
            } else {
                println!(
                    "    {} {} {}",
                    "•".dimmed(),
                    entry.name.bold(),
                    desc.dimmed()
                );
            }
        }
        println!();
    }

    Ok(())
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
        assert_eq!(kebab_to_title_case("my-cat/my-action"), "My Cat / My Action");
    }
}
