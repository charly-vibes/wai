use std::path::Path;

const WAI_START: &str = "<!-- WAI:START -->";
const WAI_END: &str = "<!-- WAI:END -->";

pub fn wai_block_content(detected_plugins: &[&str]) -> String {
    let has_beads = detected_plugins.contains(&"beads");
    let has_openspec = detected_plugins.contains(&"openspec");
    let has_companions = has_beads || has_openspec;

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
            block.push_str("- **beads (bd)** — issue tracking (tasks, bugs, dependencies)\n");
        }
        if has_openspec {
            block.push_str(
                "- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)\n",
            );
        }
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
                "| Track work items/bugs | beads | `bd create --title=\"...\" --type=task` |\n\
                 | Find available work | beads | `bd ready` |\n\
                 | Manage dependencies | beads | `bd dep add <blocked> <blocker>` |\n",
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
                "- **beads** = *what* needs to be done (concrete tasks, status tracking)\n",
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
        block.push_str("[ ] openspec tasks.md — mark completed tasks [x]\n");
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
         When context reaches ~40%: run `wai close`, then `/clear`.\n\
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
         wai pipeline run <n> --topic=<t>  # Start a run; set WAI_PIPELINE_RUN=<id>\n\
         wai pipeline advance <run-id> # Mark stage done, get next hint\n\
         ```\n",
    );
    if has_beads {
        block.push_str(
            "\n\
             ### beads\n\
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
) -> Result<InjectResult, std::io::Error> {
    let block = wai_block_content(detected_plugins);

    if path.exists() {
        let content = std::fs::read_to_string(path)?;

        if let (Some(start_idx), Some(end_idx)) = (content.find(WAI_START), content.find(WAI_END)) {
            let end_idx = end_idx + WAI_END.len();
            let mut new_content = String::with_capacity(content.len());
            new_content.push_str(&content[..start_idx]);
            new_content.push_str(&block);
            new_content.push_str(&content[end_idx..]);
            std::fs::write(path, new_content)?;
            Ok(InjectResult::Updated)
        } else {
            let mut new_content = block;
            new_content.push_str("\n\n");
            new_content.push_str(&content);
            std::fs::write(path, new_content)?;
            Ok(InjectResult::Prepended)
        }
    } else {
        std::fs::write(path, &block)?;
        Ok(InjectResult::Created)
    }
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

/// Inject or update a WAI:REFLECT block in the file at `path`.
///
/// - If the file already has a REFLECT block, it is replaced in-place.
/// - If the file has a WAI:END marker, the REFLECT block is appended after it.
/// - Otherwise the REFLECT block is appended at the end of the file.
///
/// `content` is the raw inner content (the text between the markers).
pub fn inject_reflect_block(path: &Path, content: &str) -> Result<InjectResult, std::io::Error> {
    let block = format!("{}\n{}\n{}\n", REFLECT_START, content.trim(), REFLECT_END);

    if path.exists() {
        let existing = std::fs::read_to_string(path)?;

        if let (Some(start_idx), Some(end_idx)) =
            (existing.find(REFLECT_START), existing.find(REFLECT_END))
        {
            // Update in-place.
            let end_idx = end_idx + REFLECT_END.len();
            let mut new_content = String::with_capacity(existing.len());
            new_content.push_str(&existing[..start_idx]);
            new_content.push_str(&block);
            new_content.push_str(&existing[end_idx..]);
            std::fs::write(path, new_content)?;
            return Ok(InjectResult::Updated);
        }

        // No existing REFLECT block — append after WAI:END if present.
        let mut new_content = existing.clone();
        if let Some(wai_end_idx) = existing.find(WAI_END) {
            let insert_at = wai_end_idx + WAI_END.len();
            new_content.insert_str(insert_at, &format!("\n\n{block}"));
        } else {
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push('\n');
            new_content.push_str(&block);
        }
        std::fs::write(path, new_content)?;
        Ok(InjectResult::Prepended)
    } else {
        std::fs::write(path, &block)?;
        Ok(InjectResult::Created)
    }
}

#[cfg(test)]
mod wai_block_tests {
    use super::*;

    // Phase 1: session-close openspec checklist step

    #[test]
    fn openspec_checklist_step_present_when_openspec_detected() {
        let output = wai_block_content(&["openspec"]);
        assert!(
            output.contains("openspec tasks.md"),
            "expected 'openspec tasks.md' in output"
        );
    }

    #[test]
    fn openspec_checklist_step_absent_without_openspec() {
        let output = wai_block_content(&[]);
        assert!(
            !output.contains("openspec tasks.md"),
            "unexpected 'openspec tasks.md' in output without openspec"
        );
    }

    #[test]
    fn openspec_checklist_step_ordering() {
        let output = wai_block_content(&["beads", "openspec"]);
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

    // Phase 2: cross-tool tracking section

    #[test]
    fn tracking_section_present_when_both_beads_and_openspec() {
        let output = wai_block_content(&["beads", "openspec"]);
        assert!(
            output.contains("Tracking Work Across Tools"),
            "expected 'Tracking Work Across Tools' in output"
        );
    }

    #[test]
    fn tracking_section_absent_with_only_beads() {
        let output = wai_block_content(&["beads"]);
        assert!(
            !output.contains("Tracking Work Across Tools"),
            "unexpected 'Tracking Work Across Tools' with beads only"
        );
    }

    #[test]
    fn tracking_section_absent_with_only_openspec() {
        let output = wai_block_content(&["openspec"]);
        assert!(
            !output.contains("Tracking Work Across Tools"),
            "unexpected 'Tracking Work Across Tools' with openspec only"
        );
    }

    #[test]
    fn tracking_section_between_capturing_work_and_ending_session() {
        let output = wai_block_content(&["beads", "openspec"]);
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
        let output = wai_block_content(&["beads"]);
        assert!(
            output.contains("already implemented"),
            "expected 'already implemented' near bd ready line"
        );
    }

    #[test]
    fn pre_claim_note_absent_without_beads() {
        let output = wai_block_content(&[]);
        assert!(
            !output.contains("already implemented"),
            "unexpected 'already implemented' without beads"
        );
    }

    // Phase 4: epic closure reminder

    #[test]
    fn bd_close_line_mentions_epic_with_beads() {
        let output = wai_block_content(&["beads"]);
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
        let output = wai_block_content(&[]);
        assert!(
            !output.contains("bd close <id>"),
            "unexpected 'bd close <id>' without beads"
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

    #[test]
    fn inject_reflect_block_creates_file_when_missing() {
        let dir = tmp();
        let path = dir.path().join("AGENTS.md");
        inject_reflect_block(&path, "some guidance").unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains(REFLECT_START));
        assert!(content.contains("some guidance"));
        assert!(content.contains(REFLECT_END));
    }

    #[test]
    fn inject_reflect_block_appends_after_wai_end() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(&path, "<!-- WAI:START -->\nwai stuff\n<!-- WAI:END -->\n").unwrap();
        inject_reflect_block(&path, "project guidance").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let wai_end_pos = content.find(WAI_END).unwrap();
        let reflect_start_pos = content.find(REFLECT_START).unwrap();
        assert!(
            reflect_start_pos > wai_end_pos,
            "REFLECT block should come after WAI:END"
        );
    }

    #[test]
    fn inject_reflect_block_updates_in_place() {
        let dir = tmp();
        let path = dir.path().join("CLAUDE.md");
        std::fs::write(
            &path,
            "before\n<!-- WAI:REFLECT:START -->\nold\n<!-- WAI:REFLECT:END -->\nafter\n",
        )
        .unwrap();
        inject_reflect_block(&path, "new content").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains("old\n"));
        assert!(content.contains("new content"));
        assert!(content.contains("before"));
        assert!(content.contains("after"));
    }

    #[test]
    fn inject_reflect_block_appends_at_end_when_no_wai_block() {
        let dir = tmp();
        let path = dir.path().join("AGENTS.md");
        std::fs::write(&path, "# Agents\nSome content\n").unwrap();
        inject_reflect_block(&path, "agent guidance").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Agents"));
        assert!(content.contains(REFLECT_START));
        assert!(content.contains("agent guidance"));
    }
}
