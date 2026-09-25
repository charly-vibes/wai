# Project State Machine

## Purpose

Define the phase-based state machine for projects, providing a structured workflow from initial research through completion and archival.

## Problem Statement

Development projects progress through natural phases — from research and planning through design, implementation, and review. Without a formalized state model, the current phase of a project becomes ambiguous, making it difficult to provide contextual suggestions, generate meaningful handoffs, or track progress over time. A state machine provides the structure needed for phase-aware tooling while remaining flexible enough for real-world workflows where phases may be revisited.

## Design Rationale

### Six-Phase Model

The six-phase model (research → design → plan → implement → review → archive) is a **Type 1 foundational decision**. Unlike the simpler four-phase bead lifecycle it replaces at the project level, this model captures the full arc of a development effort:

- **Research**: Gathering context, understanding the problem space
- **Design**: Creating technical designs, API contracts, architecture
- **Plan**: Defining implementation steps, scope, and success criteria after design direction is selected
- **Implement**: Writing code, building features
- **Review**: Testing, code review, validation against requirements
- **Archive**: Completed work preserved for reference

### Flexible Transitions

Unlike strict linear workflows, wai allows **flexible transitions** — users can skip forward or go back as needed. This is a deliberate **Type 2 decision** that prioritizes developer autonomy over process enforcement. The state machine tracks transitions for auditability without preventing valid workflow patterns.

### File-Based State

Project state is stored as a YAML `.state` file within the project directory. This keeps state co-located with the project's artifacts and naturally version-controllable.

## Scope and Requirements

This spec defines the state machine phases, transitions, and persistence format for projects.

### Non-Goals

- Automated phase transitions based on external signals
- Phase-specific validation rules (e.g., requiring certain artifacts before advancing)
- Multi-project phase coordination
- Custom user-defined phases

### Requirement: Phase Definitions

Projects SHALL progress through defined phases that represent workflow stages.

#### Scenario: Default phase

- **WHEN** a new project is created
- **THEN** it starts in the "research" phase

#### Scenario: Phase list

- **WHEN** querying available phases
- **THEN** the system returns: research, design, plan, implement, review, archive

### Requirement: Phase Transitions

The system SHALL support flexible phase transitions with history tracking.

#### Scenario: Advance to next phase

- **WHEN** user runs `wai phase next`
- **THEN** the project moves to the next sequential phase
- **AND** the transition is recorded with timestamp

#### Scenario: Set specific phase

- **WHEN** user runs `wai phase set <phase>`
- **THEN** the project moves to the specified phase
- **AND** the transition is recorded with timestamp

#### Scenario: Go back to previous phase

- **WHEN** user runs `wai phase back`
- **THEN** the project moves to the previous sequential phase
- **AND** the transition is recorded with timestamp

#### Scenario: Show current phase

- **WHEN** user runs `wai phase`
- **THEN** the system displays the current phase and phase history

### Requirement: State Persistence

Project phase state SHALL be persisted in a YAML file within the project directory.

#### Scenario: State file format

- **WHEN** a project has phase state
- **THEN** it is stored in `.wai/projects/<name>/.state` as:
  ```yaml
  current: plan
  history:
    - phase: research
      started: 2026-01-20T10:00:00Z
      completed: 2026-01-22T15:30:00Z
    - phase: plan
      started: 2026-01-22T15:30:00Z
  ```

### Requirement: Phase History

The system SHALL maintain a complete history of phase transitions.

#### Scenario: Transition recorded

- **WHEN** a project changes phase
- **THEN** the system records the previous phase completion time
- **AND** records the new phase start time
- **AND** the history is visible via `wai phase`
## Requirements
### Requirement: Matrix-Aware Design to Plan Transition

The system SHALL gate the `design → plan` phase transition on matrix currency
when a project has a decision matrix (`designs/matrix/`): `problem.md` must be
non-empty, `decision.md` must record a selected approach that exists under
`approaches/`, and no file under `approaches/` or `criteria/` may be newer
than the decision UTC timestamp recorded inside `decision.md`. Projects
without a matrix are never gated by it. The gate applies to every
`design → plan` transition path, including `wai phase next` and
`wai phase set plan`.

#### Scenario: Matrix gates design-to-plan when undecided

- **GIVEN** a project with a decision matrix whose `decision.md` is still the
  scaffolded template
- **WHEN** the `design → plan` transition is attempted
- **THEN** the transition is blocked
- **AND** the failure message indicates no decision has been made and
  suggests `wai matrix decide`

#### Scenario: Matrix gates design-to-plan when stale

- **GIVEN** a project with a decision matrix whose cell or criterion files
  were modified after the recorded decision timestamp
- **WHEN** the `design → plan` transition is attempted
- **THEN** the transition is blocked
- **AND** the failure message names the offending files and suggests
  `wai matrix decide`

#### Scenario: Inconclusive staleness warns instead of blocking

- **GIVEN** a decided matrix checked out fresh from git so that all matrix
  files share the checkout timestamp with the decision timestamp
- **WHEN** the `design → plan` transition is attempted
- **THEN** the transition proceeds
- **AND** a stale-decision warning is reported with a pointer to
  `wai matrix lint`

#### Scenario: Set cannot bypass the gate

- **GIVEN** a project in the design phase with an undecided matrix
- **WHEN** user runs `wai phase set plan`
- **THEN** the transition is blocked with the same message as
  `wai phase next`

#### Scenario: No matrix leaves transitions unchanged

- **GIVEN** a project that never initialized a matrix
- **WHEN** the `design → plan` transition is attempted
- **THEN** the transition behaves exactly as before this change

