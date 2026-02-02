# Context Suggestions

## Purpose

Define context-aware suggestion patterns that help users discover the next logical step based on current project state.

## Problem Statement

A status command that only presents data often leaves the user with the question, "What should I do next?" Without proactive guidance, users experience decision fatigue, make suboptimal choices about where to focus effort, and lose workflow momentum. This spec proposes a **Type 1 commitment** to integrating a proactive suggestion system within the `wai status` command, bridging the gap between information and action, and making the user more effective.

## Design Rationale

The suggestion system is designed to be proactive, integrated, and based on a clear workflow priority, representing **Type 1 decisions** that establish a core UX pattern.

- **Integrated with `status`:** Placing suggestions directly within the `wai status` output is a deliberate **Type 1 decision**. It establishes a proactive UX pattern where `wai` guides the user immediately after reviewing the project's state, reducing cognitive load and driving workflow.
- **Urgency-Based Priority:** The suggestion priority (`blocked > in-progress > ready > draft`) is a **Type 1 decision** based on a sound project management heuristic. This strict prioritization guides the user to address the most impactful items first, fostering efficient workflow and allowing users to build trust in `wai`'s recommendations.

## Scope, Dependencies, and Requirements

This spec defines the logic for generating suggestions within the `wai status` command.

### Dependencies

-   **Bead Lifecycle:** The suggestion logic depends on the phase definitions (`draft`, `ready`, `in-progress`, `done`) outlined in the `bead-lifecycle` spec.
-   **Dependency Tracking:** The "Blocked beads" scenario introduces a **critical Type 1 dependency** on a robust mechanism to define and track dependencies between beads. The effectiveness and "intelligence" of the suggestion system fundamentally rely on the accurate implementation of this external system, which is explicitly outside the scope of *this* document.

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
