use crate::error::WaiError;

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
pub(super) fn validate_skill_name_segment(segment: &str) -> Result<(), WaiError> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
