# Context Suggestions

## Purpose

Define context-aware suggestion patterns that help users discover the next logical step based on current project state.

## Problem Statement

A status command that only presents data often leaves the user with the question, "What should I do next?" Without guidance, users may experience decision fatigue or make suboptimal choices about where to focus their effort. This leads to inefficient workflows and reduced momentum. A proactive suggestion system can bridge the gap between information and action, making the user more effective.

## Design Rationale

The suggestion system is designed to be proactive, integrated, and based on a clear workflow priority.

- **Integrated with `status`:** Placing suggestions directly within the `wai status` output is intentional. It answers the user's implicit next question immediately after they review the project's state, requiring no extra commands.
- **Urgency-Based Priority:** The suggestion priority (`blocked > in-progress > ready > draft`) is based on a sound project management heuristic. It guides the user to first unblock the project, then continue existing work to maintain momentum, and finally to start new work.

## Scope, Dependencies, and Requirements

This spec defines the logic for generating suggestions within the `wai status` command.

### Dependencies

-   **Bead Lifecycle:** The suggestion logic depends on the phase definitions (`draft`, `ready`, `in-progress`, `done`) outlined in the `bead-lifecycle` spec.
-   **Dependency Tracking:** The "Blocked beads" scenario requires a mechanism to define and track dependencies between beads. The implementation of that tracking system is outside the scope of this document.

### Non-Goals

-   **Implementation of Dependency Tracking:** This spec only consumes the "blocked" state; it does not define how that state is determined.
-   **Implementation of Wrap-up Actions:** The `archive` and `retrospective` actions are suggested, but their implementation is not covered here.
-   **Suggestions in Other Commands:** This system is limited to the `wai status` command.
-   **Exact UI/Formatting:** This spec defines the logic, not the precise color or formatting of the output.

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
