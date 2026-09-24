# Spec delta: project-state-machine

## MODIFIED Requirements

### Requirement: Phase Transitions

The system SHALL support flexible phase transitions with history tracking.
When a project has a decision matrix (`designs/matrix/`), the `design → plan`
transition SHALL additionally be gated on matrix currency: `problem.md` must
be non-empty, `decision.md` must exist, and no file under `approaches/` or
`criteria/` may be newer than `decision.md`. Projects without a matrix are
never gated by it.

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

#### Scenario: Matrix gates design-to-plan when stale

- **GIVEN** a project with a decision matrix whose cell or criterion files
  were modified after `decision.md`
- **WHEN** the `design → plan` transition is attempted
- **THEN** the transition is blocked
- **AND** the failure message names the offending files and suggests
  `wai matrix decide`

#### Scenario: No matrix leaves transitions unchanged

- **GIVEN** a project that never initialized a matrix
- **WHEN** the `design → plan` transition is attempted
- **THEN** the transition behaves exactly as before this change
