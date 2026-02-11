use std::path::Path;

const WAI_START: &str = "<!-- WAI:START -->";
const WAI_END: &str = "<!-- WAI:END -->";

pub fn wai_block_content() -> String {
    format!(
        r#"{start}
# Wai — Workflow Context

This project uses **wai** to track the *why* behind decisions — research,
reasoning, and design choices that shaped the code. Run `wai status` first
to orient yourself.

## When Starting a Session

1. Run `wai status` to see active projects, current phase, and suggestions.
2. Check the phase — it tells you what kind of work is expected right now:
   - **research** → gather information, explore options, document findings
   - **design** → make architectural decisions, write design docs
   - **plan** → break work into tasks, define implementation order
   - **implement** → write code, guided by existing research/plans/designs
   - **review** → validate work against plans and designs
   - **archive** → wrap up, move to archives
3. Read existing artifacts with `wai search "<topic>"` before starting new work.

## Capturing Work

Record the reasoning behind your work, not just the output:

```bash
wai add research "findings"         # What you learned, options explored, trade-offs
wai add plan "approach"             # How you'll implement, in what order, why
wai add design "decisions"          # Architecture choices and rationale
wai add research --file notes.md    # Import longer content from a file
```

**What goes where:**
- **Research** = facts, explorations, comparisons, prior art, constraints discovered
- **Plans** = sequenced steps, task breakdowns, implementation strategies
- **Designs** = architectural decisions, component relationships, API shapes, trade-offs chosen

Use `--project <name>` if multiple projects exist. Otherwise wai picks the first one.

## Advancing Phases

Move the project forward when the current phase's work is done:

```bash
wai phase show          # Where are we now?
wai phase next          # Advance to next phase
wai phase set <phase>   # Jump to a specific phase (flexible, not enforced)
```

Phases are a guide, not a gate. Skip or go back as needed.

## When Ending a Session

Create a handoff so the next session (yours or someone else's) has context:

```bash
wai handoff create <project>
```

This generates a template with sections for: what was done, key decisions,
open questions, and next steps. Fill it in before stopping.

## Quick Reference

```bash
wai status                    # Project status and next steps
wai phase show                # Current project phase
wai new project "name"        # Create a new project
wai add research "notes"      # Add research notes
wai add plan "plan"           # Add a plan document
wai add design "design"       # Add a design document
wai search "query"            # Search across all artifacts
wai handoff create <project>  # Generate handoff document
wai sync                      # Sync agent configs
wai show                      # Overview of all items
wai timeline <project>        # Chronological view of artifacts
wai doctor                    # Check workspace health
```

## Structure

The `.wai/` directory organizes artifacts using the PARA method:
- **projects/** — active work with phase tracking and dated artifacts
- **areas/** — ongoing responsibilities (no end date)
- **resources/** — reference material, agent configs, templates
- **archives/** — completed or inactive items

Do not edit `.wai/config.toml` directly. Use `wai` commands instead.

Keep this managed block so `wai init` can refresh the instructions.

{end}"#,
        start = WAI_START,
        end = WAI_END
    )
}

pub fn inject_managed_block(path: &Path) -> Result<InjectResult, std::io::Error> {
    let block = wai_block_content();

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
