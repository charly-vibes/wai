# Design: `wai reflect` — LLM Reflection for CLAUDE.md and AGENTS.md

## Context

`wai reflect` synthesizes accumulated session context into project-specific AI
guidance, then injects it into CLAUDE.md and/or AGENTS.md. It bridges three
existing systems: `src/llm.rs` (LLM abstraction), `src/managed_block.rs`
(auto-update logic), and the handoff system.

The command is rare and deliberate — not run on every session end, but when a
developer senses "the AI assistant keeps asking the same thing."

## Goals / Non-Goals

**Goals:**
- Surface project-specific patterns, conventions, and gotchas from session context
- Accept conversation transcripts as the highest-fidelity input source
- Inject findings into CLAUDE.md and/or AGENTS.md in a dedicated managed block
- Improve handoff quality so nuances are captured at session end
- Require diff-aware confirmation before writing (safe by default)
- Reuse existing LLM infrastructure without adding new configuration

**Non-Goals:**
- Automatic/scheduled reflection (manual command only)
- Reflecting on code content (focus: session artifacts, not source files)
- Replacing the WAI:START tool-instructions block
- Parsing structured conversation formats (accept any plain text)

## Decisions

### Decision: Two Separate Managed Blocks

**What**: The REFLECT block (`<!-- WAI:REFLECT:START/END -->`) is separate from the
existing WAI block (`<!-- WAI:START/END -->`).

**Why**: WAI:START holds generic tool instructions (how to use wai, beads, openspec).
REFLECT holds project-specific learnings (this codebase's conventions, gotchas). These
are orthogonal concerns with different authors (tool template vs. LLM synthesis) and
different update cadences (`wai init` vs. `wai reflect`). Mixing them would mean every
`wai init --refresh` overwrites LLM-generated project knowledge.

**Placement**: REFLECT block is appended after WAI:END when injected for the first
time. On subsequent runs it is updated in-place. For AGENTS.md (which may have no
WAI:END), the block is appended at the end of the file.

**Alternative considered**: A sub-section inside WAI:START. Rejected because `wai init`
regenerates WAI:START content, which would wipe the LLM-generated section.

### Decision: Both CLAUDE.md and AGENTS.md Are Targets

**What**: `wai reflect` supports writing to CLAUDE.md, AGENTS.md, or both via
`--output claude.md|agents.md|both`. Default: update whichever target files already
exist in the repo root (if both exist, `--output both` is assumed).

**Why**: Claude Code uses CLAUDE.md; other AI tools (Cursor, Gemini CLI, etc.) use
AGENTS.md. Projects commonly have both. Limiting to CLAUDE.md would leave AGENTS.md
users unable to benefit from reflect.

**If neither exists**: Fail with a clear diagnostic: "No CLAUDE.md or AGENTS.md
found — run `wai init` first or create the target file manually."

**AGENTS.md placement**: The REFLECT block is appended at end of file. The OPENSPEC
managed block (if present) remains untouched.

### Decision: Three-Tier Input Hierarchy

**What**: Context sources are ranked: conversation transcripts > handoffs > research/design/plan.
The LLM prompt explains what each source type tends to reveal:

| Source | What it captures | Budget |
|--------|-----------------|--------|
| `--conversation <file>` | Failed attempts, surprises, step-by-step struggles | Up to ~30K chars, truncated from top |
| Handoffs | Session intent, next steps, gotchas (with improved template) | Up to ~40K chars, newest first |
| Research/design/plan | Explicit decisions, domain knowledge | Up to ~30K chars remaining |

**Total budget**: ~100K chars, dynamically allocated — each tier fills greedily
until its cap, then the remaining budget overflows to the next tier.

**Why conversation transcripts rank first**: They contain the actual work — the three
commands that failed before the right one worked, the surprising file that needed
editing, the pattern that only emerged after debugging. Handoffs and artifacts are
deliberate and curated; conversations are exhaustive and unfiltered.

**Format**: `--conversation <file>` accepts plain text, markdown, or JSON dump.
The LLM extracts patterns regardless of format. No schema required.

### Decision: `.reflect-meta` for Nudge Heuristic

**What**: A small TOML file `.wai/projects/<project>/.reflect-meta` stores:
```toml
last_reflected = "2026-02-24"
session_count = 12
```

**Why**: The alternative — parsing the `_Last reflected:_` date from CLAUDE.md
content — is fragile (any user edit could corrupt it; CLAUDE.md mtime changes
on every reflect run, every `wai init --refresh`, and every manual edit). A
dedicated metadata file is unambiguous, consistent with `.pending-resume`, and
doesn't require reading CLAUDE.md just to check if a nudge is needed.

**Nudge detection**: On `wai close`, count handoff files whose mtime is newer
than `last_reflected` in `.reflect-meta`. If count ≥ 5, emit the nudge. If
`.reflect-meta` doesn't exist, treat all handoffs as newer.

### Decision: Diff Before Overwrite

**What**: When a REFLECT block already exists and the user has confirmed writing,
show a unified diff of old content vs. proposed content before the final confirm,
not just "Write to CLAUDE.md?"

**Why**: Users may have manually refined the REFLECT block between runs. Silently
overwriting that work is a footgun. The diff makes the delta explicit.

**Implementation**: Store the old block content from reading CLAUDE.md/AGENTS.md
before calling the LLM. After LLM response, render the diff (using simple +/- line
comparison). If the diff is empty (no change), print "No changes to make" and exit.

### Decision: Improved Handoff Template Sections

**What**: `wai handoff create` gains two new optional template sections:
- `## Gotchas & Surprises` — unexpected behaviors, non-obvious requirements
- `## What Took Longer Than Expected` — steps that needed multiple attempts

**Why**: The reflect command's quality is bounded by handoff quality. These sections
explicitly ask the user to capture the high-value nuances that would otherwise be
lost — the kind of information that fills whole conversation threads but rarely makes
it into "What Was Done" summaries.

**Scope**: Template change only; no LLM involved in handoff creation (stays offline).
A future `--guided` mode (LLM asks structured questions) is out of scope for this change.

### Decision: Multi-Project Aggregation

**What**: By default (no `--project` flag), `wai reflect` reads handoffs and artifacts
from ALL projects in the workspace and aggregates them. `--project <name>` scopes to
one project's artifacts only.

**Why**: CLAUDE.md and AGENTS.md are workspace-global files, so the reflected content
should represent the workspace as a whole. Project-specific gotchas from different
workstreams all benefit future AI sessions in the same repository.

**REFLECT block**: One block per output file, covering all projects. The LLM is
instructed to organize by concern, not by project, to avoid noise.

## Risks / Trade-offs

- **LLM quality depends on input quality**: Terse handoffs → generic output.
  Mitigation: improved template sections + conversation transcript option + user reviews
  diff before committing.

- **Stale reflections**: REFLECT content isn't automatically updated.
  Mitigation: nudge mechanism from wai close; diff-aware update shows what changed.

- **Prompt injection in artifacts**: Corrupted handoff or conversation transcript
  could attempt prompt injection.
  Mitigation: same triple-backtick escaping used in `wai why`.

- **Large conversation transcripts**: A long session could produce a 200K-char file.
  Mitigation: hard cap at ~30K chars, truncated from the top (oldest exchange removed).

## Reflect Block Format

```markdown
<!-- WAI:REFLECT:START -->
## Project-Specific AI Context
_Last reflected: 2026-02-24 · 12 sessions analyzed_

### Conventions
- ...

### Common Gotchas
- ...

### Steps That Tend to Require Multiple Tries
- ...

### Architecture Notes
- ...
<!-- WAI:REFLECT:END -->
```

The LLM determines which sections to include based on what it actually found.
The template above is a suggestion, not a constraint — the LLM may add or omit
sections. The `_Last reflected:_` line is always written by the tool, not the LLM.

## Improved Handoff Template

```markdown
---
date: YYYY-MM-DD
project: <name>
phase: <phase>
---

# Session Handoff

## What Was Done

## Key Decisions

## Gotchas & Surprises
<!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->

## What Took Longer Than Expected
<!-- Steps that needed multiple attempts. Commands that failed before the right one. -->

## Open Questions

## Next Steps

## Context
<!-- Plugin-enriched context (beads, git, etc.) -->
```
