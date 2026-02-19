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

    // Ending a Session (unified)
    block.push_str("\n## Ending a Session\n\n");
    let mut step = 1;
    block.push_str(&format!(
        "{}. Create a handoff: `wai handoff create <project>`\n",
        step
    ));
    step += 1;
    if has_beads {
        block.push_str(&format!(
            "{}. Update issue status: `bd close <id>` for completed work\n",
            step
        ));
        step += 1;
        block.push_str(&format!(
            "{}. File new issues for remaining work: `bd create --title=\"...\"`\n",
            step
        ));
        step += 1;
    }
    block.push_str(&format!(
        "{}. Commit your changes (handoff + code)\n",
        step
    ));

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
         wai handoff create <project>  # Session handoff\n\
         wai phase show                # Current phase\n\
         wai doctor                    # Workspace health\n\
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

pub fn inject_managed_block(path: &Path, detected_plugins: &[&str]) -> Result<InjectResult, std::io::Error> {
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
