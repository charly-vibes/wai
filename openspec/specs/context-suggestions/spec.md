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
