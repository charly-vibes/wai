# Bead Lifecycle

## Purpose

Define the phases and valid state transitions for beads, providing a clear workflow from initial idea through completion.

## Problem Statement

Without a clearly defined and enforced lifecycle, the status of individual "beads" (work items) becomes ambiguous, leading to a **lack of predictability and consistency**. This results in confusion about priorities, inconsistent tracking of progress, and difficulties in accurate status reporting. Such ambiguity makes it easy for work items to be mismanaged, causing lost effort and project delays. A formalized bead lifecycle is therefore a **Type 1 foundational decision** for `wai`, essential for establishing clarity, consistency, and effective project execution.

## Design Rationale

The chosen four-phase lifecycle model (Draft, Ready, In-Progress, Done) and strict transition rules represent a **Type 1 foundational decision** for `wai`. This design intentionally embraces an **opinionated workflow** that trades extreme flexibility for clarity, predictability, and consistency.

-   **Four-Phase Model:** This model, widely adopted in project management, offers a clear progression from concept to completion. By adopting this fixed model, `wai` ensures each phase represents a distinct and meaningful state, forming a predictable backbone for all operations.
-   **Strict Transition Rules:** Enforcing valid forward and backward transitions is critical. This predictability ensures data integrity, prevents illogical bead states, and is the cornerstone for building reliable automation, accurate reporting, and context-aware suggestions. This **intentional rigidity** creates a "pit of success" for users.

## Scope and Requirements

This spec exclusively defines the states and transitions within the bead lifecycle.

### Non-Goals

-   **Bead Creation and Deletion:** The mechanics of how beads are initially created or ultimately removed from the system are covered by other specifications (e.g., CLI Core).
-   **Specific CLI/UI Commands for Transitions:** While transitions are defined here, the user-facing commands or UI elements to trigger these transitions are defined elsewhere (e.g., `cli-core`'s `move` verb).
-   **Role-Based Permissions:** This spec does not define access control or permissions for who can perform state transitions.
-   **Notifications or Automated Actions:** Automated responses or notifications triggered by phase changes are out of scope.
-   **Complex Workflow Branching:** This spec intentionally defines a linear progression with limited backward transitions. Complex, user-defined branching workflows are explicitly not supported, reinforcing `wai`'s **opinionated core workflow**.

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
