# Project Reflection

## Purpose

Define the `wai reflect` command, which synthesizes accumulated session context
into project-specific AI-assistant guidance and injects it into CLAUDE.md and/or
AGENTS.md as a persistent, project-aware REFLECT block.

## Problem Statement

As projects accumulate sessions and artifacts (research, designs, plans, handoffs),
valuable project-specific conventions, gotchas, and architectural patterns remain
buried in session history. The CLAUDE.md and AGENTS.md managed blocks provide generic
tool instructions but contain nothing project-specific — every new AI session must
rediscover the same codebase patterns from scratch.

`wai why` answers specific questions reactively. `wai reflect` complements it
proactively: given accumulated session context, ask the LLM "what project-specific
context should AI assistants know that isn't already documented?" and inject the
answer into CLAUDE.md and/or AGENTS.md as a persistent, project-aware section.

## Design Rationale

### Three-Tier Context Model

Context sources are prioritized from richest to most distilled:
1. **Conversation transcripts** — raw Claude session content (ephemeral unless captured)
2. **Handoffs** — structured session summaries with gotchas and next steps
3. **Artifacts** — research, design, and plan documents

### LLM-Required (No Fallback)

Unlike `wai why`, `wai reflect` does not fall back to search. Reflection requires
LLM synthesis to surface non-obvious patterns; keyword search cannot substitute.

### Managed Block Injection

The REFLECT block uses a separate `WAI:REFLECT:START` / `WAI:REFLECT:END` marker
pair, distinct from the existing `WAI:START` / `WAI:END` tool-instructions block,
so the two can evolve independently.

## Scope and Requirements

This spec covers the `wai reflect` command, REFLECT block management, context
gathering, and the session nudge heuristic.

### Non-Goals

- Streaming LLM responses during reflection
- Automatic reflection without user confirmation
- Reflection of non-wai project history

## Requirements

### Requirement: Reflect Command

The CLI SHALL provide `wai reflect` to synthesize accumulated session context
into project-specific AI-assistant guidance and inject it into CLAUDE.md and/or
AGENTS.md.

#### Scenario: Basic reflect run

- **WHEN** user runs `wai reflect`
- **THEN** the system reads all handoff files from all projects as the primary source
- **AND** reads research, design, and plan artifacts as secondary sources
- **AND** reads the current content of any existing REFLECT block from the target file(s)
- **AND** sends a structured prompt to the configured LLM (reusing `[why]` config)
- **AND** displays a unified diff of old vs proposed REFLECT block content
- **AND** prompts the user to confirm before writing, naming the target file(s)

#### Scenario: Conversation transcript as input

- **WHEN** user runs `wai reflect --conversation <file>`
- **THEN** the system reads the file as plain text (any format accepted)
- **AND** includes it as the highest-priority context source in the LLM prompt
- **AND** caps the transcript at ~30K chars, truncating from the beginning (oldest
  exchange removed) to preserve the most recent content
- **AND** proceeds with handoffs and artifacts filling the remaining context budget

#### Scenario: Output target selection

- **WHEN** user runs `wai reflect` without `--output`
- **THEN** the system detects which of CLAUDE.md and AGENTS.md exist in the repo root
- **AND** updates all files that exist (both, if both are present)

- **WHEN** user runs `wai reflect --output claude.md`
- **THEN** the system updates only CLAUDE.md, regardless of what exists

- **WHEN** user runs `wai reflect --output agents.md`
- **THEN** the system updates only AGENTS.md, regardless of what exists

- **WHEN** user runs `wai reflect --output both`
- **THEN** the system updates both CLAUDE.md and AGENTS.md

#### Scenario: No target file found

- **WHEN** user runs `wai reflect` and neither CLAUDE.md nor AGENTS.md exist
- **THEN** the system fails with a diagnostic error:
  "No CLAUDE.md or AGENTS.md found — run `wai init` first or create the target file"

#### Scenario: Dry run mode

- **WHEN** user runs `wai reflect --dry-run`
- **THEN** the system displays the unified diff of old vs proposed content
- **AND** exits without writing to any file
- **AND** indicates that no files were modified

#### Scenario: No changes needed

- **WHEN** the proposed REFLECT block content is identical to the existing content
- **THEN** the system prints "Reflect block is already up to date" and exits 0
- **AND** does NOT prompt the user or modify any file

#### Scenario: Auto-confirm

- **WHEN** user runs `wai reflect --yes`
- **THEN** the system skips the confirmation prompt
- **AND** writes the proposed REFLECT block to the target file(s) directly

#### Scenario: Scoped to a project

- **WHEN** user runs `wai reflect --project <name>`
- **THEN** the system reads only artifacts from the specified project
- **AND** proceeds with LLM synthesis and injection as normal

#### Scenario: No LLM available

- **WHEN** user runs `wai reflect` and no LLM backend is configured or reachable
- **THEN** the system displays an informational message explaining that LLM is required
- **AND** suggests configuring `[why]` in `.wai/config.toml` or installing Ollama
- **AND** does NOT fall back to search (reflection requires LLM synthesis)

#### Scenario: No handoffs and no conversation file

- **WHEN** user runs `wai reflect` with no `--conversation` file
- **AND** the workspace has no handoff artifacts
- **THEN** the system warns that no session context is available
- **AND** suggests running `wai handoff create <project>` to capture session context
- **AND** exits without calling the LLM

### Requirement: Reflect Managed Block

The system SHALL maintain a dedicated `WAI:REFLECT` managed block in CLAUDE.md
and/or AGENTS.md, separate from the existing `WAI` tool-instructions block.

#### Scenario: First reflect run — block created

- **WHEN** user confirms a reflect run and the target file does not contain a REFLECT block
- **THEN** the system appends the following structure:
  - In CLAUDE.md: after the `WAI:END` marker (if present), otherwise at end of file
  - In AGENTS.md: at end of file
  ```markdown
  <!-- WAI:REFLECT:START -->
  ## Project-Specific AI Context
  _Last reflected: YYYY-MM-DD · N sessions analyzed_

  [LLM-generated sections]
  <!-- WAI:REFLECT:END -->
  ```
- **AND** all existing file content outside the markers is preserved

#### Scenario: Subsequent reflect run — block updated with diff confirmation

- **WHEN** user runs `wai reflect` and the target file already contains a REFLECT block
- **THEN** the system shows a unified diff of the old block vs. proposed new block
- **AND** prompts for confirmation before writing
- **AND** on confirm, replaces only the content between `WAI:REFLECT:START` and
  `WAI:REFLECT:END`
- **AND** the `_Last reflected:_` metadata line is updated with the current date
  and session count
- **AND** all file content outside the markers is preserved

#### Scenario: Reflect block read-back for deduplication

- **WHEN** building the LLM prompt for a reflect run
- **AND** the target file contains an existing REFLECT block
- **THEN** the system includes the existing block content in the prompt
- **AND** instructs the LLM to add new patterns, correct stale ones, and omit
  duplicates already covered

### Requirement: Context Sources and Budget

The system SHALL gather context in a three-tier priority order with dynamic
budget allocation.

#### Scenario: Conversation transcript fills first tier

- **WHEN** `--conversation <file>` is provided
- **THEN** the transcript is the highest-priority source in the LLM prompt
- **AND** capped at ~30K chars, truncated from the beginning if larger
- **AND** the LLM prompt labels this source: "Conversation transcript (richest:
  raw session including failed attempts and surprises)"

#### Scenario: Handoffs fill second tier

- **WHEN** gathering context (with or without conversation transcript)
- **THEN** handoff files are read sorted by modification time (newest first)
- **AND** fill up to ~40K chars of the budget (or the remaining budget after
  the conversation transcript)
- **AND** the LLM prompt labels these: "Session handoffs (intent, next steps,
  gotchas captured at session end)"

#### Scenario: Secondary artifacts fill remaining budget

- **WHEN** context budget remains after handoffs
- **THEN** research, design, and plan artifacts fill the remaining ~30K chars
- **AND** the LLM prompt labels these: "Research and design artifacts (explicit
  decisions and domain knowledge)"

#### Scenario: Budget exceeded

- **WHEN** total content exceeds the budget (~100K chars)
- **THEN** each tier is truncated to its cap from the least-recent end
- **AND** the prompt includes a note that context was truncated

#### Scenario: Staleness flag

- **WHEN** the LLM prompt includes artifacts
- **THEN** each artifact is labeled with its creation date
- **AND** the prompt instructs the LLM to flag patterns from artifacts older than
  6 months as "potentially stale — verify still applies"

### Requirement: Reflect Metadata

The system SHALL track reflection run metadata in a dedicated file to support
the session nudge heuristic.

#### Scenario: Metadata written after successful reflect

- **WHEN** `wai reflect` successfully writes to one or more target files
- **THEN** the system writes `.wai/projects/<project>/.reflect-meta` containing:
  ```toml
  last_reflected = "YYYY-MM-DD"
  session_count = N
  ```
  where N is the number of handoff files processed in this run
- **AND** the file is created if absent or overwritten if present

#### Scenario: Missing metadata treated as never reflected

- **WHEN** `wai close` or `wai reflect` reads `.reflect-meta` and the file does not exist
- **THEN** the system treats `last_reflected` as the epoch (all handoffs are newer)

### Requirement: Session Nudge

The system SHALL suggest running `wai reflect` from `wai close` when enough
sessions have accumulated without a reflect.

#### Scenario: Nudge after threshold exceeded

- **WHEN** user runs `wai close` and creates a handoff
- **AND** 5 or more handoff files have mtime newer than the `last_reflected` date
  in `.wai/projects/<project>/.reflect-meta`
- **THEN** the system appends a suggestion after normal close output:
  `→ N sessions since last reflect — run 'wai reflect' to update <target>`
  where `<target>` lists the AI config files found in the repo root
- **AND** the nudge is informational only (does not block close or change exit code)

#### Scenario: No nudge when current

- **WHEN** user runs `wai close`
- **AND** fewer than 5 handoffs are newer than the last reflect
- **THEN** the system does NOT display the reflect nudge

### Requirement: Improved Handoff Nuance Capture

The handoff template SHALL include dedicated sections for capturing operational
nuances that are most valuable as reflect input.

#### Scenario: Handoff includes nuance sections

- **WHEN** user runs `wai handoff create <project>`
- **THEN** the generated handoff includes these sections (in addition to existing ones):
  ```markdown
  ## Gotchas & Surprises
  <!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->

  ## What Took Longer Than Expected
  <!-- Steps that needed multiple attempts. Commands that failed before the right one. -->
  ```
- **AND** both sections are empty template placeholders for the user to fill in

#### Scenario: Reflect prioritizes nuance sections

- **WHEN** building the LLM prompt and handoffs contain Gotchas or Longer-Than-Expected sections
- **THEN** the LLM prompt instructs the LLM to treat content in those sections as
  high-signal input for the Conventions and Common Gotchas sections of the REFLECT block
