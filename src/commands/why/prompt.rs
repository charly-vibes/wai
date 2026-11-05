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
    let parts = vec![
        "You are an oracle helping understand why code and decisions exist as they do.\n"
            .to_string(),
        format!("# User Question\n{}\n", ctx.query),
        meta_section(&ctx.meta),
        artifacts_section(&ctx.artifacts),
        memories_section(&ctx.memories),
        git_section(&ctx.git_context),
        truncation_note(ctx.truncated),
        task_section(),
    ];
    parts.join("\n")
}

fn meta_section(meta: &super::context::ProjectMeta) -> String {
    let phase_line = meta
        .current_phase
        .as_ref()
        .map(|phase| format!("- Current phase: {}", phase));
    let commits_line = (!meta.recent_commits.is_empty()).then(|| {
        format!(
            "- Recent commits:\n{}",
            meta.recent_commits
                .iter()
                .map(|c| format!("  - {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        )
    });
    let lines: Vec<String> = phase_line.into_iter().chain(commits_line).collect();
    if lines.is_empty() {
        String::new()
    } else {
        format!("# Project Context\n{}\n", lines.join("\n"))
    }
}

/// One artifact's fenced block, truncated so total usage stays under the cap.
/// Returns `None` when the budget is exhausted.
fn artifact_block(
    artifact: &super::context::Artifact,
    chars_used: usize,
) -> Option<(String, usize)> {
    use super::context::MAX_CONTEXT_CHARS;

    let raw = escape_artifact(&artifact.content);
    let (body, note) = if chars_used + raw.len() > MAX_CONTEXT_CHARS {
        let budget = MAX_CONTEXT_CHARS.saturating_sub(chars_used);
        if budget == 0 {
            return None;
        }
        let over = raw.len() - budget;
        (
            raw[..budget].to_string(),
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
    Some((block.clone(), chars_used + block.len()))
}

fn artifacts_section(artifacts: &[super::context::Artifact]) -> String {
    if artifacts.is_empty() {
        return String::new();
    }
    let header = "# Available Artifacts\n";
    // Each artifact's content is individually capped at MAX_CONTEXT_CHARS so
    // a single massive file cannot blow out the prompt budget.
    let header_len = header.len();
    let (text, _, _) = artifacts.iter().fold(
        (String::from(header), header_len, false),
        |(text, used, done), artifact| {
            if done {
                return (text, used, done);
            }
            match artifact_block(artifact, used) {
                Some((block, next)) => (text + &block, next, false),
                None => (text, used, true),
            }
        },
    );
    text
}

fn memories_section(memories: &Option<String>) -> String {
    memories
        .as_ref()
        .map(|memories| {
            format!(
                "# Stored Memories (bd)\n\
                 These are persistent project memories. Use them as supplementary context \
                 when answering the question:\n\n{}\n",
                memories
            )
        })
        .unwrap_or_default()
}

fn git_section(git_context: &Option<String>) -> String {
    git_context
        .as_ref()
        .map(|git| format!("# Git Context\n{}\n", git))
        .unwrap_or_default()
}

fn truncation_note(truncated: bool) -> String {
    if truncated {
        "*Note: Context was truncated to fit within token limits.*\n".to_string()
    } else {
        String::new()
    }
}

fn task_section() -> String {
    "# Task\nIdentify 3-5 most relevant artifacts. Explain why each is relevant. \
        Synthesize a narrative showing how the decision evolved (research → design → plan). \
        Suggest concrete next steps.\n\n\
        Format your response as Markdown with these sections:\n\
        ## Answer\n## Relevant Artifacts\n## Decision Chain\n## Suggestions"
        .to_string()
}
