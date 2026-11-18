## MODIFIED Requirements
### Requirement: Reflect Command
The CLI SHALL provide `wai reflect` to synthesize accumulated session context into project-specific AI-assistant guidance and write the result to a versioned reflection resource.

#### Scenario: Basic reflect run
- **WHEN** user runs `wai reflect`
- **THEN** the system reads all handoff files from all projects as the primary source
- **AND** reads research, design, and plan artifacts as secondary sources
- **AND** reads previous reflections from `.wai/resources/reflections/` as additional context
- **AND** sends a structured prompt to the configured LLM (reusing `[llm]` config)
- **AND** writes the result to `.wai/resources/reflections/<date>-<project>.md`
- **AND** prints the path of the written resource file

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
- **THEN** the system shows the resource file path that would be written
- **AND** exits without writing to any file
- **AND** still performs migration of any old WAI:REFLECT block if present

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

#### Scenario: Auto-detected local trace as input
- **WHEN** user runs `wai reflect`
- **AND** no `--conversation <file>` is provided
- **AND** a supported local trace exists for the current repository
- **THEN** the system uses the highest-ranked recent local trace as the highest-priority context source
- **AND** reports the selected source tool and session identifier before LLM synthesis

#### Scenario: Diff-only trace used for reflection
- **WHEN** the selected local trace has fidelity `diff-only`
- **THEN** the system includes only source metadata, touched files, timestamps, and diff evidence in the reflection prompt
- **AND** labels the source as reduced-fidelity context
- **AND** instructs the LLM not to infer missing conversation intent as fact

### Requirement: Context Sources and Budget
The system SHALL gather context in a three-tier priority order with dynamic budget allocation.

#### Scenario: Conversation transcript fills first tier
- **WHEN** `--conversation <file>` is provided
- **THEN** the transcript is the highest-priority source in the LLM prompt
- **AND** capped at ~30K chars, truncated from the beginning if larger
- **AND** the LLM prompt labels this source: "Conversation transcript (richest: raw session including failed attempts and surprises)"

#### Scenario: Auto-detected trace fills first tier when explicit transcript absent
- **WHEN** no `--conversation <file>` is provided
- **AND** a local trace is selected automatically
- **THEN** the selected trace becomes the highest-priority source in the LLM prompt
- **AND** the prompt labels the source with its fidelity tier and tool name

#### Scenario: No explicit transcript and no local trace
- **WHEN** user runs `wai reflect`
- **AND** no `--conversation <file>` is provided
- **AND** no local trace is found for the current repository
- **THEN** the system falls back to handoffs as the highest-priority source

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
