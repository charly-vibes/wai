# Bead Lifecycle

## Purpose

Define the phases and valid state transitions for beads, providing a clear workflow from initial idea through completion.

## Problem Statement

Without a clearly defined and enforced lifecycle, the status of individual "beads" (work items) can become ambiguous. This leads to confusion among team members about what needs to be done, inconsistent tracking of progress, and difficulties in reporting accurate project status. The absence of structured transitions also makes it easy for work items to be mismanaged, leading to lost effort and project delays. A formalized bead lifecycle is essential for maintaining clarity, consistency, and effective project execution.

## Design Rationale

The chosen four-phase lifecycle model (Draft, Ready, In-Progress, Done) and strict transition rules are designed to balance simplicity with sufficient granularity for effective work item management.

-   **Four-Phase Model:** This model is widely adopted in various project management methodologies, offering a clear progression from initial concept to completion without over-complicating the workflow. Each phase represents a distinct and meaningful state of a bead.
-   **Strict Transition Rules:** Enforcing valid forward and backward transitions ensures data integrity and prevents beads from entering illogical states. This predictability aids in automation and reporting, maintaining a consistent understanding of a bead's journey.

## Scope and Requirements

This spec exclusively defines the states and transitions within the bead lifecycle.

### Non-Goals

-   **Bead Creation and Deletion:** The mechanics of how beads are initially created or ultimately removed from the system are covered by other specifications (e.g., CLI Core).
-   **Specific CLI/UI Commands for Transitions:** While transitions are defined here, the user-facing commands or UI elements to trigger these transitions are defined elsewhere (e.g., `cli-core`'s `move` verb).
-   **Role-Based Permissions:** This spec does not define access control or permissions for who can perform state transitions.
-   **Notifications or Automated Actions:** Automated responses or notifications triggered by phase changes are out of scope.
-   **Complex Workflow Branching:** This spec defines a linear progression with limited backward transitions; complex branching workflows are not supported.

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
