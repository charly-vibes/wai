use miette::Result;

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
}
