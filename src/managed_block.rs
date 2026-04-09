use std::path::Path;

const WAI_START: &str = "<!-- WAI:START -->";
const WAI_END: &str = "<!-- WAI:END -->";

/// Info about an installed pipeline with metadata, for managed block generation.
#[derive(Debug, Clone)]
pub struct InstalledPipeline {
    pub name: String,
    pub description: String,
    pub when: String,
    pub step_count: usize,
}

pub fn wai_block_content(
    detected_plugins: &[&str],
    installed_skills: &[&str],
    installed_pipelines: &[InstalledPipeline],
) -> String {
    let has_beads = detected_plugins.contains(&"beads");
    let has_openspec = detected_plugins.contains(&"openspec");
    let has_companions = has_beads || has_openspec;
    let has_ro5 = installed_skills
        .iter()
        .any(|s| *s == "ro5" || *s == "rule-of-5" || *s == "rule-of-5-universal");

    let mut block = String::new();
    block.push_str(WAI_START);
    block.push('\n');

    // Tool Landscape (always present)
    block.push_str(
        "# Workflow Tools\n\
         \n\
         This project uses **wai** to track the *why* behind decisions — research,\n\
         reasoning, and design choices that shaped the code. Run `wai status` first\n\
         to orient yourself.\n",
    );

    if has_companions {
        block.push_str(
            "\n\
             Detected workflow tools:\n\
             - **wai** — research, reasoning, and design decisions\n",
        );
        if has_beads {
            block.push_str(
                "- **beads** — issue tracking (tasks, bugs, dependencies). \
                 CLI command: **`bd`** (not `beads`)\n",
            );
        }
        if has_openspec {
            block.push_str(
                "- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)\n",
            );
        }
        block.push_str(
            "\n\
             > **CRITICAL**: Apply TDD and Tidy First throughout — not just when writing code:\n\
             > - **Planning/task creation**: each ticket should map to a red→green→refactor cycle; \
             refactoring tasks must be separate tickets from feature tasks.\n\
             > - **Design**: define the test shape (inputs/outputs) before designing the implementation.\n\
             > - **Implementation**: write the failing test first, then make it pass, then tidy in a separate commit.\n\
             \n\
             > **When beginning research or creating a ticket**: run `wai search \"<topic>\"` \
             to check for existing patterns before writing new content.\n",
        );
    }
    if has_ro5 {
        block.push_str(
            "> **Ro5**: The Rule of 5 skill is installed. Run `/ro5` after key phase transitions \
             — implement, research, design — for iterative quality review.\n",
        );
    }

    // When to Use What (only when companion tools detected)
    if has_companions {
        block.push_str(
            "\n\
             ## When to Use What\n\
             \n\
             | Need | Tool | Example |\n\
             |------|------|---------|\n\
             | Record reasoning/research | wai | `wai add research \"findings\"` |\n\
             | Capture design decisions | wai | `wai add design \"architecture choice\"` |\n\
             | Session context transfer | wai | `wai handoff create <project>` |\n",
        );
        if has_beads {
            block.push_str(
                "| Track work items/bugs | `bd` | `bd create --title=\"...\" --type=task` |\n\
                 | Find available work | `bd` | `bd ready` |\n\
                 | Manage dependencies | `bd` | `bd dep add <blocked> <blocker>` |\n",
            );
        }
        if has_openspec {
            block.push_str(
                "| Propose system changes | openspec | Read `openspec/AGENTS.md` |\n\
                 | Define requirements | openspec | `openspec validate --strict` |\n",
            );
        }
        block.push_str(
            "\nKey distinction:\n\
             - **wai** = *why* decisions were made (reasoning, context, handoffs)\n",
        );
        if has_beads {
            block.push_str(
                "- **`bd`** (beads) = *what* needs to be done (concrete tasks, status tracking)\n",
            );
        }
        if has_openspec {
            block.push_str(
                "- **openspec** = *what the system should look like* (specs, requirements, proposals)\n",
            );
        }
    }

    // Starting a Session (unified)
    block.push_str("\n## Starting a Session\n\n");
    let mut step = 1;
    block.push_str(&format!(
        "{}. Run `wai status` to see active projects, current phase, and suggestions.\n",
        step
    ));
    step += 1;
    if has_beads {
        block.push_str(&format!(
            "{}. Run `bd ready` to find available work items.\n",
            step
        ));
        block.push_str(
            "   Before claiming: read the relevant source files to confirm\n\
             \x20  the issue is not already implemented.\n",
        );
        step += 1;
    }
    if has_openspec {
        block.push_str(&format!(
            "{}. Check `openspec list` for active change proposals.\n",
            step
        ));
        step += 1;
    }
    block.push_str(&format!(
        "{}. Check the phase — it tells you what kind of work is expected:\n\
         \x20  - **research** → gather information, explore options\n\
         \x20  - **design** → make architectural decisions\n\
         \x20  - **plan** → break work into tasks\n\
         \x20  - **implement** → write code, guided by research/plans\n\
         \x20  - **review** → validate against plans\n\
         \x20  - **archive** → wrap up\n",
        step
    ));
    step += 1;
    block.push_str(&format!(
        "{}. Read existing artifacts with `wai search \"<topic>\"` before starting new work.\n",
        step
    ));

    // Capturing Work (condensed wai core)
    block.push_str(
        "\n\
         ## Capturing Work\n\
         \n\
         Record the reasoning behind your work, not just the output:\n\
         \n\
         ```bash\n\
         wai add research \"findings\"         # What you learned, trade-offs\n\
         wai add plan \"approach\"             # How you'll implement, why\n\
         wai add design \"decisions\"          # Architecture choices, rationale\n\
         wai add research --file notes.md    # Import longer content\n\
         ```\n\
         \n\
         Use `--project <name>` if multiple projects exist. Otherwise wai picks the first one.\n\
         \n\
         Phases are a guide, not a gate. Use `wai phase show` / `wai phase next`.\n",
    );

    // Tracking Work Across Tools (when both beads and openspec present)
    if has_beads && has_openspec {
        block.push_str(
            "\n## Tracking Work Across Tools\n\
             \n\
             When beads and openspec are both active, keep them in sync:\n\
             - When creating a beads ticket for an openspec task, include the task\n\
             \x20 reference in the description (format: `<change-id>:<phase>.<task>`,\n\
             \x20 e.g. `add-why-command:7.1`)\n\
             - When closing a beads ticket linked to a task, also check the box\n\
             \x20 (`[x]`) in the change's `tasks.md`\n",
        );
    }

    // Ending a Session (unified)
    block.push_str(
        "\n## Ending a Session\n\n\
         Before saying \"done\", run this checklist:\n\n\
         ```\n\
         [ ] wai handoff create <project>   # capture context for next session\n",
    );
    if has_beads {
        block.push_str(
            "[ ] bd close <id>                  # close completed issues; also close parent epic if last sub-task\n\
             [ ] bd sync --from-main            # pull beads updates\n",
        );
    }
    if has_openspec {
        block.push_str(
            "[ ] openspec tasks.md — mark completed tasks [x]\n\
             [ ] openspec list — archive any ✓ Complete changes (`openspec archive <id> --yes`)\n",
        );
    }
    block.push_str(
        "[ ] wai reflect                    # update CLAUDE.md with project patterns (every ~5 sessions)\n\
         [ ] git add <files> && git commit  # commit code + handoff\n\
         ```\n\
         \n\
         ### Autonomous Loop\n\
         \n\
         One task per session. The resume loop:\n\
         \n\
         1. `wai prime` — orient (shows ⚡ RESUMING if mid-task)\n\
         2. Work on the single task\n\
         3. `wai close` — capture state (run this before every `/clear`)\n\
         4. `git add <files> && git commit`\n\
         5. `/clear` — fresh context\n\
         \n\
         → Next session: `wai prime` shows RESUMING with exact next steps.\n\
         \n\
         When context reaches ~40%: stop and tell the user — responses degrade past\n\
         this point. Recommend `wai close` then `/clear` to resume cleanly.\n\
         Do NOT skip `wai close` — it enables resume detection.\n",
    );

    // Quick Reference
    block.push_str(
        "\n\
         ## Quick Reference\n\
         \n\
         ### wai\n\
         ```bash\n\
         wai status                    # Project status and next steps\n\
         wai add research \"notes\"      # Add research artifact\n\
         wai add plan \"plan\"           # Add plan artifact\n\
         wai add design \"design\"       # Add design artifact\n\
         wai add skill <name>          # Scaffold a new agent skill\n\
         wai search \"query\"            # Search across artifacts\n\
         wai search --tag <tag>        # Filter by tag (repeatable)\n\
         wai search --latest           # Most recent match only\n\
         wai why \"why use TOML?\"       # Ask why (LLM-powered oracle)\n\
         wai why src/config.rs         # Explain a file's history\n\
         wai reflect                   # Synthesize project patterns into CLAUDE.md\n\
         wai close                     # Session handoff + pending-resume signal\n\
         wai phase show                # Current phase\n\
         wai doctor                    # Workspace health\n\
         wai pipeline list             # List pipelines\n\
         wai pipeline start <n> --topic=<t>  # Start a run; set WAI_PIPELINE_RUN=<id>\n\
         wai pipeline next             # Advance to next step\n\
         ```\n",
    );
    if has_beads {
        block.push_str(
            "\n\
             ### beads (CLI: `bd`)\n\
             ```bash\n\
             bd ready                     # Available work\n\
             bd show <id>                 # Issue details\n\
             bd create --title=\"...\"      # New issue\n\
             bd update <id> --status=in_progress\n\
             bd close <id>                # Complete work\n\
             ```\n",
        );
    }
    if has_openspec {
        block.push_str(
            "\n\
             ### openspec\n\
             Read `openspec/AGENTS.md` for full instructions.\n\
             ```bash\n\
             openspec list              # Active changes\n\
             openspec list --specs      # Capabilities\n\
             ```\n",
        );
    }

    // Available Pipelines (only when pipelines with metadata exist)
    if !installed_pipelines.is_empty() {
        block.push_str(
            "\n\
             ## Available Pipelines\n\
             \n\
             | Pipeline | When to Use | Start |\n\
             |----------|-------------|-------|\n",
        );
        for p in installed_pipelines {
            block.push_str(&format!(
                "| {} | {} | `wai pipeline start {} --topic=<topic>` |\n",
                p.name, p.when, p.name,
            ));
        }
        block.push_str(
            "\n> Pipeline steps may have gates that enforce artifact creation, review \
             coverage, and oracle checks before advancement. \
             Run `wai pipeline gates <name>` for details.\n",
        );
    }

    // Structure + footer
    block.push_str(
        "\n\
         ## Structure\n\
         \n\
         The `.wai/` directory organizes artifacts using the PARA method:\n\
         - **projects/** — active work with phase tracking and dated artifacts\n\
         - **areas/** — ongoing responsibilities (no end date)\n\
         - **resources/** — reference material, agent configs, templates\n\
         - **archives/** — completed or inactive items\n\
         \n\
         Do not edit `.wai/config.toml` directly. Use `wai` commands instead.\n\
         \n\
         Keep this managed block so `wai init` can refresh the instructions.\n\
         \n",
    );

    block.push_str(WAI_END);
    block
}

pub fn inject_managed_block(
    path: &Path,
    detected_plugins: &[&str],
    installed_skills: &[&str],
    installed_pipelines: &[InstalledPipeline],
) -> Result<InjectResult, std::io::Error> {
    let wai_block = wai_block_content(detected_plugins, installed_skills, installed_pipelines);
    let ref_block = format!(
        "{}\n{}{}\n",
        REFLECT_REF_START,
        wai_reflect_ref_content(),
        REFLECT_REF_END
    );

    if path.exists() {
        let content = std::fs::read_to_string(path)?;

        if let (Some(start_idx), Some(end_idx)) = (content.find(WAI_START), content.find(WAI_END)) {
            let wai_end_pos = end_idx + WAI_END.len();
            let mut new_content = String::with_capacity(content.len() + 512);
            new_content.push_str(&content[..start_idx]);
            new_content.push_str(&wai_block);

            // Handle content after WAI:END — update existing REF block or append one.
            let tail = &content[wai_end_pos..];
            if let (Some(ref_start), Some(ref_end)) =
                (tail.find(REFLECT_REF_START), tail.find(REFLECT_REF_END))
            {
                if ref_start < ref_end {
                    let ref_end_abs = ref_end + REFLECT_REF_END.len();
                    new_content.push_str(&tail[..ref_start]);
                    new_content.push_str(&ref_block);
                    new_content.push_str(&tail[ref_end_abs..]);
                } else {
                    // Inverted markers — treat as no REF block.
                    new_content.push_str("\n\n");
                    new_content.push_str(&ref_block);
                    new_content.push_str(tail);
                }
            } else {
                new_content.push_str("\n\n");
                new_content.push_str(&ref_block);
                new_content.push_str(tail);
            }

            std::fs::write(path, new_content)?;
            Ok(InjectResult::Updated)
        } else {
            let mut new_content = wai_block;
            new_content.push_str("\n\n");
            new_content.push_str(&ref_block);
            new_content.push_str("\n\n");
            new_content.push_str(&content);
            std::fs::write(path, new_content)?;
            Ok(InjectResult::Prepended)
        }
    } else {
        let mut new_content = wai_block;
        new_content.push_str("\n\n");
        new_content.push_str(&ref_block);
        std::fs::write(path, &new_content)?;
        Ok(InjectResult::Created)
    }
}

/// Extract the actual WAI block content (between WAI:START and WAI:END, inclusive).
/// Returns `None` if the file does not exist or has no block.
pub fn read_managed_block(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let start = content.find(WAI_START)?;
    let end = content.find(WAI_END)? + WAI_END.len();
    if start > end {
        return None;
    }
    Some(content[start..end].to_string())
}

pub fn has_managed_block(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match std::fs::read_to_string(path) {
        Ok(content) => content.contains(WAI_START) && content.contains(WAI_END),
        Err(_) => false,
    }
}

pub enum InjectResult {
    Created,
    Prepended,
    Updated,
}

impl InjectResult {
    pub fn description(&self, filename: &str) -> String {
        match self {
            InjectResult::Created => format!("Created {} with wai instructions", filename),
            InjectResult::Prepended => {
                format!("Added wai instructions to existing {}", filename)
            }
            InjectResult::Updated => format!("Updated wai instructions in {}", filename),
        }
    }
}

// ── REFLECT:REF block ────────────────────────────────────────────────────────

pub const REFLECT_REF_START: &str = "<!-- WAI:REFLECT:REF:START -->";
pub const REFLECT_REF_END: &str = "<!-- WAI:REFLECT:REF:END -->";

/// Returns the slim reference block content that tells agents where project
/// patterns live and instructs them to search before starting research.
pub fn wai_reflect_ref_content() -> &'static str {
    "## Accumulated Project Patterns\n\
     \n\
     Project-specific conventions, gotchas, and architecture notes live in\n\
     `.wai/resources/reflections/`. Run `wai search \"<topic>\"` to retrieve relevant\n\
     context before starting research or creating tickets.\n\
     \n\
     > **Before research or ticket creation**: always run `wai search \"<topic>\"` to\n\
     > check for known patterns. Do not rediscover what is already documented.\n"
}

// ── REFLECT block ────────────────────────────────────────────────────────────

const REFLECT_START: &str = "<!-- WAI:REFLECT:START -->";
const REFLECT_END: &str = "<!-- WAI:REFLECT:END -->";

/// Returns true if the file at `path` contains a WAI:REFLECT managed block.
pub fn has_reflect_block(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match std::fs::read_to_string(path) {
        Ok(content) => content.contains(REFLECT_START) && content.contains(REFLECT_END),
        Err(_) => false,
    }
}

/// Read the content between the WAI:REFLECT markers (excluding the markers
/// themselves). Returns `None` if the file does not exist or has no block.
pub fn read_reflect_block(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let start = content.find(REFLECT_START)? + REFLECT_START.len();
    let end = content.find(REFLECT_END)?;
    if start > end {
        return None;
    }
    Some(content[start..end].to_string())
}

#[cfg(test)]
mod wai_block_tests {
    use super::*;

    // Phase 1: session-close openspec checklist step

    #[test]
    fn openspec_checklist_step_present_when_openspec_detected() {
        let output = wai_block_content(&["openspec"], &[], &[]);
        assert!(
            output.contains("openspec tasks.md"),
            "expected 'openspec tasks.md' in output"
        );
    }

    #[test]
    fn openspec_checklist_step_absent_without_openspec() {
        let output = wai_block_content(&[], &[], &[]);
        assert!(
            !output.contains("openspec tasks.md"),
            "unexpected 'openspec tasks.md' in output without openspec"
        );
    }

    #[test]
    fn openspec_checklist_step_ordering() {
        let output = wai_block_content(&["beads", "openspec"], &[], &[]);
        let bd_sync_pos = output
            .find("bd sync --from-main")
            .expect("bd sync --from-main not found");
        let openspec_pos = output
            .find("openspec tasks.md")
            .expect("openspec tasks.md not found");
        let wai_reflect_pos = output.find("wai reflect").expect("wai reflect not found");
        assert!(
            bd_sync_pos < openspec_pos,
            "openspec tasks.md should appear after bd sync --from-main"
        );
        assert!(
            openspec_pos < wai_reflect_pos,
            "openspec tasks.md should appear before wai reflect"
        );
    }

    #[test]
    fn openspec_archive_step_present_when_openspec_detected() {
        let output = wai_block_content(&["openspec"], &[], &[]);
        assert!(
            output.contains("openspec archive"),
            "expected 'openspec archive' in output"
        );
    }

    #[test]
    fn openspec_archive_step_absent_without_openspec() {
        let output = wai_block_content(&[], &[], &[]);
        assert!(
            !output.contains("openspec archive"),
            "unexpected 'openspec archive' in output without openspec"
        );
    }

    #[test]
    fn openspec_archive_step_after_tasks_step() {
        let output = wai_block_content(&["openspec"], &[], &[]);
        let tasks_pos = output
            .find("openspec tasks.md")
            .expect("openspec tasks.md not found");
        let archive_pos = output
            .find("openspec archive")
            .expect("openspec archive not found");
        assert!(
            tasks_pos < archive_pos,
            "openspec archive should appear after openspec tasks.md"
        );
    }

    // Phase 2: cross-tool tracking section

    #[test]
    fn tracking_section_present_when_both_beads_and_openspec() {
        let output = wai_block_content(&["beads", "openspec"], &[], &[]);
        assert!(
            output.contains("Tracking Work Across Tools"),
            "expected 'Tracking Work Across Tools' in output"
        );
    }

    #[test]
    fn tracking_section_absent_with_only_beads() {
        let output = wai_block_content(&["beads"], &[], &[]);
        assert!(
            !output.contains("Tracking Work Across Tools"),
            "unexpected 'Tracking Work Across Tools' with beads only"
        );
    }

    #[test]
    fn tracking_section_absent_with_only_openspec() {
        let output = wai_block_content(&["openspec"], &[], &[]);
        assert!(
            !output.contains("Tracking Work Across Tools"),
            "unexpected 'Tracking Work Across Tools' with openspec only"
        );
    }

    #[test]
    fn tracking_section_between_capturing_work_and_ending_session() {
        let output = wai_block_content(&["beads", "openspec"], &[], &[]);
        let capturing_pos = output
            .find("## Capturing Work")
            .expect("## Capturing Work not found");
        let tracking_pos = output
            .find("## Tracking Work Across Tools")
            .expect("## Tracking Work Across Tools not found");
        let ending_pos = output
            .find("## Ending a Session")
            .expect("## Ending a Session not found");
        assert!(
            capturing_pos < tracking_pos,
            "Tracking section should appear after Capturing Work"
        );
        assert!(
            tracking_pos < ending_pos,
            "Tracking section should appear before Ending a Session"
        );
    }

    // Phase 3: pre-claim implementation check

    #[test]
    fn pre_claim_note_present_with_beads() {
        let output = wai_block_content(&["beads"], &[], &[]);
        assert!(
            output.contains("already implemented"),
            "expected 'already implemented' near bd ready line"
        );
    }

    #[test]
    fn pre_claim_note_absent_without_beads() {
        let output = wai_block_content(&[], &[], &[]);
        assert!(
            !output.contains("already implemented"),
            "unexpected 'already implemented' without beads"
        );
    }

    // Phase 4: epic closure reminder

    #[test]
    fn bd_close_line_mentions_epic_with_beads() {
        let output = wai_block_content(&["beads"], &[], &[]);
        let bd_close_line = output
            .lines()
            .find(|l| l.contains("bd close <id>"))
            .expect("bd close line not found");
        assert!(
            bd_close_line.contains("epic") || bd_close_line.contains("parent"),
            "bd close line should mention 'epic' or 'parent', got: {bd_close_line}"
        );
    }

    #[test]
    fn bd_close_line_absent_without_beads() {
        let output = wai_block_content(&[], &[], &[]);
        assert!(
            !output.contains("bd close <id>"),
            "unexpected 'bd close <id>' without beads"
        );
    }

    // TDD/Tidy First disclaimer

    #[test]
    fn tdd_disclaimer_present_with_companion_tools() {
        let output = wai_block_content(&["beads", "openspec"], &[], &[]);
        assert!(
            output.contains("CRITICAL"),
            "expected CRITICAL disclaimer in output with companion tools"
        );
        assert!(
            output.contains("TDD"),
            "expected 'TDD' in output with companion tools"
        );
        assert!(
            output.contains("Tidy First"),
            "expected 'Tidy First' in output with companion tools"
        );
    }

    #[test]
    fn tdd_disclaimer_present_with_beads_only() {
        let output = wai_block_content(&["beads"], &[], &[]);
        assert!(
            output.contains("CRITICAL"),
            "expected CRITICAL disclaimer in output with beads"
        );
    }

    #[test]
    fn tdd_disclaimer_absent_without_companion_tools() {
        let output = wai_block_content(&[], &[], &[]);
        assert!(
            !output.contains("Tidy First"),
            "unexpected 'Tidy First' in output without companion tools"
        );
    }

    #[test]
    fn tdd_disclaimer_before_when_to_use_what() {
        let output = wai_block_content(&["beads"], &[], &[]);
        let critical_pos = output
            .find("CRITICAL")
            .expect("CRITICAL not found in output");
        let when_pos = output
            .find("## When to Use What")
            .expect("## When to Use What not found");
        assert!(
            critical_pos < when_pos,
            "CRITICAL disclaimer should appear before '## When to Use What'"
        );
    }

    // ro5 skill reminder

    #[test]
    fn ro5_reminder_present_when_skill_installed() {
        for name in &["ro5", "rule-of-5", "rule-of-5-universal"] {
            let output = wai_block_content(&[], &[name], &[]);
            assert!(
                output.contains("/ro5"),
                "expected '/ro5' in output when skill '{name}' installed"
            );
            assert!(
                output.contains("Rule of 5"),
                "expected 'Rule of 5' in output when skill '{name}' installed"
            );
        }
    }

    #[test]
    fn ro5_reminder_absent_without_skill() {
        let output = wai_block_content(&["beads", "openspec"], &[], &[]);
        assert!(
            !output.contains("/ro5"),
            "unexpected '/ro5' in output without ro5 skill"
        );
    }

    // 6.4: search-before-research instruction present with companions, absent without
    // The instruction uses "before writing new content" as its distinguishing phrase.

    const SEARCH_INSTRUCTION: &str = "before writing new content";

    #[test]
    fn search_before_research_present_with_companions() {
        for plugins in [
            &["beads"][..],
            &["openspec"][..],
            &["beads", "openspec"][..],
        ] {
            let output = wai_block_content(plugins, &[], &[]);
            assert!(
                output.contains(SEARCH_INSTRUCTION),
                "expected search-before-research instruction with plugins {:?}",
                plugins
            );
        }
    }

    #[test]
    fn search_before_research_absent_without_companions() {
        let output = wai_block_content(&[], &[], &[]);
        assert!(
            !output.contains(SEARCH_INSTRUCTION),
            "unexpected search-before-research instruction without companion tools"
        );
    }

    #[test]
    fn search_before_research_after_tdd_disclaimer() {
        let output = wai_block_content(&["beads"], &[], &[]);
        let tdd_pos = output.find("CRITICAL").expect("CRITICAL not found");
        let search_pos = output
            .find(SEARCH_INSTRUCTION)
            .expect("search instruction not found");
        assert!(
            search_pos > tdd_pos,
            "search-before-research should appear after TDD disclaimer"
        );
    }

    // Context pressure warning

    #[test]
    fn context_pressure_tells_user() {
        let output = wai_block_content(&[], &[], &[]);
        assert!(
            output.contains("stop and tell the user"),
            "expected 'stop and tell the user' context pressure instruction"
        );
        assert!(
            output.contains("responses degrade"),
            "expected 'responses degrade' phrasing"
        );
    }

    // Pipeline section tests

    #[test]
    fn pipeline_section_present_when_pipelines_installed() {
        let pipelines = vec![InstalledPipeline {
            name: "scientific-research".to_string(),
            description: "AI-assisted research".to_string(),
            when: "Frontier-level research requiring systematic validation".to_string(),
            step_count: 8,
        }];
        let output = wai_block_content(&[], &[], &pipelines);
        assert!(
            output.contains("## Available Pipelines"),
            "expected 'Available Pipelines' section"
        );
        assert!(
            output.contains("scientific-research"),
            "expected pipeline name in output"
        );
        assert!(
            output.contains("Frontier-level research"),
            "expected 'when' description in output"
        );
        assert!(
            output.contains("wai pipeline start scientific-research --topic=<topic>"),
            "expected start command in output"
        );
    }

    #[test]
    fn pipeline_section_absent_when_no_pipelines() {
        let output = wai_block_content(&[], &[], &[]);
        assert!(
            !output.contains("Available Pipelines"),
            "unexpected 'Available Pipelines' section without pipelines"
        );
    }

    #[test]
    fn pipeline_section_includes_gate_note() {
        let pipelines = vec![InstalledPipeline {
            name: "test".to_string(),
            description: "Test pipeline".to_string(),
            when: "Testing".to_string(),
            step_count: 2,
        }];
        let output = wai_block_content(&[], &[], &pipelines);
        assert!(
            output.contains("gates"),
            "expected gate note in pipeline section"
        );
    }

    #[test]
    fn pipeline_section_between_quick_ref_and_structure() {
        let pipelines = vec![InstalledPipeline {
            name: "test".to_string(),
            description: "Test".to_string(),
            when: "Testing".to_string(),
            step_count: 1,
        }];
        let output = wai_block_content(&[], &[], &pipelines);
        let quick_ref_pos = output
            .find("## Quick Reference")
            .expect("Quick Reference not found");
        let pipeline_pos = output
            .find("## Available Pipelines")
            .expect("Available Pipelines not found");
        let structure_pos = output.find("## Structure").expect("Structure not found");
        assert!(
            pipeline_pos > quick_ref_pos,
            "Available Pipelines should appear after Quick Reference"
        );
        assert!(
            pipeline_pos < structure_pos,
            "Available Pipelines should appear before Structure"
        );
    }

    #[test]
    fn pipeline_section_lists_multiple_pipelines() {
        let pipelines = vec![
            InstalledPipeline {
                name: "alpha".to_string(),
                description: "Alpha workflow".to_string(),
                when: "When alpha".to_string(),
                step_count: 3,
            },
            InstalledPipeline {
                name: "beta".to_string(),
                description: "Beta workflow".to_string(),
                when: "When beta".to_string(),
                step_count: 5,
            },
        ];
        let output = wai_block_content(&[], &[], &pipelines);
        assert!(output.contains("alpha"));
        assert!(output.contains("beta"));
        assert!(output.contains("When alpha"));
        assert!(output.contains("When beta"));
    }
}

#[cfg(test)]
mod reflect_ref_tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    // 6.3: wai_reflect_ref_content() contains "wai search" and the resource path
    #[test]
    fn reflect_ref_content_contains_wai_search() {
        let content = wai_reflect_ref_content();
        assert!(
            content.contains("wai search"),
            "expected 'wai search' in reflect_ref_content"
        );
    }

    #[test]
    fn reflect_ref_content_contains_resource_path() {
        let content = wai_reflect_ref_content();
        assert!(
            content.contains(".wai/resources/reflections/"),
            "expected resource path in reflect_ref_content"
        );
    }

    // 6.5: WAI:REFLECT:REF:START/END block injected by inject_managed_block()
    #[test]
    fn inject_managed_block_adds_reflect_ref_block() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "# Header\n").unwrap();
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains(REFLECT_REF_START),
            "expected WAI:REFLECT:REF:START in output"
        );
        assert!(
            content.contains(REFLECT_REF_END),
            "expected WAI:REFLECT:REF:END in output"
        );
    }

    #[test]
    fn inject_managed_block_reflect_ref_after_wai_end() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "# Header\n").unwrap();
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let wai_end_pos = content.find(WAI_END).expect("WAI:END not found");
        let ref_start_pos = content
            .find(REFLECT_REF_START)
            .expect("REFLECT:REF:START not found");
        assert!(
            ref_start_pos > wai_end_pos,
            "REFLECT:REF block should appear after WAI:END"
        );
    }

    #[test]
    fn inject_managed_block_updates_reflect_ref_in_place() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(
            &path,
            "<!-- WAI:START -->\nwai\n<!-- WAI:END -->\n\n\
             <!-- WAI:REFLECT:REF:START -->\nold content\n<!-- WAI:REFLECT:REF:END -->\n",
        )
        .unwrap();
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let count = content.matches(REFLECT_REF_START).count();
        assert_eq!(count, 1, "should not duplicate REFLECT:REF block");
        assert!(
            !content.contains("old content"),
            "should have replaced old REF content"
        );
    }
}

#[cfg(test)]
mod reflect_tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn has_reflect_block_false_when_file_missing() {
        let dir = tmp();
        assert!(!has_reflect_block(&dir.path().join("CLAUDE.md")));
    }

    #[test]
    fn has_reflect_block_false_when_no_markers() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "# Hello\nSome content\n").unwrap();
        assert!(!has_reflect_block(&path));
    }

    #[test]
    fn has_reflect_block_true_when_markers_present() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(
            &path,
            "# Hello\n<!-- WAI:REFLECT:START -->\nfoo\n<!-- WAI:REFLECT:END -->\n",
        )
        .unwrap();
        assert!(has_reflect_block(&path));
    }

    #[test]
    fn read_reflect_block_returns_none_when_missing() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "# No block here\n").unwrap();
        assert_eq!(read_reflect_block(&path), None);
    }

    #[test]
    fn read_reflect_block_returns_inner_content() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(
            &path,
            "pre\n<!-- WAI:REFLECT:START -->\ninner content\n<!-- WAI:REFLECT:END -->\npost\n",
        )
        .unwrap();
        let got = read_reflect_block(&path).unwrap();
        assert!(got.contains("inner content"));
    }
}
