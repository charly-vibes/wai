# Context Suggestions

## Purpose

Define context-aware suggestion patterns that help users discover the next logical step based on current project state and phase.

## Problem Statement

A status command that only presents data often leaves the user with the question, "What should I do next?" Without proactive guidance, users experience decision fatigue, make suboptimal choices about where to focus effort, and lose workflow momentum. This spec proposes a **Type 1 commitment** to integrating a proactive suggestion system within the `wai status` command, bridging the gap between information and action, and making the user more effective.

## Design Rationale

The suggestion system is designed to be proactive, integrated, and based on project phase awareness, representing **Type 1 decisions** that establish a core UX pattern.

- **Integrated with `status`:** Placing suggestions directly within the `wai status` output is a deliberate **Type 1 decision**. It establishes a proactive UX pattern where `wai` guides the user immediately after reviewing the project's state, reducing cognitive load and driving workflow.
- **Phase-Based Priority:** Suggestions are driven by the current project phase (research, design, plan, implement, review, archive) and available plugin data. Each phase has natural next actions that wai can suggest contextually.

## Scope, Dependencies, and Requirements

This spec defines the logic for generating suggestions within the `wai status` command.

### Dependencies

- **Project State Machine:** The suggestion logic depends on the phase definitions outlined in the `project-state-machine` spec.
- **Plugin System:** Plugins (beads, git, openspec) provide additional context for richer suggestions.

### Non-Goals

- Implementation of suggested actions — suggestions are informational only
- Suggestions in commands other than `wai status`
- Exact UI/formatting — this spec defines logic, not presentation
## Requirements
### Requirement: Phase-Based Suggestions

The status command SHALL provide contextual next-step suggestions based on the current project phase.

#### Scenario: Research phase

- **WHEN** project is in "research" phase
- **THEN** suggest adding research notes: `wai add research "..."`
- **AND** suggest advancing when ready: `wai phase next`

#### Scenario: Plan phase

- **WHEN** project is in "plan" phase
- **THEN** suggest adding a plan: `wai add plan "..."`
- **AND** suggest advancing to design: `wai phase next`

#### Scenario: Design phase

- **WHEN** project is in "design" phase
- **THEN** suggest adding designs: `wai add design "..."`
- **AND** suggest advancing to implementation: `wai phase next`

#### Scenario: Implement phase

- **WHEN** project is in "implement" phase
- **THEN** suggest creating a handoff when pausing: `wai handoff create`
- **AND** suggest advancing to review: `wai phase next`

#### Scenario: Review phase

- **WHEN** project is in "review" phase
- **THEN** suggest completing and archiving: `wai phase next`
- **AND** suggest going back if issues found: `wai phase back`

#### Scenario: Archive phase

- **WHEN** project is in "archive" phase
- **THEN** suggest starting a new project: `wai new project`

### Requirement: Plugin-Enhanced Suggestions

When plugins are active, suggestions SHALL incorporate plugin context.

#### Scenario: Beads plugin active

- **WHEN** beads plugin reports open issues
- **THEN** status includes issue count and suggests reviewing: `wai beads list`

#### Scenario: Git plugin active

- **WHEN** git plugin reports uncommitted changes
- **THEN** status includes change summary

### Requirement: Empty Project Suggestions

When a project has no artifacts, suggestions SHALL guide initial setup.

#### Scenario: New empty project

- **WHEN** project has no research, plans, or designs
- **THEN** suggest starting with research: `wai add research "..."`
- **AND** suggest checking phase: `wai phase`

### Requirement: Pipeline Suggestions

When pipelines are configured, suggestions SHALL surface pipeline discovery and recovery actions.

#### Scenario: Active pipeline run

- **WHEN** a pipeline run is active (resolved from `WAI_PIPELINE_RUN` env var or `.wai/resources/pipelines/.last-run`)
- **THEN** status emits a "⚡ PIPELINE ACTIVE: `<name>` step N/M" line in a Pipeline section
- **AND** suggests reprinting the current step: `wai pipeline current`

#### Scenario: Pipelines available but no active run

- **WHEN** no active pipeline run exists
- **AND** `.wai/resources/pipelines/` contains at least one valid `.toml` pipeline definition
- **THEN** status emits an "Available pipelines" section listing each pipeline with name, description, and step count
- **AND** suggests discovering pipelines: `wai pipeline suggest`

#### Scenario: Stale pipeline pointer

- **WHEN** `.last-run` pointer exists but the referenced run file is missing
- **THEN** the stale pointer is silently ignored
- **AND** status falls back to the "available pipelines" or idle state

### Requirement: Suggestion Output Blocks

Status output SHALL include a clearly labeled suggestion block for human and machine parsing.

#### Scenario: Suggestion block formatting

- **WHEN** status suggestions are shown
- **THEN** they appear under a `Suggestions:` heading
- **AND** each suggestion includes a short label and a suggested command

#### Scenario: Suggestions as JSON

- **WHEN** user runs `wai status --json`
- **THEN** the suggestions are returned as an array with `label` and `command` fields

### Requirement: Stale Phase Detection

The status command SHALL detect when a project has not advanced phases in more
than 14 days and surface a suggestion to either advance or archive.

Detection criteria:
- Current phase is NOT `archive`
- Time elapsed since the current phase started exceeds 14 days
- Applies to all non-archive phases (research, design, plan, implement, review)

#### Scenario: Project stuck in implement phase

- **WHEN** a project is in `implement` phase
- **AND** the phase started more than 14 days ago
- **THEN** `wai status` includes a stale-phase suggestion
- **AND** the suggestion offers `wai phase next` to advance
- **AND** the suggestion offers `wai move <name> archives` to abandon

#### Scenario: Active project within threshold is not flagged

- **WHEN** a project's current phase started 13 days ago
- **THEN** `wai status` does NOT include a stale-phase suggestion for that project

#### Scenario: Archived project is not flagged

- **WHEN** a project is in `archive` phase regardless of elapsed time
- **THEN** `wai status` does NOT include a stale-phase suggestion for that project

#### Scenario: Stale signal in JSON output

- **WHEN** the user runs `wai status --json` and a project is stale
- **THEN** the suggestions array includes an entry with a label indicating staleness
- **AND** includes commands for advancing or archiving

---

### Requirement: Completion Readiness Signal

The status command SHALL detect when a project in `review` phase has at least
one handoff artifact and surface a suggestion to archive it.

Detection criteria:
- Current phase is `review`
- At least one handoff artifact exists in the project's `handoffs/` directory

#### Scenario: Review phase with handoff suggests archiving

- **WHEN** a project is in `review` phase
- **AND** the project has at least one handoff artifact
- **THEN** `wai status` includes a completion-readiness suggestion
- **AND** the suggestion offers `wai move <name> archives`
- **AND** the suggestion offers `wai phase next` as an alternative

#### Scenario: Review phase without handoff does not trigger

- **WHEN** a project is in `review` phase
- **AND** the project has zero handoff artifacts
- **THEN** `wai status` does NOT include a completion-readiness suggestion for that project

#### Scenario: Completion signal in JSON output

- **WHEN** the user runs `wai status --json` and a project looks complete
- **THEN** the suggestions array includes an entry indicating the project is
  ready to archive

