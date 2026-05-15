use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
pub(super) enum SkillSource {
    Local,
    Global,
}

/// Skill entry for listing
#[derive(Debug, Clone, Serialize)]
pub(super) struct SkillEntry {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) category: Option<String>,
    pub(super) description: String,
    pub(super) path: String,
    pub(super) source: SkillSource,
}

/// JSON payload for skill list
#[derive(Debug, Serialize)]
pub(super) struct SkillListPayload {
    pub(super) skills: Vec<SkillEntry>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
