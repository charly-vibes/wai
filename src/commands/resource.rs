use miette::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::cli::{ResourceAddCommands, ResourceImportCommands, ResourceListCommands};
use crate::error::WaiError;

/// Validates a skill name according to the following rules:
/// - Only lowercase a-z, digits 0-9, and hyphens allowed
/// - No leading, trailing, or consecutive hyphens
/// - Maximum 64 characters
/// - Cannot be empty, ".", "..", or start with "."
pub fn validate_skill_name(name: &str) -> Result<(), WaiError> {
    // Check for empty string
    if name.is_empty() {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot be empty".to_string(),
        });
    }

    // Check for special names
    if name == "." || name == ".." {
        return Err(WaiError::InvalidSkillName {
            message: format!("'{}' is not a valid skill name", name),
        });
    }

    // Check for names starting with "."
    if name.starts_with('.') {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot start with '.'".to_string(),
        });
    }

    // Check length
    if name.len() > 64 {
        return Err(WaiError::InvalidSkillName {
            message: format!("Skill name too long ({} chars, max 64)", name.len()),
        });
    }

    // Check for leading hyphen
    if name.starts_with('-') {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot start with a hyphen".to_string(),
        });
    }

    // Check for trailing hyphen
    if name.ends_with('-') {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot end with a hyphen".to_string(),
        });
    }

    // Check for consecutive hyphens
    if name.contains("--") {
        return Err(WaiError::InvalidSkillName {
            message: "Skill name cannot contain consecutive hyphens".to_string(),
        });
    }

    // Check for valid characters (lowercase a-z, digits 0-9, hyphens)
    for (idx, ch) in name.chars().enumerate() {
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
    miette::bail!("Add skill '{}' not yet implemented", name);
}

fn list_skills(json: bool) -> Result<()> {
    if json {
        miette::bail!("List skills (JSON) not yet implemented");
    } else {
        miette::bail!("List skills not yet implemented");
    }
}

fn import_skills(from: Option<String>) -> Result<()> {
    let path = from.unwrap_or_else(|| ".".to_string());
    miette::bail!("Import skills from '{}' not yet implemented", path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_skill_names() {
        // Valid names should pass
        assert!(validate_skill_name("my-skill").is_ok());
        assert!(validate_skill_name("skill123").is_ok());
        assert!(validate_skill_name("a").is_ok());
        assert!(validate_skill_name("my-cool-skill-2").is_ok());
        assert!(validate_skill_name("abc-123-xyz").is_ok());
    }

    #[test]
    fn test_empty_name() {
        let result = validate_skill_name("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot be empty"));
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot start with '.'"));
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot start with a hyphen"));
    }

    #[test]
    fn test_trailing_hyphen() {
        let result = validate_skill_name("skill-");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot end with a hyphen"));
    }

    #[test]
    fn test_consecutive_hyphens() {
        let result = validate_skill_name("my--skill");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("consecutive hyphens"));
    }

    #[test]
    fn test_invalid_characters() {
        // Uppercase
        let result = validate_skill_name("MySkill");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid character"));

        // Underscore
        let result = validate_skill_name("my_skill");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid character"));

        // Space
        let result = validate_skill_name("my skill");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid character"));

        // Special characters
        let result = validate_skill_name("my@skill");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid character"));
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
}
