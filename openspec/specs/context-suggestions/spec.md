# Context Suggestions

## Purpose

Define context-aware suggestion patterns that help users discover the next logical step based on current project state.

## Requirements

### Requirement: Status Suggestions

The status command SHALL provide contextual next-step suggestions based on current project state.

#### Scenario: Empty project

- **WHEN** project has no beads
- **THEN** suggest creating first bead

#### Scenario: Ready beads available

- **WHEN** project has beads in "ready" phase
- **THEN** suggest starting implementation

#### Scenario: In-progress beads

- **WHEN** project has beads in "in-progress" phase
- **THEN** suggest continuing work on active beads
- **AND** show which beads are currently in progress

#### Scenario: Blocked beads

- **WHEN** project has beads with unmet dependencies
- **THEN** suggest resolving blockers first
- **AND** show what is blocking each bead

#### Scenario: Completed project

- **WHEN** all beads are in "done" phase
- **THEN** suggest project wrap-up actions (archive, retrospective)

#### Scenario: Mixed states

- **WHEN** project has beads in multiple phases
- **THEN** prioritize suggestions: blocked > in-progress > ready > draft
