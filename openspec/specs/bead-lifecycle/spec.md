# Bead Lifecycle

## Purpose

Define the phases and valid state transitions for beads, providing a clear workflow from initial idea through completion.

## Requirements

### Requirement: Phase Definitions

Beads SHALL exist in one of four phases that represent workflow progress.

#### Scenario: Draft phase

- **WHEN** a bead is created
- **THEN** it starts in the "draft" phase
- **AND** draft indicates the bead is not yet ready for implementation

#### Scenario: Ready phase

- **WHEN** a bead has been refined and is implementation-ready
- **THEN** it can be moved to "ready" phase
- **AND** ready indicates all requirements and acceptance criteria are defined

#### Scenario: In-progress phase

- **WHEN** work has started on a bead
- **THEN** it is in "in-progress" phase
- **AND** in-progress indicates active development

#### Scenario: Done phase

- **WHEN** a bead's implementation is complete and verified
- **THEN** it is in "done" phase
- **AND** done indicates all acceptance criteria are met

### Requirement: Valid Phase Transitions

The system SHALL enforce valid phase transitions to maintain workflow integrity.

#### Scenario: Forward transitions

- **WHEN** moving a bead forward in the workflow
- **THEN** valid transitions are:
  - draft → ready
  - ready → in-progress
  - in-progress → done

#### Scenario: Backward transitions

- **WHEN** moving a bead backward in the workflow
- **THEN** valid transitions are:
  - ready → draft (needs more refinement)
  - in-progress → ready (work paused, not started)
  - done → in-progress (reopened due to issues)

#### Scenario: Invalid transitions

- **WHEN** user attempts an invalid transition (e.g., draft → done)
- **THEN** the system rejects the transition
- **AND** shows valid target phases from the current phase

### Requirement: Phase Change Tracking

The system SHALL track phase changes for audit and metrics.

#### Scenario: Transition recorded

- **WHEN** a bead changes phase
- **THEN** the system records the previous phase, new phase, and timestamp
- **AND** the change is visible in bead history
