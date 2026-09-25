# Spec delta: project-state-machine

## ADDED Requirements

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
