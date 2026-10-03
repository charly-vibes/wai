// Detailed agent instructions (`.wai/AGENTS.md`) — Layer 2 of progressive
// disclosure: full session lifecycle, command reference, capturing work,
// cross-tool sync, and PARA structure.
//
// Extracted from managed_block.rs (tidy-first move, no behavior change).

use std::path::Path;

use crate::managed_block::{InstalledPipeline, ubiquitous_language_note};

pub fn wai_detailed_content(
    repo_root: &Path,
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

    let mut doc = String::new();
    doc.push_str(DETAILED_HEADER);
    if has_companions {
        doc.push_str(&detailed_when_to_use(has_beads, has_openspec));
    }
    doc.push_str(&detailed_starting_session(has_beads, has_openspec));
    doc.push_str(&detailed_capturing_work());
    if has_beads && has_openspec {
        doc.push_str(&detailed_tracking_across_tools());
    }
    doc.push_str(&detailed_ending_session(has_beads, has_openspec));
    doc.push_str(&detailed_quality_gate());
    doc.push_str(&detailed_quick_reference(has_beads, has_openspec));
    doc.push_str(&detailed_pipelines_section(installed_pipelines));
    if has_ro5 {
        doc.push_str(&detailed_ro5_reminder());
    }
    if let Some(note) = ubiquitous_language_note(repo_root) {
        doc.push('\n');
        doc.push_str(note);
    }
    doc.push_str(&detailed_structure());
    doc
}

/// File header: managed-by notice.
const DETAILED_HEADER: &str = "# wai Workflow Reference\n\n\
     > This file is managed by `wai init`. Do not edit manually.\n\
     > Changes will be overwritten on the next init.\n\n";

fn detailed_when_to_use(has_beads: bool, has_openspec: bool) -> String {
    let mut doc = String::new();
    // When to Use What (only when companion tools detected)
    doc.push_str(
        "## When to Use What\n\
             \n\
             | Need | Tool | Example |\n\
             |------|------|---------|\n\
             | Record reasoning/research | wai | `wai add research \"findings\"` |\n\
             | Capture design decisions | wai | `wai add design \"architecture choice\"` |\n\
             | Session context transfer | wai | `wai handoff create <project>` |\n",
    );
    if has_beads {
        doc.push_str(
            "| Track work items/bugs | `bd` | `bd create --title=\"...\" --type=task` |\n\
                 | Find available work | `bd` | `bd ready` |\n\
                 | Manage dependencies | `bd` | `bd dep add <blocked> <blocker>` |\n",
        );
    }
    if has_openspec {
        doc.push_str(
            "| Propose system changes | openspec | Read `openspec/AGENTS.md` |\n\
                 | Define requirements | openspec | `openspec validate --strict` |\n",
        );
    }
    doc.push_str(
        "\nKey distinction:\n\
             - **wai** = *why* decisions were made (reasoning, context, handoffs)\n",
    );
    if has_beads {
        doc.push_str(
            "- **`bd`** (beads) = *what* needs to be done (concrete tasks, status tracking)\n",
        );
    }
    if has_openspec {
        doc.push_str(
                "- **openspec** = *what the system should look like* (specs, requirements, proposals)\n",
            );
    }

    doc
}

/// Ro5 skill reminder, present only when the skill is installed.
fn detailed_ro5_reminder() -> String {
    "\n> **Ro5**: The Rule of 5 skill is installed. Run `/ro5` after key phase transitions \
     — implement, research, design — for iterative quality review.\n"
        .to_string()
}

fn detailed_starting_session(has_beads: bool, has_openspec: bool) -> String {
    let mut doc = String::new();
    // Starting a Session
    doc.push_str("\n## Starting a Session\n\n");
    let mut step = 1;
    doc.push_str(&format!(
        "{}. Run `wai sync` to ensure all agent tools and skills are correctly projected.\n",
        step
    ));
    step += 1;
    doc.push_str(&format!(
        "{}. Run `wai status` to see active projects, current phase, and suggestions.\n",
        step
    ));
    step += 1;
    if has_beads {
        doc.push_str(&format!(
            "{}. Run `bd ready` to find available work items.\n",
            step
        ));
        doc.push_str(
            "   Before claiming: read the relevant source files to confirm\n\
             \x20  the issue is not already implemented.\n",
        );
        step += 1;
    }
    if has_openspec {
        doc.push_str(&format!(
            "{}. Check `openspec list` for active change proposals.\n",
            step
        ));
        step += 1;
    }
    doc.push_str(&format!(
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
    doc.push_str(&format!(
        "{}. Read existing artifacts with `wai search \"<topic>\"` before starting new work.\n",
        step
    ));

    doc
}

fn detailed_capturing_work() -> String {
    let mut doc = String::new();
    // Capturing Work
    doc.push_str(
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

    doc
}

fn detailed_tracking_across_tools() -> String {
    let mut doc = String::new();
    // Tracking Work Across Tools
    doc.push_str(
        "\n## Tracking Work Across Tools\n\
             \n\
             When beads and openspec are both active, keep them in sync:\n\
             - When creating a beads ticket for an openspec task, include the task\n\
             \x20 reference in the description (format: `<change-id>:<phase>.<task>`,\n\
             \x20 e.g. `add-why-command:7.1`)\n\
             - When closing a beads ticket linked to a task, also check the box\n\
             \x20 (`[x]`) in the change's `tasks.md`\n",
    );
    doc
}

fn detailed_ending_session(has_beads: bool, has_openspec: bool) -> String {
    let mut doc = String::new();
    // Ending a Session
    doc.push_str(
        "\n## Ending a Session\n\n\
         Before saying \"done\", run this checklist:\n\n\
         ```\n\
         [ ] wai handoff create <project>   # capture context for next session\n",
    );
    if has_beads {
        doc.push_str(
            "[ ] bd close <id>                  # close completed issues; also close parent epic if last sub-task\n",
        );
    }
    if has_openspec {
        doc.push_str(
            "[ ] openspec tasks.md — mark completed tasks [x]\n\
             [ ] openspec list — archive any ✓ Complete changes (`openspec archive <id> --yes`)\n",
        );
    }
    doc.push_str(
        "[ ] wai reflect                    # update CLAUDE.md with project patterns (every ~5 sessions)\n\
         [ ] git add <files> && git commit  # commit code + handoff\n\
         ```\n",
    );
    if has_beads {
        doc.push_str(
            "\nIf beads needs any extra follow-up beyond `bd close`, run `bd` and use the\ncommands your installed version offers. Do not assume a hard-coded sync\nsubcommand.\n",
        );
    }
    doc.push_str(
        "\n### Autonomous Loop\n\
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

    doc
}

fn detailed_quality_gate() -> String {
    let mut doc = String::new();
    // Quality Gate — ledger required before final commit/response
    doc.push_str(
        "\n\
         ## Quality Gate\n\
         \n\
         Before your final commit or response, produce a quality ledger:\n\
         \n\
         ```\n\
         Changed  — files/modules touched and why\n\
         Verified — commands run to confirm correctness (test, build, lint)\n\
         Review   — what was reviewed, by whom/what (self, ro5, pair)\n\
         Risks    — known risks, edge cases, or deferred concerns\n\
         Next     — follow-up work, if any\n\
         ```\n\
         \n\
         The ledger lives in the commit message or session handoff, not in code.\n",
    );

    doc
}

fn detailed_quick_reference(has_beads: bool, has_openspec: bool) -> String {
    let mut doc = String::new();
    // Quick Reference
    doc.push_str(
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
        doc.push_str(
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
        doc.push_str(
            "\n\
             ### openspec\n\
             Read `openspec/AGENTS.md` for full instructions.\n\
             ```bash\n\
             openspec list              # Active changes\n\
             openspec list --specs      # Capabilities\n\
             ```\n",
        );
    }

    doc
}

fn detailed_pipelines_section(installed_pipelines: &[InstalledPipeline]) -> String {
    let mut doc = String::new();
    // Available Pipelines
    doc.push_str(
        "\n\
             ## Available Pipelines\n\
             \n\
             | Pipeline | When to Use | Start |\n\
             |----------|-------------|-------|\n",
    );
    for p in installed_pipelines {
        doc.push_str(&format!(
            "| {} | {} | `wai pipeline start {} --topic=<topic>` |\n",
            p.name, p.when, p.name,
        ));
    }
    doc.push_str(
        "\n> Pipeline steps may have gates that enforce artifact creation, review \
             coverage, and oracle checks before advancement. \
             Run `wai pipeline gates <name>` for details.\n",
    );
    doc
}

fn detailed_structure() -> String {
    let mut doc = String::new();
    // Structure
    doc.push_str(
        "\n\
         ## Structure\n\
         \n\
         The `.wai/` directory organizes artifacts using the PARA method:\n\
         - **projects/** — active work with phase tracking and dated artifacts\n\
         - **areas/** — ongoing responsibilities (no end date)\n\
         - **resources/** — reference material, agent configs, templates\n\
         - **archives/** — completed or inactive items\n\
         \n\
         Do not edit `.wai/config.toml` directly. Use `wai` commands instead.\n",
    );

    doc
}

/// Write the detailed agent instructions to `.wai/AGENTS.md`.
///
/// This file is fully managed by wai (overwritten on each init).
pub fn write_detailed_agents_file(
    wai_dir: &Path,
    detected_plugins: &[&str],
    installed_skills: &[&str],
    installed_pipelines: &[InstalledPipeline],
) -> Result<DetailedFileResult, std::io::Error> {
    let path = wai_dir.join("AGENTS.md");
    let repo_root = wai_dir.parent().unwrap_or(wai_dir);
    let content = wai_detailed_content(
        repo_root,
        detected_plugins,
        installed_skills,
        installed_pipelines,
    );
    let existed = path.exists();
    std::fs::write(&path, &content)?;
    Ok(if existed {
        DetailedFileResult::Updated
    } else {
        DetailedFileResult::Created
    })
}

pub enum DetailedFileResult {
    Created,
    Updated,
}

impl DetailedFileResult {
    pub fn description(&self) -> String {
        match self {
            DetailedFileResult::Created => {
                "Created .wai/AGENTS.md with detailed workflow reference".to_string()
            }
            DetailedFileResult::Updated => "Updated .wai/AGENTS.md workflow reference".to_string(),
        }
    }
}

#[cfg(test)]
mod detailed_tests {
    use super::*;
    use crate::managed_block::{WAI_END, WAI_START};

    #[test]
    fn detailed_block_includes_ubiquitous_language_note_when_index_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let note_root = dir.path().join(".wai/resources/ubiquitous-language");
        std::fs::create_dir_all(&note_root).unwrap();
        std::fs::write(note_root.join("README.md"), "# Index\n").unwrap();

        let output = wai_detailed_content(dir.path(), &[], &[], &[]);
        assert!(output.contains("## Ubiquitous Language"));
        assert!(output.contains("read it first as the"));
        assert!(output.contains("Avoid loading every terminology file"));
    }

    #[test]
    fn detailed_block_omits_ubiquitous_language_note_without_index() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output = wai_detailed_content(dir.path(), &[], &[], &[]);
        assert!(!output.contains("## Ubiquitous Language"));
    }

    // ── Detailed content (Layer 2: .wai/AGENTS.md) ───────────────────────

    #[test]
    fn detailed_contains_session_start() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(output.contains("## Starting a Session"));
    }

    #[test]
    fn detailed_wai_sync_before_status() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        let sync_pos = output.find("wai sync").expect("wai sync not found");
        let status_pos = output.find("wai status").expect("wai status not found");
        assert!(sync_pos < status_pos);
    }

    #[test]
    fn detailed_contains_capturing_work() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(output.contains("## Capturing Work"));
    }

    #[test]
    fn detailed_contains_ending_session() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(output.contains("## Ending a Session"));
    }

    #[test]
    fn detailed_contains_quick_reference() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(output.contains("## Quick Reference"));
    }

    #[test]
    fn detailed_contains_structure() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(output.contains("## Structure"));
    }

    #[test]
    fn detailed_openspec_checklist_present_when_detected() {
        let output = wai_detailed_content(Path::new("."), &["openspec"], &[], &[]);
        assert!(output.contains("openspec tasks.md"));
    }

    #[test]
    fn detailed_openspec_checklist_absent_without_openspec() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(!output.contains("openspec tasks.md"));
    }

    #[test]
    fn detailed_openspec_archive_present_when_detected() {
        let output = wai_detailed_content(Path::new("."), &["openspec"], &[], &[]);
        assert!(output.contains("openspec archive"));
    }

    #[test]
    fn detailed_tracking_section_present_when_both() {
        let output = wai_detailed_content(Path::new("."), &["beads", "openspec"], &[], &[]);
        assert!(output.contains("Tracking Work Across Tools"));
    }

    #[test]
    fn detailed_tracking_section_absent_with_only_beads() {
        let output = wai_detailed_content(Path::new("."), &["beads"], &[], &[]);
        assert!(!output.contains("Tracking Work Across Tools"));
    }

    #[test]
    fn detailed_pre_claim_note_present_with_beads() {
        let output = wai_detailed_content(Path::new("."), &["beads"], &[], &[]);
        assert!(output.contains("already implemented"));
    }

    #[test]
    fn detailed_pre_claim_note_absent_without_beads() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(!output.contains("already implemented"));
    }

    #[test]
    fn detailed_bd_close_mentions_epic_with_beads() {
        let output = wai_detailed_content(Path::new("."), &["beads"], &[], &[]);
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
    fn detailed_beads_note_present_with_beads() {
        let output = wai_detailed_content(Path::new("."), &["beads"], &[], &[]);
        assert!(output.contains("Do not assume a hard-coded sync"));
    }

    #[test]
    fn detailed_beads_note_absent_without_beads() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(!output.contains("Do not assume a hard-coded sync"));
    }

    #[test]
    fn detailed_contains_autonomous_loop() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(output.contains("Autonomous Loop"));
    }

    // ── Quality Gate (detailed content) ─────────────────────────────────

    #[test]
    fn detailed_contains_quality_gate_section() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(
            output.contains("## Quality Gate"),
            "expected Quality Gate section in detailed content"
        );
    }

    #[test]
    fn detailed_quality_gate_requires_ledger_fields() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        for field in ["Changed", "Verified", "Review", "Risks", "Next"] {
            assert!(
                output.contains(field),
                "quality ledger should require '{field}'"
            );
        }
    }

    #[test]
    fn detailed_quality_gate_after_ending_session() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        let ending_pos = output
            .find("Ending a Session")
            .expect("Ending a Session not found");
        let gate_pos = output.find("Quality Gate").expect("Quality Gate not found");
        assert!(
            gate_pos > ending_pos,
            "Quality Gate should appear after Ending a Session"
        );
    }

    #[test]
    fn detailed_does_not_have_wai_markers() {
        let output = wai_detailed_content(Path::new("."), &[], &[], &[]);
        assert!(!output.contains(WAI_START));
        assert!(!output.contains(WAI_END));
    }
}
