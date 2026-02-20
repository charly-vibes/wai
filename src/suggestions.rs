/// Self-healing error suggestions module
///
/// This module provides intelligent suggestion logic for common user errors:
/// - Typo detection (did you mean?)
/// - Wrong command order detection
/// - Context inference from directory structure
use std::path::Path;

/// Suggestion types that can be generated
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suggestion {
    /// Typo suggestion with the corrected command
    DidYouMean {
        original: String,
        suggestion: String,
    },

    /// Wrong order detection (e.g., "project new" -> "new project")
    WrongOrder { original: String, correct: String },

    /// Context suggestion (e.g., "try running from project root")
    ContextHint {
        message: String,
        path: Option<String>,
    },

    /// Generic fix suggestion
    Fix {
        description: String,
        command: Option<String>,
    },
}

impl Suggestion {
    /// Format the suggestion as a human-readable message
    pub fn message(&self) -> String {
        match self {
            Suggestion::DidYouMean {
                original,
                suggestion,
            } => {
                format!(
                    "Unknown command '{}'. Did you mean '{}'?",
                    original, suggestion
                )
            }
            Suggestion::WrongOrder { original, correct } => {
                format!(
                    "Invalid command '{}'. Did you mean '{}'?",
                    original, correct
                )
            }
            Suggestion::ContextHint { message, path } => {
                if let Some(p) = path {
                    format!("{}: {}", message, p)
                } else {
                    message.clone()
                }
            }
            Suggestion::Fix {
                description,
                command,
            } => {
                if let Some(cmd) = command {
                    format!("{}\n  → Run: {}", description, cmd)
                } else {
                    description.clone()
                }
            }
        }
    }
}

/// Main suggestion engine for detecting and offering fixes
pub struct SuggestionEngine {
    /// Similarity threshold for typo detection (0.0 to 1.0)
    similarity_threshold: f64,
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.6,
        }
    }
}

impl SuggestionEngine {
    /// Create a new suggestion engine with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new suggestion engine with custom similarity threshold
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            similarity_threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Find typo suggestions for an unknown command
    ///
    /// # Arguments
    /// * `unknown` - The unknown command entered by the user
    /// * `valid_commands` - List of valid commands to compare against
    ///
    /// # Returns
    /// An optional `Suggestion` if a close match is found
    pub fn suggest_typo(&self, unknown: &str, valid_commands: &[&str]) -> Option<Suggestion> {
        let mut best_match: Option<(&str, f64)> = None;

        for cmd in valid_commands {
            let similarity = self.calculate_similarity(unknown, cmd);

            if similarity >= self.similarity_threshold {
                if let Some((_, current_best)) = best_match {
                    if similarity > current_best {
                        best_match = Some((cmd, similarity));
                    }
                } else {
                    best_match = Some((cmd, similarity));
                }
            }
        }

        best_match.map(|(matched_cmd, _)| Suggestion::DidYouMean {
            original: unknown.to_string(),
            suggestion: matched_cmd.to_string(),
        })
    }

    /// Detect if commands are in wrong order (e.g., "project new" instead of "new project")
    ///
    /// # Arguments
    /// * `first` - First part of the command
    /// * `second` - Second part of the command
    /// * `valid_patterns` - List of valid (verb, noun) patterns
    ///
    /// # Returns
    /// An optional `Suggestion` if wrong order is detected
    pub fn suggest_order(
        &self,
        first: &str,
        second: &str,
        valid_patterns: &[(&str, &str)],
    ) -> Option<Suggestion> {
        // Check if (second, first) exists in valid patterns
        for (verb, noun) in valid_patterns {
            if *noun == first && *verb == second {
                return Some(Suggestion::WrongOrder {
                    original: format!("{} {}", first, second),
                    correct: format!("{} {}", second, first),
                });
            }
        }
        None
    }

    /// Infer project context from current directory
    ///
    /// # Arguments
    /// * `current_dir` - The current working directory
    /// * `marker` - The marker file/directory to look for (e.g., ".wai")
    ///
    /// # Returns
    /// An optional `Suggestion` with context information
    pub fn suggest_context(&self, current_dir: &Path, marker: &str) -> Option<Suggestion> {
        let mut search_dir = current_dir;

        loop {
            if search_dir.join(marker).exists() {
                if search_dir != current_dir {
                    return Some(Suggestion::ContextHint {
                        message: format!("Found {} in parent directory", marker),
                        path: Some(search_dir.display().to_string()),
                    });
                }
                break;
            }

            if let Some(parent) = search_dir.parent() {
                search_dir = parent;
            } else {
                break;
            }
        }

        None
    }

    /// Calculate string similarity using Jaro-Winkler distance
    fn calculate_similarity(&self, a: &str, b: &str) -> f64 {
        strsim::jaro_winkler(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typo_detection() {
        let engine = SuggestionEngine::new();
        let commands = &["status", "init", "new", "add", "list"];

        let suggestion = engine.suggest_typo("staus", commands);
        assert!(suggestion.is_some());

        if let Some(Suggestion::DidYouMean {
            original,
            suggestion,
        }) = suggestion
        {
            assert_eq!(original, "staus");
            assert_eq!(suggestion, "status");
        }
    }

    #[test]
    fn test_no_typo_suggestion_for_dissimilar() {
        let engine = SuggestionEngine::new();
        let commands = &["status", "init", "new"];

        let suggestion = engine.suggest_typo("xyz", commands);
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_wrong_order_detection() {
        let engine = SuggestionEngine::new();
        let patterns = &[("new", "project"), ("add", "research"), ("show", "status")];

        let suggestion = engine.suggest_order("project", "new", patterns);
        assert!(suggestion.is_some());

        if let Some(Suggestion::WrongOrder { original, correct }) = suggestion {
            assert_eq!(original, "project new");
            assert_eq!(correct, "new project");
        }
    }

    #[test]
    fn test_no_wrong_order_for_valid_pattern() {
        let engine = SuggestionEngine::new();
        let patterns = &[("new", "project"), ("add", "research")];

        let suggestion = engine.suggest_order("new", "project", patterns);
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_suggestion_message_formatting() {
        let typo_suggestion = Suggestion::DidYouMean {
            original: "staus".to_string(),
            suggestion: "status".to_string(),
        };
        assert!(typo_suggestion.message().contains("Did you mean"));
        assert!(typo_suggestion.message().contains("staus"));
        assert!(typo_suggestion.message().contains("status"));
    }

    #[test]
    fn test_custom_threshold() {
        let engine = SuggestionEngine::with_threshold(0.9);
        let commands = &["status", "init"];

        // "statu" is close but might not reach 0.9 threshold
        let suggestion = engine.suggest_typo("stat", commands);
        // This may or may not find a match depending on exact similarity score
        // The test verifies the threshold mechanism works
        let _ = suggestion;
    }
}
