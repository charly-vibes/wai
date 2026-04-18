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

#### Scenario: Explicit conversation transcript fills first tier
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
