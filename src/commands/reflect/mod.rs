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
        Some(other) => Err(miette::miette!(
            "Unknown output target '{}'. Use 'claude.md', 'agents.md', or 'both'.",
            other
        )),
        None => {
            let has_claude = claude_md.exists();
            let has_agents = agents_md.exists();
            if !has_claude && !has_agents {
                return Err(miette::miette!(
                    "No CLAUDE.md or AGENTS.md found in '{}'. \
                     Run `wai init` first or create the target file manually.",
                    repo_root.display()
                ));
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

    parts.push(role_intro());
    parts.push(input_hierarchy());
    if let Some(section) = existing_blocks_section(ctx) {
        parts.push(section);
    }
    if let Some(section) = memories_section(ctx) {
        parts.push(section);
    }
    if let Some(section) = conversation_section(ctx) {
        parts.push(section);
    }
    if let Some(section) = handoffs_section(ctx) {
        parts.push(section);
    }
    if let Some(section) = secondary_section(ctx) {
        parts.push(section);
    }
    if let Some(section) = previous_reflections_section(ctx) {
        parts.push(section);
    }
    parts.push(output_instructions(today));

    parts.join("\n")
}

fn role_intro() -> String {
    "You are synthesizing project-specific AI assistant guidance.\n\
     Your goal: read the session context below and extract patterns, conventions, \
     gotchas, and architectural notes that AI assistants should know when working on \
     this project. Focus on information that is NOT already in the 'Already Documented' \
     section below.\n"
        .to_string()
}

fn input_hierarchy() -> String {
    "# Input Hierarchy\n\
     The context below comes from three tiers, ranked by richness:\n\
     1. **Conversation transcript** — raw session detail; failed attempts, surprises, \
        step-by-step struggles (most information-dense)\n\
     2. **Handoff artifacts** — session summaries; intent, next steps, and gotchas\n\
     3. **Research/design/plan artifacts** — explicit decisions and domain knowledge\n\
     When referencing patterns, note the artifact date. If an artifact is older than \
     6 months, flag it as potentially stale.\n"
        .to_string()
}

/// Content already present in the REFLECT block, so the LLM doesn't repeat it.
fn existing_blocks_section(ctx: &ReflectContext) -> Option<String> {
    if ctx.existing_blocks.is_empty() {
        return None;
    }
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
    Some(already)
}

/// Insights already captured as persistent bd memories.
fn memories_section(ctx: &ReflectContext) -> Option<String> {
    ctx.memories.as_ref().map(|memories| {
        format!(
            "# Already in Global Memories\n\
             The following insights are already captured as persistent memories in bd. \
             Do NOT re-derive or repeat them:\n\n{}\n",
            memories
        )
    })
}

fn conversation_section(ctx: &ReflectContext) -> Option<String> {
    ctx.conversation.as_ref().map(|transcript| {
        format!(
            "# Conversation Transcript\n```\n{}\n```\n",
            escape_fences(transcript)
        )
    })
}

fn handoffs_section(ctx: &ReflectContext) -> Option<String> {
    if ctx.handoffs.is_empty() {
        return None;
    }
    let mut section = String::from("# Handoff Artifacts\n");
    for h in &ctx.handoffs {
        section.push_str(&format!(
            "\n## {}\n```\n{}\n```\n",
            h.rel_path,
            escape_fences(&h.content)
        ));
    }
    Some(section)
}

fn secondary_section(ctx: &ReflectContext) -> Option<String> {
    if ctx.secondary.is_empty() {
        return None;
    }
    let mut section = String::from("# Research / Design / Plan Artifacts\n");
    for s in &ctx.secondary {
        section.push_str(&format!(
            "\n## {} ({})\n```\n{}\n```\n",
            s.rel_path,
            s.kind,
            escape_fences(&s.content)
        ));
    }
    Some(section)
}

fn previous_reflections_section(ctx: &ReflectContext) -> Option<String> {
    if ctx.previous_reflections.is_empty() {
        return None;
    }
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
    Some(section)
}

fn output_instructions(today: &str) -> String {
    format!(
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
    )
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

/// CLI surface for `wai reflect` (clap derive); `verbose` comes from the
/// global `-v` counter, not a per-subcommand flag.
#[derive(Debug, Clone, clap::Args)]
pub struct ReflectArgs {
    /// Project name (auto-detected when only one project exists)
    #[arg(short, long)]
    pub project: Option<String>,

    /// Path to a plain-text conversation transcript (highest-priority context)
    #[arg(short, long, value_name = "FILE")]
    pub conversation: Option<PathBuf>,

    /// Output target: claude.md, agents.md, or both (default: auto-detect)
    #[arg(short, long, value_name = "TARGET")]
    pub output: Option<String>,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,

    /// Skip the confirmation prompt and write directly
    #[arg(short, long)]
    pub yes: bool,

    /// Inject pre-generated content directly (skips LLM call).
    /// Used in agent mode: the agent calls `wai reflect --inject-content "..."` after
    /// receiving the context block from an initial `wai reflect` run.
    #[arg(long, value_name = "CONTENT")]
    pub inject_content: Option<String>,

    /// Global verbosity counter, wired from the top-level `-v` flags.
    #[arg(skip)]
    pub verbose: u8,

    /// Store top-level bullet points from the generated reflection as bd memories
    #[arg(long)]
    pub save_memories: bool,
}

pub fn run(args: ReflectArgs) -> Result<()> {
    let project_root = super::require_project()?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Resolve early as Option<String>: unified resolution, but fall back to
    // None rather than erroring — reflect can operate without a project.
    let project_name = super::resolve_project(&project_root, args.project.as_deref())
        .ok()
        .map(|r| r.name);

    // Detect output targets (CLAUDE.md / AGENTS.md).
    let targets = detect_output_targets(&project_root, args.output.as_deref())?;

    migrate_legacy_reflect_blocks(&project_root, &targets, project_name.as_deref(), &today)?;

    // Gather context.
    println!();
    println!("  {} Gathering context …", "◆".cyan());
    let ctx = gather_reflect_context(&project_root, args.conversation.as_deref(), &targets)?;

    // Call LLM (or use injected content / agent-mode sentinel path).
    let Some(raw_response) =
        resolve_raw_response(&project_root, &ctx, args.inject_content, &today)?
    else {
        // Agent mode: context already sent; the agent feeds back via --inject-content.
        return Ok(());
    };

    // Extract REFLECT content from LLM response.
    let new_content = extract_reflect_content(&raw_response);

    // --dry-run: show the resource file path that would be written, then exit.
    if args.dry_run {
        return print_dry_run(&project_root, project_name.as_deref());
    }

    // Write resource file.
    let project_str = project_name.as_deref();
    let resource_path = write_reflect_resource(
        &project_root,
        project_str.unwrap_or("project"),
        &new_content,
        ctx.handoff_count,
    )?;

    // Update .reflect-meta for the resolved project.
    update_reflect_meta(&project_root, project_name.as_deref(), &today)?;

    // Print success with the resource file path.
    println!();
    println!("  {} Wrote {}", "✓".green(), resource_path.display().bold());
    println!();

    if args.save_memories {
        save_reflect_memories(&project_root, &new_content);
    }

    Ok(())
}

/// Migration step (3.1–3.2): scan target files for legacy
/// WAI:REFLECT:START/END blocks; if any exist, migrate once and replace
/// with slim REF blocks.
fn migrate_legacy_reflect_blocks(
    project_root: &Path,
    targets: &[PathBuf],
    project_name: Option<&str>,
    today: &str,
) -> Result<()> {
    let refl_dir = crate::config::reflections_dir(project_root);
    // Check whether a *-migrated.md already exists.
    let migrated_exists = migrated_reflection_exists(&refl_dir);
    let ref_block = format!(
        "{}\n{}{}\n",
        REFLECT_REF_START,
        wai_reflect_ref_content(),
        REFLECT_REF_END
    );

    let mut migrated = false;
    let mut first_content: Option<String> = None;
    for target in targets {
        if has_reflect_block(target) {
            // Extract content from the first file that has the block for migration.
            if first_content.is_none() {
                first_content = read_reflect_block(target);
            }
            replace_legacy_reflect_block(target, &ref_block)?;
            migrated = true;
        }
    }

    // Write migrated resource file if we found content and no migrated file exists.
    if let Some(content) = first_content
        && !migrated_exists
    {
        write_migrated_reflection(&refl_dir, &content, project_name, today)?;
    }

    if migrated {
        println!();
        println!(
            "  {} Migrated WAI:REFLECT block(s) to resource file.",
            "◆".cyan()
        );
    }
    Ok(())
}

/// Whether a `*-migrated.md` reflection resource already exists.
fn migrated_reflection_exists(refl_dir: &Path) -> bool {
    if !refl_dir.exists() {
        return false;
    }
    std::fs::read_dir(refl_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .any(|e| e.file_name().to_string_lossy().contains("-migrated"))
        })
        .unwrap_or(false)
}

/// Replace a legacy WAI:REFLECT:START/END block in `target` with the slim
/// REF block (idempotent: skips if a REF block already follows).
fn replace_legacy_reflect_block(target: &Path, ref_block: &str) -> Result<()> {
    let existing = std::fs::read_to_string(target).into_diagnostic()?;
    let reflect_start_marker = "<!-- WAI:REFLECT:START -->";
    let reflect_end_marker = "<!-- WAI:REFLECT:END -->";
    if let (Some(s), Some(e)) = (
        existing.find(reflect_start_marker),
        existing.find(reflect_end_marker),
    ) && s < e
    {
        let end_pos = e + reflect_end_marker.len();
        // Check if a REF block already exists after the old block.
        let already_has_ref = existing[end_pos..].contains(REFLECT_REF_START);
        let mut new_content = String::with_capacity(existing.len());
        new_content.push_str(&existing[..s]);
        if !already_has_ref {
            new_content.push_str(ref_block);
        }
        new_content.push_str(&existing[end_pos..]);
        std::fs::write(target, new_content).into_diagnostic()?;
    }
    Ok(())
}

/// Persist the migrated legacy REFLECT content as a dated resource file.
fn write_migrated_reflection(
    refl_dir: &Path,
    content: &str,
    project_name: Option<&str>,
    today: &str,
) -> Result<()> {
    std::fs::create_dir_all(refl_dir).into_diagnostic()?;
    let project_slug = project_name
        .map(slug::slugify)
        .unwrap_or_else(|| "project".to_string());
    let migrated_filename = format!("{}-{}-migrated.md", today, project_slug);
    let migrated_path = refl_dir.join(&migrated_filename);
    let front_matter = format!(
        "---\ndate: \"{}\"\nproject: \"{}\"\ntype: reflection-migrated\n---\n\n{}",
        today,
        project_name.unwrap_or("unknown"),
        content.trim()
    );
    std::fs::write(&migrated_path, front_matter).into_diagnostic()?;
    Ok(())
}

/// Call the LLM (or honor --inject-content directly). `None` means the
/// agent-mode sentinel path fired: the context was printed for the enclosing
/// agent, which will feed the REFLECT content back via --inject-content.
fn resolve_raw_response(
    project_root: &Path,
    ctx: &ReflectContext,
    inject_content: Option<String>,
    today: &str,
) -> Result<Option<String>> {
    if let Some(content) = inject_content {
        // Agent provided the content directly via --inject-content.
        return Ok(Some(content));
    }
    println!("  {} Calling LLM …", "○".dimmed());
    let prompt = build_reflect_prompt(ctx, today);
    let raw = call_llm(project_root, &prompt)?;
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
        return Ok(None);
    }
    Ok(Some(raw))
}

/// --dry-run: show the resource file path that would be written.
fn print_dry_run(project_root: &Path, project_name: Option<&str>) -> Result<()> {
    let project_str = project_name.unwrap_or("project");
    let would_write = predict_reflect_resource_path(project_root, project_str);
    println!();
    println!("  {} Dry run — would write:", "○".dimmed());
    println!("  {}", would_write.display());
    println!();
    Ok(())
}

/// Update `.reflect-meta` (last_reflected date, session count) for the
/// resolved project.
fn update_reflect_meta(project_root: &Path, project_name: Option<&str>, today: &str) -> Result<()> {
    if let Some(name) = project_name {
        let project_dir = crate::config::projects_dir(project_root).join(name);
        if project_dir.exists() {
            let existing_meta = read_reflect_meta(&project_dir)?.unwrap_or(ReflectMeta {
                last_reflected: today.to_string(),
                session_count: 0,
            });
            let new_meta = ReflectMeta {
                last_reflected: today.to_string(),
                session_count: existing_meta.session_count + 1,
            };
            write_reflect_meta(&project_dir, &new_meta)?;
        }
    }
    Ok(())
}

/// Save top-level bullets from the reflection as bd memories.
fn save_reflect_memories(project_root: &Path, new_content: &str) {
    let bullets = extract_top_level_bullets(new_content);
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
            match store_memory(project_root, bullet) {
                Ok(()) => saved += 1,
                Err(e) => eprintln!("! Could not save memory: {}", e),
            }
        }
        println!("  {} Saved {} memories", "✓".green(), saved);
    }
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

        let migrated_files = find_migrated_files(&dir);
        assert_migrated_resource(&migrated_files);
        assert_migration_replaced_claude_block(&dir);
    }

    /// Exactly one `*-migrated.md` in .wai/resources/reflections/.
    fn find_migrated_files(dir: &tempfile::TempDir) -> Vec<std::fs::DirEntry> {
        let refl_dir = crate::config::reflections_dir(dir.path());
        let files: Vec<_> = fs::read_dir(&refl_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("-migrated"))
            .collect();
        assert_eq!(
            files.len(),
            1,
            "expected exactly one *-migrated.md file, found: {:?}",
            files.iter().map(|e| e.file_name()).collect::<Vec<_>>()
        );
        files
    }

    /// Resource file carries `type: reflection-migrated` front-matter.
    fn assert_migrated_resource(migrated_files: &[std::fs::DirEntry]) {
        let migrated_content = fs::read_to_string(migrated_files[0].path()).unwrap();
        assert!(
            migrated_content.contains("type: reflection-migrated"),
            "migrated file should have type: reflection-migrated in front-matter, got:\n{}",
            migrated_content
        );
    }

    /// CLAUDE.md lost the legacy WAI:REFLECT block and gained the slim REF block.
    fn assert_migration_replaced_claude_block(dir: &tempfile::TempDir) {
        let claude_content = fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
        assert!(
            !claude_content.contains("WAI:REFLECT:START"),
            "CLAUDE.md should no longer have WAI:REFLECT:START after migration"
        );
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
