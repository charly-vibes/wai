# managed-block Specification

## Purpose
TBD - created by archiving change add-managed-block-openspec-checklist. Update Purpose after archive.
## Requirements
### Requirement: Openspec step in session-close checklist

The generated WAI block's session-close checklist SHALL include a step to mark
completed openspec tasks `[x]` when the openspec plugin is detected. The step
SHALL appear after the beads steps (if present) and before `wai reflect`.

#### Scenario: Openspec detected at init time

- **WHEN** `wai init` or `wai reflect` runs
- **AND** the openspec plugin is detected
- **THEN** the generated WAI block's "Ending a Session" checklist includes:
  `[ ] openspec tasks.md — mark completed tasks [x]`

#### Scenario: Openspec not detected

- **WHEN** `wai init` or `wai reflect` runs
- **AND** the openspec plugin is NOT detected
- **THEN** the generated WAI block does NOT include the openspec checklist step

---

### Requirement: Cross-tool tracking convention

When both beads and openspec plugins are detected, the generated WAI block SHALL
include a "Tracking Work Across Tools" section that defines the cross-reference
convention for linking beads tickets to openspec task IDs.

The section SHALL appear after the "Capturing Work" section and before "Ending
a Session".

The convention SHALL specify:
- When creating a beads ticket for an openspec task, include the task reference
  (format: `<change-id>:<phase>.<task>`, e.g. `add-why-command:7.1`) in the
  ticket description.
- When closing a beads ticket linked to an openspec task, also check the
  corresponding box (`[x]`) in the change's `tasks.md`.

#### Scenario: Both plugins detected

- **WHEN** `wai init` or `wai reflect` runs
- **AND** both beads and openspec plugins are detected
- **THEN** the generated WAI block includes a "Tracking Work Across Tools"
  section with the cross-reference format and sync rule

#### Scenario: Only one plugin detected

- **WHEN** `wai init` or `wai reflect` runs
- **AND** only one of beads or openspec is detected (not both)
- **THEN** the "Tracking Work Across Tools" section is NOT included

---

### Requirement: Pre-claim implementation check

The generated WAI block's "Starting a Session" section SHALL include a
pre-claim guard (when beads is detected) reminding the agent to verify an issue
is not already implemented before claiming it.

#### Scenario: Beads detected — pre-claim note present

- **WHEN** `wai init` or `wai reflect` runs
- **AND** the beads plugin is detected
- **THEN** the "Starting a Session" section includes a note after `bd ready`
  that reads: before claiming an issue, read the relevant source files to
  confirm the work is not already done

#### Scenario: Beads not detected — no pre-claim note

- **WHEN** `wai init` or `wai reflect` runs
- **AND** the beads plugin is NOT detected
- **THEN** no pre-claim note is included

---

### Requirement: Epic closure reminder in session-close checklist

The generated WAI block's session-close checklist `bd close` step SHALL include
a note (when beads is detected) to also close the parent epic when the last
sub-task of a group is completed.

#### Scenario: Beads detected — epic note on close step

- **WHEN** `wai init` or `wai reflect` runs
- **AND** the beads plugin is detected
- **THEN** the `bd close <id>` checklist step includes an inline comment noting
  that parent epics should be closed when their last sub-task is done

#### Scenario: Beads not detected — no epic note

- **WHEN** `wai init` or `wai reflect` runs
- **AND** the beads plugin is NOT detected
- **THEN** no epic closure reminder is included

### Requirement: Available Pipelines in managed block

The generated WAI block SHALL include an "Available Pipelines" section when
pipelines with `[pipeline.metadata]` are installed, listing each pipeline's name,
`when` description, and start command. The section SHALL also note that pipeline
steps may have gates and reference `wai pipeline gates` for gate details.

#### Scenario: Pipelines with metadata installed

- **WHEN** `wai init` or `wai init --update` runs
- **AND** `.wai/resources/pipelines/` contains one or more pipelines with `[pipeline.metadata]`
- **THEN** the generated WAI block includes an "Available Pipelines" table with
  columns: Pipeline, When to Use, Start command
- **AND** a note: "Pipeline steps have gates that enforce artifact creation, review
  coverage, and oracle checks before advancement."

#### Scenario: No pipelines installed

- **WHEN** `wai init` runs
- **AND** no pipelines exist in `.wai/resources/pipelines/`
- **THEN** no "Available Pipelines" section is generated

#### Scenario: Pipeline without metadata

- **WHEN** a pipeline exists but lacks `[pipeline.metadata]`
- **THEN** that pipeline is NOT included in the "Available Pipelines" table
- **AND** `wai doctor` warns about the missing metadata

---

### Requirement: Managed block staleness detection

`wai doctor` SHALL detect when the managed block in CLAUDE.md or AGENTS.md is
out of date relative to the current workspace state (installed pipelines,
plugins, skills). Detection SHALL compare the generated block content against
the actual content between `WAI:START` and `WAI:END` markers.

#### Scenario: Block is stale after pipeline added

- **WHEN** `wai doctor` runs
- **AND** a pipeline with metadata has been added since the last `wai init --update`
- **AND** the CLAUDE.md managed block does not include that pipeline
- **THEN** doctor reports warning: "CLAUDE.md managed block outdated — run 'wai init --update' to refresh"

#### Scenario: Block is current

- **WHEN** `wai doctor` runs
- **AND** the managed block content matches what would be generated
- **THEN** no staleness warning is reported

#### Scenario: AGENTS.md also checked

- **WHEN** `wai doctor` runs
- **AND** AGENTS.md exists with WAI:START/WAI:END markers
- **THEN** staleness detection runs on AGENTS.md as well

#### Scenario: User-modified managed block always shows stale

- **WHEN** `wai doctor` runs
- **AND** the user has manually edited content between WAI:START and WAI:END markers
- **THEN** doctor reports the block as outdated
- **AND** running `wai init --update` will overwrite the user's modifications
  (content between markers is managed by wai)

