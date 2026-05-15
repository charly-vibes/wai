// ── Response parsing ───────────────────────────────────────────────────────────

/// Relevance level extracted from the LLM's artifact references.
#[derive(Debug, Clone, PartialEq)]
pub enum Relevance {
    High,
    Medium,
    Low,
}

impl Relevance {
    pub(super) fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "high" => Some(Relevance::High),
            "medium" | "med" => Some(Relevance::Medium),
            "low" => Some(Relevance::Low),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Relevance::High => "High",
            Relevance::Medium => "Medium",
            Relevance::Low => "Low",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Relevance::High => "●",
            Relevance::Medium => "◐",
            Relevance::Low => "○",
        }
    }
}

/// A single artifact reference extracted from the LLM response.
#[derive(Debug, Clone)]
pub struct ArtifactRef {
    pub path: String,
    pub description: String,
    pub relevance: Option<Relevance>,
}

/// LLM response parsed into structured sections.
#[derive(Debug)]
pub struct ParsedResponse {
    pub answer: String,
    pub relevant_artifacts: Vec<ArtifactRef>,
    pub decision_chain: String,
    pub suggestions: Vec<String>,
    /// Original raw text from the LLM.
    pub raw: String,
}

/// Parse a raw LLM markdown response into structured sections.
///
/// Handles malformed output gracefully: if no `## ` headers are found, the
/// entire response is treated as the answer.
pub fn parse_response(raw: &str) -> ParsedResponse {
    let sections = split_sections(raw);

    let answer = if sections.is_empty() {
        // Completely malformed — treat whole text as answer
        raw.trim().to_string()
    } else {
        sections.get("Answer").cloned().unwrap_or_default()
    };

    let artifacts_text = sections
        .get("Relevant Artifacts")
        .cloned()
        .unwrap_or_default();
    let relevant_artifacts = parse_artifact_refs(&artifacts_text);

    let decision_chain = sections.get("Decision Chain").cloned().unwrap_or_default();

    let suggestions_text = sections.get("Suggestions").cloned().unwrap_or_default();
    let suggestions = parse_suggestions(&suggestions_text);

    ParsedResponse {
        answer,
        relevant_artifacts,
        decision_chain,
        suggestions,
        raw: raw.to_string(),
    }
}

/// Split markdown text into a map of `section_name → content` by `## ` headers.
pub(super) fn split_sections(text: &str) -> std::collections::HashMap<String, String> {
    let mut sections = std::collections::HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_content = String::new();

    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(name) = current_name.take() {
                sections.insert(name, current_content.trim().to_string());
            }
            current_name = Some(heading.trim().to_string());
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if let Some(name) = current_name {
        sections.insert(name, current_content.trim().to_string());
    }
    sections
}

/// Parse artifact references from the `## Relevant Artifacts` section.
///
/// Recognises lines like:
/// - `- `.wai/projects/…/file.md` (High) — description`
/// - `- .wai/projects/…/file.md [Medium]: description`
/// - Lines without a recognisable path are skipped.
pub(super) fn parse_artifact_refs(text: &str) -> Vec<ArtifactRef> {
    let mut refs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Strip leading list markers
        let content = line.trim_start_matches(['-', '*', '+']).trim();

        // Extract backtick-quoted path or plain path-like token
        let (path, rest) = if let Some(after_open) = content.strip_prefix('`') {
            if let Some(close) = after_open.find('`') {
                (
                    after_open[..close].to_string(),
                    after_open[close + 1..].trim(),
                )
            } else {
                extract_bare_path(content)
            }
        } else {
            extract_bare_path(content)
        };

        if path.is_empty() || !path.contains('/') {
            continue;
        }

        let relevance = extract_relevance(rest);
        let description = clean_description(rest);

        refs.push(ArtifactRef {
            path,
            description,
            relevance,
        });
    }
    refs
}

/// Extract the first whitespace-delimited token that looks like a path
/// (contains `/`) and return `(path, rest)`.
fn extract_bare_path(content: &str) -> (String, &str) {
    let trimmed = content.trim_start_matches('#').trim();
    if let Some(space) = trimmed.find(char::is_whitespace) {
        let token = &trimmed[..space];
        if token.contains('/') {
            return (token.to_string(), trimmed[space..].trim());
        }
    } else if trimmed.contains('/') {
        return (trimmed.to_string(), "");
    }
    (String::new(), content)
}

/// Look for `(High)`, `[High]`, `(Medium)`, etc. in a string.
pub(super) fn extract_relevance(text: &str) -> Option<Relevance> {
    for word in text.split_whitespace() {
        let stripped =
            word.trim_matches(|c: char| c == '(' || c == ')' || c == '[' || c == ']' || c == ':');
        if let Some(r) = Relevance::from_str(stripped) {
            return Some(r);
        }
    }
    None
}

/// Remove relevance markers and leading punctuation to get a clean description.
fn clean_description(text: &str) -> String {
    // Strip leading brackets/parens relevance tokens then em-dash or colon separators
    let mut s = text.to_string();
    // Remove parenthesised or bracketed relevance markers
    for marker in &["(High)", "(Medium)", "(Low)", "[High]", "[Medium]", "[Low]"] {
        s = s.replace(marker, "");
    }
    // Strip leading —, -, :
    let trimmed = s.trim().trim_start_matches(['—', '-', ':']).trim();
    trimmed.to_string()
}

/// Parse bullet/numbered points from the `## Suggestions` section.
pub(super) fn parse_suggestions(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let stripped = line
                .trim()
                .trim_start_matches(|c: char| {
                    c.is_ascii_digit() || c == '.' || c == '-' || c == '*' || c == '+'
                })
                .trim();
            if stripped.is_empty() {
                None
            } else {
                Some(stripped.to_string())
            }
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_response ──

    #[test]
    fn parse_well_formed_response_extracts_all_sections() {
        let raw = "## Answer\nTOML is simple.\n## Relevant Artifacts\n- `.wai/a.md` (High) — key doc\n## Decision Chain\nResearch → Design\n## Suggestions\n- Use TOML everywhere";
        let p = parse_response(raw);
        assert_eq!(p.answer, "TOML is simple.");
        assert_eq!(p.decision_chain, "Research → Design");
        assert_eq!(p.suggestions, vec!["Use TOML everywhere"]);
    }

    #[test]
    fn parse_malformed_response_uses_raw_as_answer() {
        let raw = "No headers here, just plain text.";
        let p = parse_response(raw);
        assert_eq!(p.answer, raw);
        assert!(p.relevant_artifacts.is_empty());
        assert!(p.suggestions.is_empty());
    }

    #[test]
    fn parse_missing_section_is_empty() {
        let raw = "## Answer\nSome answer.\n## Suggestions\n- Do this";
        let p = parse_response(raw);
        assert!(p.relevant_artifacts.is_empty());
        assert_eq!(p.decision_chain, "");
        assert_eq!(p.suggestions, vec!["Do this"]);
    }

    // ── split_sections ──

    #[test]
    fn split_sections_handles_multiple_sections() {
        let text = "## Foo\nfoo content\n## Bar\nbar content";
        let sections = split_sections(text);
        assert_eq!(sections.get("Foo").map(|s| s.as_str()), Some("foo content"));
        assert_eq!(sections.get("Bar").map(|s| s.as_str()), Some("bar content"));
    }

    #[test]
    fn split_sections_empty_text_returns_empty_map() {
        let sections = split_sections("");
        assert!(sections.is_empty());
    }

    #[test]
    fn split_sections_text_without_headers_returns_empty_map() {
        let sections = split_sections("just plain text, no headers");
        assert!(sections.is_empty());
    }

    // ── extract_relevance ──

    #[test]
    fn extract_relevance_parses_parenthesised_high() {
        assert_eq!(
            extract_relevance("(High) — important"),
            Some(Relevance::High)
        );
    }

    #[test]
    fn extract_relevance_parses_bracketed_medium() {
        assert_eq!(
            extract_relevance("[Medium]: explanation"),
            Some(Relevance::Medium)
        );
    }

    #[test]
    fn extract_relevance_parses_low() {
        assert_eq!(
            extract_relevance("(Low) — less important"),
            Some(Relevance::Low)
        );
    }

    #[test]
    fn extract_relevance_returns_none_when_absent() {
        assert_eq!(extract_relevance("no relevance marker here"), None);
    }

    #[test]
    fn extract_relevance_case_insensitive() {
        assert_eq!(extract_relevance("(high)"), Some(Relevance::High));
        assert_eq!(extract_relevance("(MEDIUM)"), Some(Relevance::Medium));
    }

    // ── parse_artifact_refs ──

    #[test]
    fn parse_artifact_refs_extracts_backtick_path_and_relevance() {
        let text = "- `.wai/projects/why/research/2024-01-01.md` (High) — explains rationale";
        let refs = parse_artifact_refs(text);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, ".wai/projects/why/research/2024-01-01.md");
        assert_eq!(refs[0].relevance, Some(Relevance::High));
        assert!(refs[0].description.contains("explains rationale"));
    }

    #[test]
    fn parse_artifact_refs_extracts_bare_path() {
        let text = "- .wai/projects/why/design/arch.md [Medium]: architecture doc";
        let refs = parse_artifact_refs(text);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, ".wai/projects/why/design/arch.md");
        assert_eq!(refs[0].relevance, Some(Relevance::Medium));
    }

    #[test]
    fn parse_artifact_refs_skips_lines_without_paths() {
        let text = "- No path here\n- also no path\n- `.wai/foo/bar.md` (Low) — desc";
        let refs = parse_artifact_refs(text);
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn parse_artifact_refs_empty_text_returns_empty() {
        let refs = parse_artifact_refs("");
        assert!(refs.is_empty());
    }

    // ── parse_suggestions ──

    #[test]
    fn parse_suggestions_extracts_bullet_points() {
        let text = "- Update docs\n- Add tests\n- Refactor config";
        let suggestions = parse_suggestions(text);
        assert_eq!(
            suggestions,
            vec!["Update docs", "Add tests", "Refactor config"]
        );
    }

    #[test]
    fn parse_suggestions_strips_numbering() {
        let text = "1. First suggestion\n2. Second suggestion";
        let suggestions = parse_suggestions(text);
        assert_eq!(suggestions, vec!["First suggestion", "Second suggestion"]);
    }

    #[test]
    fn parse_suggestions_skips_blank_lines() {
        let text = "- A\n\n- B";
        let suggestions = parse_suggestions(text);
        assert_eq!(suggestions, vec!["A", "B"]);
    }

    // ── Relevance ──

    #[test]
    fn relevance_as_str_roundtrips() {
        assert_eq!(Relevance::High.as_str(), "High");
        assert_eq!(Relevance::Medium.as_str(), "Medium");
        assert_eq!(Relevance::Low.as_str(), "Low");
    }

    #[test]
    fn relevance_from_str_accepts_med_alias() {
        assert_eq!(Relevance::from_str("med"), Some(Relevance::Medium));
    }
}
