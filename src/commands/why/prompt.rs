//! LLM prompt assembly for `wai why`.

use super::context::GatheredContext;

/// Escape content to prevent triple-backtick fences from breaking the prompt structure.
fn escape_artifact(content: &str) -> String {
    content.replace("```", "~~~")
}

// ── Error messaging ───────────────────────────────────────────────────────────

// ── Prompt builder ────────────────────────────────────────────────────────────

/// Build the prompt sent to the LLM from gathered context.
pub fn build_prompt(ctx: &GatheredContext) -> String {
    use super::context::MAX_CONTEXT_CHARS;

    let mut parts: Vec<String> = Vec::new();

    parts.push(
        "You are an oracle helping understand why code and decisions exist as they do.\n"
            .to_string(),
    );
    parts.push(format!("# User Question\n{}\n", ctx.query));

    // Project metadata
    let mut meta_lines = Vec::new();
    if let Some(ref phase) = ctx.meta.current_phase {
        meta_lines.push(format!("- Current phase: {}", phase));
    }
    if !ctx.meta.recent_commits.is_empty() {
        meta_lines.push(format!(
            "- Recent commits:\n{}",
            ctx.meta
                .recent_commits
                .iter()
                .map(|c| format!("  - {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    if !meta_lines.is_empty() {
        parts.push(format!("# Project Context\n{}\n", meta_lines.join("\n")));
    }

    // Artifacts — wrap in code fences and escape injection attempts.
    // Each artifact's content is individually capped at MAX_CONTEXT_CHARS so
    // a single massive file cannot blow out the prompt budget.
    if !ctx.artifacts.is_empty() {
        let mut artifact_text = String::from("# Available Artifacts\n");
        let mut chars_used: usize = artifact_text.len();
        for artifact in &ctx.artifacts {
            let raw = escape_artifact(&artifact.content);
            let (body, note) = if chars_used + raw.len() > MAX_CONTEXT_CHARS {
                let budget = MAX_CONTEXT_CHARS.saturating_sub(chars_used);
                if budget == 0 {
                    break;
                }
                let over = raw.len() - budget;
                let truncated = &raw[..budget];
                (
                    truncated.to_string(),
                    format!("\n[... truncated: {} chars over budget ...]", over),
                )
            } else {
                (raw, String::new())
            };
            let block = format!(
                "\n## {} ({})\n```\n{}{}\n```\n",
                artifact.rel_path,
                artifact.kind.label(),
                body,
                note,
            );
            chars_used += block.len();
            artifact_text.push_str(&block);
        }
        parts.push(artifact_text);
    }

    if let Some(ref memories) = ctx.memories {
        parts.push(format!(
            "# Stored Memories (bd)\n\
             These are persistent project memories. Use them as supplementary context \
             when answering the question:\n\n{}\n",
            memories
        ));
    }

    // Git context for file queries
    if let Some(ref git) = ctx.git_context {
        parts.push(format!("# Git Context\n{}\n", git));
    }

    if ctx.truncated {
        parts.push("*Note: Context was truncated to fit within token limits.*\n".to_string());
    }

    parts.push(
        "# Task\nIdentify 3-5 most relevant artifacts. Explain why each is relevant. \
        Synthesize a narrative showing how the decision evolved (research → design → plan). \
        Suggest concrete next steps.\n\n\
        Format your response as Markdown with these sections:\n\
        ## Answer\n## Relevant Artifacts\n## Decision Chain\n## Suggestions"
            .to_string(),
    );

    parts.join("\n")
}
