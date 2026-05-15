pub mod context;
pub mod meta;

// Re-export commonly used items so callers (e.g. close.rs) can import from
// `super::reflect::` without knowing the submodule layout.
pub use context::{count_handoffs_since, gather_reflect_context};
pub use meta::{
    predict_reflect_resource_path, read_reflect_meta, write_reflect_meta, write_reflect_resource,
};

use std::path::{Path, PathBuf};

use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::config::ProjectConfig;
use crate::llm::{AGENT_SENTINEL, detect_backend};
use crate::managed_block::{
    REFLECT_REF_END, REFLECT_REF_START, has_reflect_block, read_reflect_block,
    wai_reflect_ref_content,
};
use crate::plugin::store_memory;

use context::ReflectContext;
use meta::ReflectMeta;

// ── Output target detection ──────────────────────────────────────────────────

/// Detect the output target(s) based on `--output` override and what files
/// actually exist in `repo_root`.
///
/// Returns an error if neither CLAUDE.md nor AGENTS.md exist and no explicit
/// `--output` was given (or if the given `--output` target file doesn't exist).
pub fn detect_output_targets(
    repo_root: &Path,
    output_override: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let claude_md = repo_root.join("CLAUDE.md");
    let agents_md = repo_root.join("AGENTS.md");

    match output_override {
        Some("claude.md") => Ok(vec![claude_md]),
        Some("agents.md") => Ok(vec![agents_md]),
        Some("both") => Ok(vec![claude_md, agents_md]),
        Some(other) => {
            miette::bail!(
                "Unknown output target '{}'. Use 'claude.md', 'agents.md', or 'both'.",
                other
            )
        }
        None => {
            let has_claude = claude_md.exists();
            let has_agents = agents_md.exists();
            if !has_claude && !has_agents {
                miette::bail!(
                    "No CLAUDE.md or AGENTS.md found in '{}'. \
                     Run `wai init` first or create the target file manually.",
                    repo_root.display()
                )
            }
            let mut targets = Vec::new();
            if has_claude {
                targets.push(claude_md);
            }
            if has_agents {
                targets.push(agents_md);
            }
            Ok(targets)
        }
    }
}

// ── LLM integration (Phase 3) ─────────────────────────────────────────────────

/// Escape triple-backtick fences in artifact content to prevent prompt injection.
fn escape_fences(content: &str) -> String {
    content.replace("```", "~~~")
}

/// Build the reflection prompt from gathered context.
///
/// The prompt instructs the LLM to produce a WAI:REFLECT block in the format
/// defined in design.md.
pub fn build_reflect_prompt(ctx: &ReflectContext, today: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push(
        "You are synthesizing project-specific AI assistant guidance.\n\
         Your goal: read the session context below and extract patterns, conventions, \
         gotchas, and architectural notes that AI assistants should know when working on \
         this project. Focus on information that is NOT already in the 'Already Documented' \
         section below.\n"
            .to_string(),
    );

    parts.push(
        "# Input Hierarchy\n\
         The context below comes from three tiers, ranked by richness:\n\
         1. **Conversation transcript** — raw session detail; failed attempts, surprises, \
            step-by-step struggles (most information-dense)\n\
         2. **Handoff artifacts** — session summaries; intent, next steps, and gotchas\n\
         3. **Research/design/plan artifacts** — explicit decisions and domain knowledge\n\
         When referencing patterns, note the artifact date. If an artifact is older than \
         6 months, flag it as potentially stale.\n"
            .to_string(),
    );

    if !ctx.existing_blocks.is_empty() {
        let mut already = String::from("# Already Documented\n");
        already.push_str(
            "The following content is already in the REFLECT block. Do NOT repeat it — \
             only add new, distinct learnings:\n\n",
        );
        for (path, block) in &ctx.existing_blocks {
            already.push_str(&format!(
                "## Existing block in {}\n```\n{}\n```\n\n",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown"),
                escape_fences(block)
            ));
        }
        parts.push(already);
    }

    if let Some(ref memories) = ctx.memories {
        parts.push(format!(
            "# Already in Global Memories\n\
             The following insights are already captured as persistent memories in bd. \
             Do NOT re-derive or repeat them:\n\n{}\n",
            memories
        ));
    }

    if let Some(ref transcript) = ctx.conversation {
        parts.push(format!(
            "# Conversation Transcript\n```\n{}\n```\n",
            escape_fences(transcript)
        ));
    }

    if !ctx.handoffs.is_empty() {
        let mut section = String::from("# Handoff Artifacts\n");
        for h in &ctx.handoffs {
            section.push_str(&format!(
                "\n## {}\n```\n{}\n```\n",
                h.rel_path,
                escape_fences(&h.content)
            ));
        }
        parts.push(section);
    }

    if !ctx.secondary.is_empty() {
        let mut section = String::from("# Research / Design / Plan Artifacts\n");
        for s in &ctx.secondary {
            section.push_str(&format!(
                "\n## {} ({})\n```\n{}\n```\n",
                s.rel_path,
                s.kind,
                escape_fences(&s.content)
            ));
        }
        parts.push(section);
    }

    if !ctx.previous_reflections.is_empty() {
        let mut section = String::from(
            "# Previous Reflections\n\nExtend and correct these — do not repeat them verbatim:\n",
        );
        for r in &ctx.previous_reflections {
            section.push_str(&format!(
                "\n## {}\n```\n{}\n```\n",
                r.rel_path,
                escape_fences(&r.content)
            ));
        }
        parts.push(section);
    }

    parts.push(format!(
        "# Output Instructions\n\
         Today's date: {today}\n\n\
         Produce ONLY the inner content for a WAI:REFLECT block — no extra commentary. \
         The content will be wrapped in <!-- WAI:REFLECT:START --> and <!-- WAI:REFLECT:END --> \
         markers by the tool. Start your response with:\n\n\
         ## Project-Specific AI Context\n\
         _Last reflected: {today} · N sessions analyzed_\n\n\
         Then include whichever of the following sections are relevant (omit sections \
         where you found nothing new):\n\
         ### Conventions\n\
         ### Common Gotchas\n\
         ### Steps That Tend to Require Multiple Tries\n\
         ### Architecture Notes\n\n\
         Be concise and actionable. Each bullet should help an AI assistant avoid \
         repeating a past mistake or discovering a known pattern from scratch."
    ));

    parts.join("\n")
}

/// Extract the inner content of a WAI:REFLECT block from an LLM response.
///
/// The LLM may:
/// 1. Return the block wrapped in WAI:REFLECT markers → extract the inner content
/// 2. Return the block wrapped in markdown fences → strip the fences
/// 3. Return raw content → use as-is
pub fn extract_reflect_content(response: &str) -> String {
    const START: &str = "<!-- WAI:REFLECT:START -->";
    const END: &str = "<!-- WAI:REFLECT:END -->";

    // Try to extract from WAI:REFLECT markers first.
    if let (Some(start_idx), Some(end_idx)) = (response.find(START), response.find(END)) {
        let inner_start = start_idx + START.len();
        if inner_start <= end_idx {
            return response[inner_start..end_idx].trim().to_string();
        }
    }

    // Strip leading/trailing markdown fences.
    let trimmed = response.trim();
    if trimmed.starts_with("```") {
        let after_fence = trimmed.trim_start_matches('`');
        // Skip the optional language hint on the first line.
        let body = after_fence
            .find('\n')
            .map(|i| &after_fence[i + 1..])
            .unwrap_or(after_fence);
        let body = if body.trim_end().ends_with("```") {
            let end = body.trim_end().len() - 3;
            body[..end].trim_end()
        } else {
            body.trim_end()
        };
        return body.trim().to_string();
    }

    trimmed.to_string()
}

/// Call the LLM backend with the given prompt and return the raw response.
///
/// Reuses `LlmConfig` and `detect_backend` from `src/llm.rs` (task 3.1).
/// Returns an error if no backend is available.
///
/// If the env var `WAI_REFLECT_MOCK_RESPONSE` is set, its value is returned
/// directly without calling any LLM (used for integration testing only).
pub fn call_llm(project_root: &Path, prompt: &str) -> Result<String> {
    if let Ok(mock) = std::env::var("WAI_REFLECT_MOCK_RESPONSE") {
        return Ok(mock);
    }

    let why_cfg = ProjectConfig::load(project_root)
        .map(|c| c.llm_config().into_owned())
        .unwrap_or_default();

    let backend = detect_backend(&why_cfg).ok_or_else(|| {
        miette::miette!(
            "No LLM available. Configure one in .wai/config.toml under [llm], \
             or set ANTHROPIC_API_KEY."
        )
    })?;

    let _ = prompt;
    backend
        .complete(prompt)
        .map_err(|e| miette::miette!("LLM error: {}", e))
}

// ── Command handler ───────────────────────────────────────────────────────────

/// Extract top-level bullet point text from a markdown string.
///
/// A top-level bullet is a line starting with `- ` or `* ` (not indented).
/// The bullet marker is stripped. Each bullet text is truncated to 60 chars
/// for use as a bd memory key.
pub fn extract_top_level_bullets(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let stripped = if let Some(rest) = line.strip_prefix("- ") {
                rest
            } else {
                line.strip_prefix("* ")?
            };
            let text = stripped.trim().to_string();
            if text.is_empty() {
                return None;
            }
            // Truncate to 60 chars (by char count, not bytes)
            let truncated: String = text.chars().take(60).collect();
            Some(truncated)
        })
        .collect()
}

pub struct ReflectArgs {
    pub project: Option<String>,
    pub conversation: Option<PathBuf>,
    pub output: Option<String>,
    pub dry_run: bool,
    pub yes: bool,
    pub inject_content: Option<String>,
    pub verbose: u8,
    pub save_memories: bool,
}

pub fn run(args: ReflectArgs) -> Result<()> {
    let ReflectArgs {
        project,
        conversation,
        output,
        dry_run,
        yes: _yes,
        inject_content,
        verbose: _verbose,
        save_memories,
    } = args;
    let project_root = super::require_project()?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Resolve project_name early as Option<String>.
    // Uses unified resolution but falls back to None rather than erroring,
    // since reflect can operate without a specific project.
    let project_name: Option<String> = super::resolve_project(&project_root, project.as_deref())
        .ok()
        .map(|r| r.name);

    // Detect output targets (CLAUDE.md / AGENTS.md).
    let targets = detect_output_targets(&project_root, output.as_deref())?;

    // ── Migration step (3.1–3.2) ──────────────────────────────────────────────
    // Scan target files for old WAI:REFLECT:START/END blocks.
    // If any exist, migrate once and replace with slim REF blocks.
    {
        let refl_dir = crate::config::reflections_dir(&project_root);
        // Check whether a *-migrated.md already exists.
        let migrated_exists = if refl_dir.exists() {
            std::fs::read_dir(&refl_dir)
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .any(|e| e.file_name().to_string_lossy().contains("-migrated"))
                })
                .unwrap_or(false)
        } else {
            false
        };

        let ref_block = format!(
            "{}\n{}{}\n",
            REFLECT_REF_START,
            wai_reflect_ref_content(),
            REFLECT_REF_END
        );

        let mut migration_notice_printed = false;
        let mut first_content: Option<String> = None;

        for target in &targets {
            if !has_reflect_block(target) {
                continue;
            }

            // Extract content from the first file that has the block for migration.
            if first_content.is_none() {
                first_content = read_reflect_block(target);
            }

            // Replace old WAI:REFLECT block with WAI:REFLECT:REF block.
            let existing = std::fs::read_to_string(target).into_diagnostic()?;
            // Find and replace the REFLECT:START/END block with the REF block.
            let reflect_start_marker = "<!-- WAI:REFLECT:START -->";
            let reflect_end_marker = "<!-- WAI:REFLECT:END -->";
            if let (Some(s), Some(e)) = (
                existing.find(reflect_start_marker),
                existing.find(reflect_end_marker),
            ) && s < e
            {
                let end_pos = e + reflect_end_marker.len();
                // Check if a REF block already exists after the old block.
                let tail = &existing[end_pos..];
                let already_has_ref = tail.contains(REFLECT_REF_START);
                let mut new_content = String::with_capacity(existing.len());
                new_content.push_str(&existing[..s]);
                if !already_has_ref {
                    new_content.push_str(&ref_block);
                }
                new_content.push_str(&existing[end_pos..]);
                std::fs::write(target, new_content).into_diagnostic()?;
            }

            if !migration_notice_printed {
                migration_notice_printed = true;
            }
        }

        // Write migrated resource file if we found content and no migrated file exists.
        if let Some(content) = first_content
            && !migrated_exists
        {
            std::fs::create_dir_all(&refl_dir).into_diagnostic()?;
            let project_slug = project_name
                .as_deref()
                .map(slug::slugify)
                .unwrap_or_else(|| "project".to_string());
            let migrated_filename = format!("{}-{}-migrated.md", today, project_slug);
            let migrated_path = refl_dir.join(&migrated_filename);
            let front_matter = format!(
                "---\ndate: \"{}\"\nproject: \"{}\"\ntype: reflection-migrated\n---\n\n{}",
                today,
                project_name.as_deref().unwrap_or("unknown"),
                content.trim()
            );
            std::fs::write(&migrated_path, front_matter).into_diagnostic()?;
        }

        if migration_notice_printed {
            println!();
            println!(
                "  {} Migrated WAI:REFLECT block(s) to resource file.",
                "◆".cyan()
            );
        }
    }

    // Gather context.
    println!();
    println!("  {} Gathering context …", "◆".cyan());
    let ctx = gather_reflect_context(&project_root, conversation.as_deref(), &targets)?;

    // Call LLM (or use injected content / agent-mode sentinel path).
    let raw_response = if let Some(content) = inject_content {
        // Agent provided the content directly via --inject-content.
        content
    } else {
        println!("  {} Calling LLM …", "○".dimmed());
        let prompt = build_reflect_prompt(&ctx, &today);
        let raw = call_llm(&project_root, &prompt)?;
        if raw == AGENT_SENTINEL {
            // AgentBackend already printed [AGENT CONTEXT]...[/AGENT CONTEXT] to stdout.
            // The enclosing agent will read the context and generate the REFLECT block.
            // Instruct it to feed the result back via --inject-content.
            println!();
            println!("  {} Agent mode — context sent to agent.", "◆".cyan());
            println!(
                "  {} Once the agent provides the REFLECT content, run:",
                "○".dimmed()
            );
            println!(
                "  {}   wai reflect --inject-content '<content>'",
                "○".dimmed()
            );
            return Ok(());
        }
        raw
    };

    // Extract REFLECT content from LLM response.
    let new_content = extract_reflect_content(&raw_response);

    // --dry-run: show the resource file path that would be written, then exit.
    if dry_run {
        let project_str = project_name.as_deref().unwrap_or("project");
        let would_write = predict_reflect_resource_path(&project_root, project_str);
        println!();
        println!("  {} Dry run — would write:", "○".dimmed());
        println!("  {}", would_write.display());
        println!();
        return Ok(());
    }

    // Write resource file.
    let project_str = project_name.as_deref().unwrap_or("project");
    let resource_path =
        write_reflect_resource(&project_root, project_str, &new_content, ctx.handoff_count)?;

    // Update .reflect-meta for the resolved project.
    if let Some(ref name) = project_name {
        let project_dir = crate::config::projects_dir(&project_root).join(name);
        if project_dir.exists() {
            let existing_meta = read_reflect_meta(&project_dir)?.unwrap_or(ReflectMeta {
                last_reflected: today.clone(),
                session_count: 0,
            });
            let new_meta = ReflectMeta {
                last_reflected: today.clone(),
                session_count: existing_meta.session_count + 1,
            };
            write_reflect_meta(&project_dir, &new_meta)?;
        }
    }

    // Print success with the resource file path.
    println!();
    println!(
        "  {} Wrote {}",
        "✓".green(),
        resource_path.display().to_string().bold()
    );
    println!();

    if save_memories {
        let bullets = extract_top_level_bullets(&new_content);
        if bullets.is_empty() {
            println!(
                "  {} --save-memories: no top-level bullets found in reflection",
                "○".dimmed()
            );
        } else {
            println!(
                "  {} Saving {} bullet(s) to bd memories …",
                "◆".cyan(),
                bullets.len()
            );
            let mut saved = 0u32;
            for bullet in &bullets {
                match store_memory(&project_root, bullet) {
                    Ok(()) => saved += 1,
                    Err(e) => eprintln!("! Could not save memory: {}", e),
                }
            }
            println!("  {} Saved {} memories", "✓".green(), saved);
        }
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    // ── Output target detection tests ─────────────────────────────────────

    #[test]
    fn detect_output_targets_errors_when_neither_file_exists() {
        let dir = tmp();
        let result = detect_output_targets(dir.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn detect_output_targets_auto_selects_existing_file() {
        let dir = tmp();
        fs::write(dir.path().join("CLAUDE.md"), "# Claude").unwrap();
        let targets = detect_output_targets(dir.path(), None).unwrap();
        assert_eq!(targets.len(), 1);
        assert!(targets[0].ends_with("CLAUDE.md"));
    }

    #[test]
    fn detect_output_targets_selects_both_when_both_exist() {
        let dir = tmp();
        fs::write(dir.path().join("CLAUDE.md"), "# Claude").unwrap();
        fs::write(dir.path().join("AGENTS.md"), "# Agents").unwrap();
        let targets = detect_output_targets(dir.path(), None).unwrap();
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn detect_output_targets_respects_explicit_claude() {
        let dir = tmp();
        // No files needed for explicit target.
        let targets = detect_output_targets(dir.path(), Some("claude.md")).unwrap();
        assert_eq!(targets.len(), 1);
        assert!(targets[0].ends_with("CLAUDE.md"));
    }

    #[test]
    fn detect_output_targets_respects_explicit_both() {
        let dir = tmp();
        let targets = detect_output_targets(dir.path(), Some("both")).unwrap();
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn detect_output_targets_errors_on_unknown_value() {
        let dir = tmp();
        let result = detect_output_targets(dir.path(), Some("unknown.md"));
        assert!(result.is_err());
    }

    // ── Reflection prompt construction tests ─────────────────────────────

    fn empty_context() -> ReflectContext {
        ReflectContext {
            conversation: None,
            handoffs: vec![],
            handoff_count: 0,
            secondary: vec![],
            existing_blocks: vec![],
            previous_reflections: vec![],
            memories: None,
        }
    }

    #[test]
    fn build_reflect_prompt_contains_role() {
        let ctx = empty_context();
        let prompt = build_reflect_prompt(&ctx, "2026-02-24");
        assert!(prompt.contains("synthesizing project-specific AI assistant guidance"));
    }

    #[test]
    fn build_reflect_prompt_contains_output_instructions() {
        let ctx = empty_context();
        let prompt = build_reflect_prompt(&ctx, "2026-02-24");
        assert!(prompt.contains("2026-02-24"));
        assert!(prompt.contains("WAI:REFLECT"));
    }

    #[test]
    fn build_reflect_prompt_includes_conversation_when_provided() {
        let ctx = ReflectContext {
            conversation: Some("session transcript here".to_string()),
            handoffs: vec![],
            handoff_count: 0,
            secondary: vec![],
            existing_blocks: vec![],
            previous_reflections: vec![],
            memories: None,
        };
        let prompt = build_reflect_prompt(&ctx, "2026-02-24");
        assert!(prompt.contains("session transcript here"));
        assert!(prompt.contains("Conversation Transcript"));
    }

    #[test]
    fn build_reflect_prompt_includes_handoffs() {
        use context::HandoffEntry;
        let ctx = ReflectContext {
            conversation: None,
            handoffs: vec![HandoffEntry {
                rel_path: ".wai/projects/foo/handoffs/h.md".to_string(),
                content: "handoff notes here".to_string(),
            }],
            handoff_count: 1,
            secondary: vec![],
            existing_blocks: vec![],
            previous_reflections: vec![],
            memories: None,
        };
        let prompt = build_reflect_prompt(&ctx, "2026-02-24");
        assert!(prompt.contains("handoff notes here"));
        assert!(prompt.contains("Handoff Artifacts"));
    }

    #[test]
    fn build_reflect_prompt_includes_existing_blocks() {
        let ctx = ReflectContext {
            conversation: None,
            handoffs: vec![],
            handoff_count: 0,
            secondary: vec![],
            existing_blocks: vec![(PathBuf::from("CLAUDE.md"), "existing guidance".to_string())],
            previous_reflections: vec![],
            memories: None,
        };
        let prompt = build_reflect_prompt(&ctx, "2026-02-24");
        assert!(prompt.contains("existing guidance"));
        assert!(prompt.contains("Already Documented"));
    }

    #[test]
    fn build_reflect_prompt_includes_memories_when_provided() {
        let ctx = ReflectContext {
            memories: Some("- Use fetch_memories for bd integration".to_string()),
            ..empty_context()
        };
        let prompt = build_reflect_prompt(&ctx, "2026-03-04");
        assert!(prompt.contains("Already in Global Memories"));
        assert!(prompt.contains("fetch_memories"));
    }

    #[test]
    fn build_reflect_prompt_escapes_triple_backticks_in_artifacts() {
        let ctx = ReflectContext {
            conversation: Some("some ```code``` here".to_string()),
            handoffs: vec![],
            handoff_count: 0,
            secondary: vec![],
            existing_blocks: vec![],
            previous_reflections: vec![],
            memories: None,
        };
        let prompt = build_reflect_prompt(&ctx, "2026-02-24");
        // Should be escaped to ~~~
        assert!(!prompt.contains("```code```"));
        assert!(prompt.contains("~~~code~~~"));
    }

    // ── REFLECT block extraction tests ────────────────────────────────────

    #[test]
    fn extract_reflect_content_from_markers() {
        let response = "<!-- WAI:REFLECT:START -->\ninner content\n<!-- WAI:REFLECT:END -->";
        assert_eq!(extract_reflect_content(response), "inner content");
    }

    #[test]
    fn extract_reflect_content_strips_markdown_fences() {
        let response = "```\nsome content\n```";
        assert_eq!(extract_reflect_content(response), "some content");
    }

    #[test]
    fn extract_reflect_content_strips_fenced_with_language() {
        let response = "```markdown\nsome content\n```";
        assert_eq!(extract_reflect_content(response), "some content");
    }

    #[test]
    fn extract_reflect_content_passthrough_for_plain_text() {
        let response = "## Project Notes\n- Use TDD";
        assert_eq!(
            extract_reflect_content(response),
            "## Project Notes\n- Use TDD"
        );
    }

    #[test]
    fn extract_reflect_content_prefers_markers_over_fences() {
        let response = "```\n<!-- WAI:REFLECT:START -->\ninner\n<!-- WAI:REFLECT:END -->\n```";
        let result = extract_reflect_content(response);
        assert_eq!(result, "inner");
    }

    // ── Integration test: migration path (6.6) ───────────────────────────────

    /// Set up a minimal workspace in `dir`:
    /// - `.wai/` (satisfies require_project)
    /// - `.wai/projects/<project>/` (enables auto-detection)
    /// - `CLAUDE.md` with an old WAI:REFLECT block
    fn setup_migration_workspace(dir: &TempDir, project: &str) {
        let root = dir.path();
        // Create .wai dir so require_project() can find the root.
        fs::create_dir_all(root.join(".wai")).unwrap();
        // Create the project dir so auto-detection picks it up.
        fs::create_dir_all(root.join(".wai/projects").join(project)).unwrap();
        // Write CLAUDE.md with an old REFLECT block.
        let claude_md = root.join("CLAUDE.md");
        fs::write(
            &claude_md,
            "# Preamble\n\
             <!-- WAI:REFLECT:START -->\n\
             ## Old Patterns\n\
             - old convention\n\
             <!-- WAI:REFLECT:END -->\n\
             # Postamble\n",
        )
        .unwrap();
    }

    #[test]
    #[serial]
    fn integration_test_migration_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        setup_migration_workspace(&dir, "my-project");

        // Save and switch current directory so require_project() finds our workspace.
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = run(ReflectArgs {
            project: None,
            conversation: None,
            output: None,
            dry_run: false,
            yes: true,
            inject_content: Some("# Patterns\ntest content from inject".to_string()),
            verbose: 0,
            save_memories: false,
        });

        // Restore working directory before asserting, so failures don't break
        // other serial tests.
        std::env::set_current_dir(&original_dir).unwrap();

        result.expect("run() should succeed");

        // 1. A resource file was created in .wai/resources/reflections/ with
        //    a filename matching *-migrated.md.
        let refl_dir = crate::config::reflections_dir(dir.path());
        assert!(
            refl_dir.exists(),
            "reflections dir should have been created"
        );
        let migrated_files: Vec<_> = fs::read_dir(&refl_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("-migrated"))
            .collect();
        assert_eq!(
            migrated_files.len(),
            1,
            "expected exactly one *-migrated.md file, found: {:?}",
            migrated_files
                .iter()
                .map(|e| e.file_name())
                .collect::<Vec<_>>()
        );

        // 2. The resource file contains `type: reflection-migrated` in its
        //    YAML front-matter.
        let migrated_content = fs::read_to_string(migrated_files[0].path()).unwrap();
        assert!(
            migrated_content.contains("type: reflection-migrated"),
            "migrated file should have type: reflection-migrated in front-matter, got:\n{}",
            migrated_content
        );

        // 3. CLAUDE.md no longer contains WAI:REFLECT:START (old block replaced).
        let claude_content = fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
        assert!(
            !claude_content.contains("WAI:REFLECT:START"),
            "CLAUDE.md should no longer have WAI:REFLECT:START after migration"
        );

        // 4. CLAUDE.md contains WAI:REFLECT:REF:START (slim replacement block).
        assert!(
            claude_content.contains("WAI:REFLECT:REF:START"),
            "CLAUDE.md should contain WAI:REFLECT:REF:START after migration"
        );
    }

    #[test]
    fn extract_top_level_bullets_returns_top_level_only() {
        let content = "### Conventions\n- Use fetch_memories for bd integration\n  - nested bullet (ignored)\n* Another top-level bullet\n\nSome plain text\n- Third bullet";
        let bullets = extract_top_level_bullets(content);
        assert_eq!(bullets.len(), 3);
        assert!(bullets[0].contains("fetch_memories"));
        assert!(bullets[1].contains("Another top-level"));
        assert!(bullets[2].contains("Third bullet"));
    }

    #[test]
    fn extract_top_level_bullets_truncates_at_60_chars() {
        let long_bullet = format!("- {}", "x".repeat(100));
        let bullets = extract_top_level_bullets(&long_bullet);
        assert_eq!(bullets.len(), 1);
        assert!(bullets[0].chars().count() <= 60);
    }
}
