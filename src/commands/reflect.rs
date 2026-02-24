use std::path::{Path, PathBuf};
use std::time::SystemTime;

use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::config::{ProjectConfig, wai_dir};
use crate::llm::detect_backend;
use crate::managed_block::{inject_reflect_block, read_reflect_block};

// ── Reflect metadata ─────────────────────────────────────────────────────────

/// Metadata stored in `.wai/projects/<project>/.reflect-meta`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReflectMeta {
    pub last_reflected: String,
    pub session_count: u32,
}

/// Read `.reflect-meta` TOML from `project_dir`. Returns `None` if the file
/// does not exist; returns an error if the file is malformed.
pub fn read_reflect_meta(project_dir: &Path) -> Result<Option<ReflectMeta>> {
    let path = project_dir.join(".reflect-meta");
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path).into_diagnostic()?;
    let table: toml::Table = raw.parse().into_diagnostic()?;

    let last_reflected = table
        .get("last_reflected")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let session_count = table
        .get("session_count")
        .and_then(|v| v.as_integer())
        .unwrap_or(0) as u32;

    Ok(Some(ReflectMeta {
        last_reflected,
        session_count,
    }))
}

/// Write `.reflect-meta` TOML to `project_dir`, overwriting any existing file.
pub fn write_reflect_meta(project_dir: &Path, meta: &ReflectMeta) -> Result<()> {
    let path = project_dir.join(".reflect-meta");
    let content = format!(
        "last_reflected = \"{}\"\nsession_count = {}\n",
        meta.last_reflected, meta.session_count
    );
    std::fs::write(&path, content).into_diagnostic()?;
    Ok(())
}

// ── Output target detection ──────────────────────────────────────────────────

/// Which files `wai reflect` should update.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputTarget {
    ClaudeMd,
    AgentsMd,
    Both,
}

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

// ── Context gathering ─────────────────────────────────────────────────────────

/// Budget allocations for the three context tiers.
const CONVERSATION_BUDGET: usize = 30_000;
const HANDOFF_BUDGET: usize = 40_000;
const SECONDARY_BUDGET: usize = 30_000;

/// All context gathered before calling the LLM.
#[derive(Debug)]
pub struct ReflectContext {
    /// Conversation transcript content (truncated to budget from the top).
    pub conversation: Option<String>,
    /// Handoff artifacts, newest-first, concatenated up to budget.
    pub handoffs: Vec<HandoffEntry>,
    /// Research/design/plan artifacts up to budget.
    pub secondary: Vec<SecondaryEntry>,
    /// Existing REFLECT block content per target file (to avoid repeating).
    pub existing_blocks: Vec<(PathBuf, String)>,
}

#[derive(Debug)]
pub struct HandoffEntry {
    pub rel_path: String,
    pub content: String,
}

#[derive(Debug)]
pub struct SecondaryEntry {
    pub rel_path: String,
    pub kind: &'static str,
    pub content: String,
}

/// Read the conversation transcript from `path`, truncating to `budget` chars
/// by removing the oldest content from the top.
pub fn read_conversation(path: &Path, budget: usize) -> Result<String> {
    let raw = std::fs::read_to_string(path).into_diagnostic()?;
    Ok(truncate_from_top(&raw, budget))
}

/// Truncate text to `max_chars` by discarding characters from the front
/// (oldest content) so the most recent content is preserved.
pub fn truncate_from_top(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    let overflow = text.len() - max_chars;
    // Advance past the overflow point to the next newline so we don't split mid-line.
    let cut_at = text[overflow..]
        .find('\n')
        .map(|i| overflow + i + 1)
        .unwrap_or(overflow);
    text[cut_at..].to_string()
}

/// Read all handoff artifacts from `.wai/projects/*/handoffs/*.md`, sorted by
/// mtime descending (newest first), concatenated up to `budget` chars.
pub fn read_handoffs(project_root: &Path, budget: usize) -> Vec<HandoffEntry> {
    let wai = wai_dir(project_root);
    let mut entries: Vec<(SystemTime, String, String)> = Vec::new(); // (mtime, rel_path, content)

    let projects_dir = wai.join("projects");
    if !projects_dir.exists() {
        return Vec::new();
    }

    for entry in WalkDir::new(&projects_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Must be in a handoffs/ subdirectory and have .md extension.
        if !path
            .to_string_lossy()
            .contains("/handoffs/")
        {
            continue;
        }
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let rel_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        entries.push((mtime, rel_path, content));
    }

    // Sort newest-first.
    entries.sort_by(|a, b| b.0.cmp(&a.0));

    // Fill up to budget.
    let mut result = Vec::new();
    let mut used = 0usize;
    for (_, rel_path, content) in entries {
        if used >= budget {
            break;
        }
        let take = content.len().min(budget - used);
        let trimmed = content[..take].to_string();
        used += trimmed.len();
        result.push(HandoffEntry {
            rel_path,
            content: trimmed,
        });
    }
    result
}

/// Read secondary artifacts (research, design, plan) from the `.wai/` tree,
/// sorted newest-first, up to `budget` chars.
pub fn read_secondary_artifacts(project_root: &Path, budget: usize) -> Vec<SecondaryEntry> {
    let wai = wai_dir(project_root);
    let mut entries: Vec<(SystemTime, String, &'static str, String)> = Vec::new();

    for entry in WalkDir::new(&wai)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        // Skip hidden files.
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        let path_str = path.to_string_lossy();
        let kind: Option<&'static str> = if path_str.contains("/research/") {
            Some("research")
        } else if path_str.contains("/designs/") {
            Some("design")
        } else if path_str.contains("/plans/") {
            Some("plan")
        } else {
            None
        };
        let kind = match kind {
            Some(k) => k,
            None => continue,
        };

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let rel_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        entries.push((mtime, rel_path, kind, content));
    }

    entries.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = Vec::new();
    let mut used = 0usize;
    for (_, rel_path, kind, content) in entries {
        if used >= budget {
            break;
        }
        let take = content.len().min(budget - used);
        let trimmed = content[..take].to_string();
        used += trimmed.len();
        result.push(SecondaryEntry {
            rel_path,
            kind,
            content: trimmed,
        });
    }
    result
}

/// Gather all context for a reflect run.
pub fn gather_reflect_context(
    project_root: &Path,
    conversation_path: Option<&Path>,
    output_targets: &[PathBuf],
) -> Result<ReflectContext> {
    let conversation = match conversation_path {
        Some(p) => Some(read_conversation(p, CONVERSATION_BUDGET)?),
        None => None,
    };

    let handoffs = read_handoffs(project_root, HANDOFF_BUDGET);
    let secondary = read_secondary_artifacts(project_root, SECONDARY_BUDGET);

    // Read existing REFLECT blocks from each target so LLM can avoid repeating them.
    let existing_blocks = output_targets
        .iter()
        .filter_map(|p| {
            read_reflect_block(p).map(|block| (p.clone(), block))
        })
        .collect();

    Ok(ReflectContext {
        conversation,
        handoffs,
        secondary,
        existing_blocks,
    })
}

// ── Close nudge helpers (Phase 5) ────────────────────────────────────────────

/// Count handoff `.md` files in `project_handoffs_dir` whose mtime is
/// after `since_date` (format: `"YYYY-MM-DD"`).
///
/// If `since_date` is empty or unparsable, all handoffs are counted.
pub fn count_handoffs_since(project_handoffs_dir: &Path, since_date: &str) -> u32 {
    let cutoff: Option<SystemTime> = if since_date.is_empty() {
        None
    } else {
        parse_date_to_system_time(since_date)
    };

    if !project_handoffs_dir.exists() {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(project_handoffs_dir) else {
        return 0;
    };
    let mut count = 0u32;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        match cutoff {
            None => count += 1,
            Some(cutoff_time) => {
                let mtime = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                if mtime > cutoff_time {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Parse a `YYYY-MM-DD` date string into a `SystemTime` at midnight UTC.
///
/// Returns `None` if the string is not a valid date.
pub fn parse_date_to_system_time(date: &str) -> Option<SystemTime> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    use std::time::UNIX_EPOCH;

    // Compute days since Unix epoch (1970-01-01).
    let days = days_since_epoch(year, month, day)?;
    UNIX_EPOCH.checked_add(std::time::Duration::from_secs(days * 86400))
}

/// Compute the number of days from 1970-01-01 to the given date.
fn days_since_epoch(year: i32, month: u32, day: u32) -> Option<u64> {
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }
    // Use the Julian Day Number algorithm.
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    let jdn = day as i32 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    // Julian Day Number of 1970-01-01
    const JDN_EPOCH: i32 = 2440588;
    let days = jdn - JDN_EPOCH;
    if days < 0 { None } else { Some(days as u64) }
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
/// Reuses `WhyConfig` and `detect_backend` from `src/llm.rs` (task 3.1).
/// Returns an error if no backend is available.
///
/// If the env var `WAI_REFLECT_MOCK_RESPONSE` is set, its value is returned
/// directly without calling any LLM (used for integration testing only).
pub fn call_llm(project_root: &Path, prompt: &str) -> Result<String> {
    if let Ok(mock) = std::env::var("WAI_REFLECT_MOCK_RESPONSE") {
        return Ok(mock);
    }

    let why_cfg = ProjectConfig::load(project_root)
        .ok()
        .and_then(|c| c.why)
        .unwrap_or_default();

    let backend = detect_backend(&why_cfg).ok_or_else(|| {
        miette::miette!(
            "No LLM available. Configure one in .wai/config.toml under [why], \
             or set ANTHROPIC_API_KEY."
        )
    })?;

    let _ = prompt;
    backend
        .complete(prompt)
        .map_err(|e| miette::miette!("LLM error: {}", e))
}

// ── Diff display (4.1) ────────────────────────────────────────────────────────

/// A line in a diff output.
#[derive(Debug, PartialEq, Clone)]
pub enum DiffLine {
    Added(String),
    Removed(String),
    Context(String),
}

/// Compute a simple line-level diff between `old` and `new`.
///
/// Returns `None` if the content is identical (no change needed).
pub fn compute_diff(old: Option<&str>, new: &str) -> Option<Vec<DiffLine>> {
    let old_str = old.unwrap_or("");
    if old_str.trim() == new.trim() {
        return None;
    }

    let old_lines: Vec<&str> = old_str.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Build LCS table (simple O(n*m) approach; blocks are small).
    let m = old_lines.len();
    let n = new_lines.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old_lines[i - 1] == new_lines[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to build the diff.
    let mut result = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            result.push(DiffLine::Context(old_lines[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            result.push(DiffLine::Added(new_lines[j - 1].to_string()));
            j -= 1;
        } else {
            result.push(DiffLine::Removed(old_lines[i - 1].to_string()));
            i -= 1;
        }
    }
    result.reverse();
    Some(result)
}

/// Print diff lines to stdout with color.
pub fn print_diff(diff: &[DiffLine]) {
    for line in diff {
        match line {
            DiffLine::Added(s) => println!("{}", format!("+ {}", s).green()),
            DiffLine::Removed(s) => println!("{}", format!("- {}", s).red()),
            DiffLine::Context(s) => println!("  {}", s),
        }
    }
}

// ── Command handler ───────────────────────────────────────────────────────────

pub fn run(
    project: Option<String>,
    conversation: Option<PathBuf>,
    output: Option<String>,
    dry_run: bool,
    yes: bool,
    _verbose: u8,
) -> Result<()> {
    use crate::context::current_context;

    let project_root = super::require_project()?;
    let context = current_context();

    // 2.4: Detect output targets.
    let targets = detect_output_targets(&project_root, output.as_deref())?;

    // 2.1–2.5: Gather context.
    println!();
    println!(
        "  {} Gathering context …",
        "◆".cyan()
    );
    let ctx = gather_reflect_context(&project_root, conversation.as_deref(), &targets)?;

    // 3.1–3.3: Call LLM.
    println!(
        "  {} Calling LLM …",
        "○".dimmed()
    );

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let prompt = build_reflect_prompt(&ctx, &today);

    let raw_response = call_llm(&project_root, &prompt)?;

    // 3.4: Extract REFLECT content.
    let new_content = extract_reflect_content(&raw_response);

    // 4.1: Compute diffs per target.
    let mut target_diffs: Vec<(PathBuf, Option<Vec<DiffLine>>)> = Vec::new();
    for target in &targets {
        let existing = read_reflect_block(target);
        let diff = compute_diff(existing.as_deref(), &new_content);
        target_diffs.push((target.clone(), diff));
    }

    // 4.3: Handle empty diff case (all targets unchanged).
    let all_unchanged = target_diffs.iter().all(|(_, d)| d.is_none());
    if all_unchanged {
        println!();
        println!(
            "  {} REFLECT block is already up to date.",
            "○".dimmed()
        );
        return Ok(());
    }

    // Show diff.
    println!();
    println!("  {} Proposed changes:", "◆".cyan().bold());
    for (target, diff_opt) in &target_diffs {
        let filename = target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        if let Some(diff) = diff_opt {
            println!();
            println!("  {}", filename.bold());
            print_diff(diff);
        } else {
            println!();
            println!(
                "  {} {} — no changes",
                "○".dimmed(),
                filename
            );
        }
    }

    // 4.2: --dry-run exits here.
    if dry_run {
        println!();
        println!(
            "  {} Dry run — no files written.",
            "○".dimmed()
        );
        return Ok(());
    }

    // 4.4–4.5: Confirm or auto-confirm with --yes.
    let file_list: Vec<&str> = targets
        .iter()
        .filter_map(|t| t.file_name().and_then(|n| n.to_str()))
        .collect();
    let confirm_msg = format!("Write to {}?", file_list.join(" and "));

    let confirmed = if yes || context.yes || context.no_input {
        true
    } else {
        cliclack::confirm(confirm_msg)
            .interact()
            .into_diagnostic()?
    };

    if !confirmed {
        println!();
        println!("  {} Cancelled.", "○".dimmed());
        return Ok(());
    }

    // 4.6: Inject REFLECT block into each target.
    let mut written = Vec::new();
    for (target, diff_opt) in &target_diffs {
        if diff_opt.is_some() {
            inject_reflect_block(target, &new_content)
                .into_diagnostic()?;
            written.push(target.clone());
        }
    }

    // 4.7: Write .reflect-meta.
    if let Some(project_name) = project.as_deref() {
        let project_dir = crate::config::projects_dir(&project_root).join(project_name);
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
    } else {
        // Auto-detect the single project if only one exists.
        let projects_dir = crate::config::projects_dir(&project_root);
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            let projects: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            if projects.len() == 1 {
                let project_dir = projects[0].path();
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
    }

    // 4.8: Print success message.
    println!();
    for path in &written {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        println!(
            "  {} Updated {}",
            "✓".green(),
            filename.bold()
        );
    }
    println!();

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    // ── ReflectMeta tests ──────────────────────────────────────────────────

    #[test]
    fn read_reflect_meta_returns_none_when_file_missing() {
        let dir = tmp();
        let result = read_reflect_meta(dir.path()).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn write_then_read_reflect_meta_round_trips() {
        let dir = tmp();
        let meta = ReflectMeta {
            last_reflected: "2026-02-24".to_string(),
            session_count: 7,
        };
        write_reflect_meta(dir.path(), &meta).unwrap();
        let read_back = read_reflect_meta(dir.path()).unwrap().expect("should exist");
        assert_eq!(read_back.last_reflected, "2026-02-24");
        assert_eq!(read_back.session_count, 7);
    }

    #[test]
    fn write_reflect_meta_creates_valid_toml_file() {
        let dir = tmp();
        let meta = ReflectMeta {
            last_reflected: "2026-01-01".to_string(),
            session_count: 3,
        };
        write_reflect_meta(dir.path(), &meta).unwrap();
        let raw = fs::read_to_string(dir.path().join(".reflect-meta")).unwrap();
        assert!(raw.contains("last_reflected = \"2026-01-01\""));
        assert!(raw.contains("session_count = 3"));
    }

    #[test]
    fn write_reflect_meta_overwrites_existing() {
        let dir = tmp();
        let first = ReflectMeta {
            last_reflected: "2026-01-01".to_string(),
            session_count: 1,
        };
        write_reflect_meta(dir.path(), &first).unwrap();
        let second = ReflectMeta {
            last_reflected: "2026-02-24".to_string(),
            session_count: 12,
        };
        write_reflect_meta(dir.path(), &second).unwrap();
        let read_back = read_reflect_meta(dir.path()).unwrap().unwrap();
        assert_eq!(read_back.last_reflected, "2026-02-24");
        assert_eq!(read_back.session_count, 12);
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

    // ── Conversation transcript reader tests ──────────────────────────────

    #[test]
    fn truncate_from_top_returns_full_when_within_budget() {
        let text = "short text";
        assert_eq!(truncate_from_top(text, 1000), text);
    }

    #[test]
    fn truncate_from_top_preserves_recent_content() {
        let text = "old line\nnewer line\nnewest line\n";
        let truncated = truncate_from_top(text, 20);
        assert!(truncated.contains("newest line"));
        assert!(!truncated.contains("old line"));
    }

    #[test]
    fn read_conversation_reads_file_within_budget() {
        let dir = tmp();
        let path = dir.path().join("transcript.txt");
        fs::write(&path, "session content here").unwrap();
        let content = read_conversation(&path, 30_000).unwrap();
        assert_eq!(content, "session content here");
    }

    #[test]
    fn read_conversation_truncates_large_file() {
        let dir = tmp();
        let path = dir.path().join("transcript.txt");
        // Write old block (a's) then new block (b's); budget fits only the new block.
        // 100 a's + \n + 100 b's + \n = 202 chars. Budget=110 cuts off the a's.
        let big = "a".repeat(100) + "\n" + &"b".repeat(100) + "\n";
        fs::write(&path, &big).unwrap();
        let content = read_conversation(&path, 110).unwrap();
        assert!(content.trim_end().ends_with('b'));
        assert!(!content.contains('a'));
    }

    // ── Handoff artifact reader tests ─────────────────────────────────────

    fn make_wai_handoff(root: &Path, project: &str, filename: &str, content: &str) {
        let dir = root.join(".wai/projects").join(project).join("handoffs");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn read_handoffs_returns_empty_when_no_wai_dir() {
        let dir = tmp();
        let handoffs = read_handoffs(dir.path(), HANDOFF_BUDGET);
        assert!(handoffs.is_empty());
    }

    #[test]
    fn read_handoffs_collects_handoff_files() {
        let dir = tmp();
        make_wai_handoff(dir.path(), "my-project", "2026-02-24-session.md", "# Session\nContent");
        let handoffs = read_handoffs(dir.path(), HANDOFF_BUDGET);
        assert_eq!(handoffs.len(), 1);
        assert!(handoffs[0].content.contains("Content"));
    }

    #[test]
    fn read_handoffs_respects_budget() {
        let dir = tmp();
        make_wai_handoff(dir.path(), "proj", "h1.md", &"x".repeat(30_000));
        make_wai_handoff(dir.path(), "proj", "h2.md", &"y".repeat(30_000));
        let handoffs = read_handoffs(dir.path(), 40_000);
        let total: usize = handoffs.iter().map(|h| h.content.len()).sum();
        assert!(total <= 40_000);
    }

    // ── Close nudge tests (Phase 5) ───────────────────────────────────────

    #[test]
    fn parse_date_to_system_time_known_date() {
        // 1970-01-01 should be epoch.
        let t = parse_date_to_system_time("1970-01-01").unwrap();
        assert_eq!(t, std::time::UNIX_EPOCH);
    }

    #[test]
    fn parse_date_to_system_time_returns_none_for_bad_date() {
        assert!(parse_date_to_system_time("not-a-date").is_none());
        assert!(parse_date_to_system_time("2026-13-01").is_none());
    }

    #[test]
    fn count_handoffs_since_returns_zero_when_dir_missing() {
        let dir = tmp();
        let handoffs = dir.path().join("handoffs");
        assert_eq!(count_handoffs_since(&handoffs, ""), 0);
    }

    #[test]
    fn count_handoffs_since_counts_all_when_no_date() {
        let dir = tmp();
        let handoffs = dir.path().join("handoffs");
        fs::create_dir_all(&handoffs).unwrap();
        fs::write(handoffs.join("h1.md"), "content").unwrap();
        fs::write(handoffs.join("h2.md"), "content").unwrap();
        assert_eq!(count_handoffs_since(&handoffs, ""), 2);
    }

    #[test]
    fn count_handoffs_since_skips_non_md_files() {
        let dir = tmp();
        let handoffs = dir.path().join("handoffs");
        fs::create_dir_all(&handoffs).unwrap();
        fs::write(handoffs.join("h1.md"), "content").unwrap();
        fs::write(handoffs.join("notes.txt"), "ignored").unwrap();
        assert_eq!(count_handoffs_since(&handoffs, ""), 1);
    }

    #[test]
    fn count_handoffs_since_excludes_old_files_by_date() {
        let dir = tmp();
        let handoffs = dir.path().join("handoffs");
        fs::create_dir_all(&handoffs).unwrap();
        // Write a file then immediately use a far-future cutoff date.
        fs::write(handoffs.join("h1.md"), "content").unwrap();
        // 9999-12-31 is far in the future, so the file's mtime is before it.
        assert_eq!(count_handoffs_since(&handoffs, "9999-12-31"), 0);
    }

    // ── Diff computation tests (4.1) ─────────────────────────────────────

    #[test]
    fn compute_diff_returns_none_when_identical() {
        assert_eq!(compute_diff(Some("same content"), "same content"), None);
    }

    #[test]
    fn compute_diff_returns_none_when_only_whitespace_differs() {
        assert_eq!(compute_diff(Some("content\n"), "content"), None);
    }

    #[test]
    fn compute_diff_returns_additions_when_no_old() {
        let diff = compute_diff(None, "new line").unwrap();
        assert!(diff.iter().any(|l| matches!(l, DiffLine::Added(_))));
    }

    #[test]
    fn compute_diff_marks_new_lines_as_added() {
        let diff = compute_diff(Some(""), "added line").unwrap();
        assert!(diff.iter().any(|l| matches!(l, DiffLine::Added(s) if s == "added line")));
    }

    #[test]
    fn compute_diff_marks_removed_lines_as_removed() {
        let diff = compute_diff(Some("removed line"), "").unwrap();
        assert!(diff.iter().any(|l| matches!(l, DiffLine::Removed(s) if s == "removed line")));
    }

    #[test]
    fn compute_diff_preserves_context_lines() {
        let old = "same\nchanged\nsame2";
        let new = "same\nnew content\nsame2";
        let diff = compute_diff(Some(old), new).unwrap();
        assert!(diff.iter().any(|l| matches!(l, DiffLine::Context(s) if s == "same")));
        assert!(diff.iter().any(|l| matches!(l, DiffLine::Context(s) if s == "same2")));
        assert!(diff.iter().any(|l| matches!(l, DiffLine::Added(s) if s == "new content")));
        assert!(diff.iter().any(|l| matches!(l, DiffLine::Removed(s) if s == "changed")));
    }

    // ── Reflection prompt construction tests ─────────────────────────────

    fn empty_context() -> ReflectContext {
        ReflectContext {
            conversation: None,
            handoffs: vec![],
            secondary: vec![],
            existing_blocks: vec![],
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
            secondary: vec![],
            existing_blocks: vec![],
        };
        let prompt = build_reflect_prompt(&ctx, "2026-02-24");
        assert!(prompt.contains("session transcript here"));
        assert!(prompt.contains("Conversation Transcript"));
    }

    #[test]
    fn build_reflect_prompt_includes_handoffs() {
        let ctx = ReflectContext {
            conversation: None,
            handoffs: vec![HandoffEntry {
                rel_path: ".wai/projects/foo/handoffs/h.md".to_string(),
                content: "handoff notes here".to_string(),
            }],
            secondary: vec![],
            existing_blocks: vec![],
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
            secondary: vec![],
            existing_blocks: vec![(
                PathBuf::from("CLAUDE.md"),
                "existing guidance".to_string(),
            )],
        };
        let prompt = build_reflect_prompt(&ctx, "2026-02-24");
        assert!(prompt.contains("existing guidance"));
        assert!(prompt.contains("Already Documented"));
    }

    #[test]
    fn build_reflect_prompt_escapes_triple_backticks_in_artifacts() {
        let ctx = ReflectContext {
            conversation: Some("some ```code``` here".to_string()),
            handoffs: vec![],
            secondary: vec![],
            existing_blocks: vec![],
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
        assert_eq!(extract_reflect_content(response), "## Project Notes\n- Use TDD");
    }

    #[test]
    fn extract_reflect_content_prefers_markers_over_fences() {
        let response = "```\n<!-- WAI:REFLECT:START -->\ninner\n<!-- WAI:REFLECT:END -->\n```";
        let result = extract_reflect_content(response);
        assert_eq!(result, "inner");
    }

    // ── Secondary artifact reader tests ───────────────────────────────────

    fn make_wai_artifact(root: &Path, project: &str, kind: &str, filename: &str, content: &str) {
        let dir = root.join(".wai/projects").join(project).join(kind);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn read_secondary_artifacts_collects_research_design_plan() {
        let dir = tmp();
        make_wai_artifact(dir.path(), "proj", "research", "r.md", "research content");
        make_wai_artifact(dir.path(), "proj", "designs", "d.md", "design content");
        make_wai_artifact(dir.path(), "proj", "plans", "p.md", "plan content");
        let artifacts = read_secondary_artifacts(dir.path(), SECONDARY_BUDGET);
        assert_eq!(artifacts.len(), 3);
        let kinds: Vec<&str> = artifacts.iter().map(|a| a.kind).collect();
        assert!(kinds.contains(&"research"));
        assert!(kinds.contains(&"design"));
        assert!(kinds.contains(&"plan"));
    }

    #[test]
    fn read_secondary_artifacts_skips_handoffs() {
        let dir = tmp();
        make_wai_handoff(dir.path(), "proj", "h.md", "handoff content");
        make_wai_artifact(dir.path(), "proj", "research", "r.md", "research content");
        let artifacts = read_secondary_artifacts(dir.path(), SECONDARY_BUDGET);
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].kind, "research");
    }

    #[test]
    fn read_secondary_artifacts_respects_budget() {
        let dir = tmp();
        make_wai_artifact(dir.path(), "proj", "research", "r1.md", &"a".repeat(20_000));
        make_wai_artifact(dir.path(), "proj", "research", "r2.md", &"b".repeat(20_000));
        let artifacts = read_secondary_artifacts(dir.path(), 25_000);
        let total: usize = artifacts.iter().map(|a| a.content.len()).sum();
        assert!(total <= 25_000);
    }
}
