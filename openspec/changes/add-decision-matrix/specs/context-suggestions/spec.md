# Spec delta: context-suggestions

## MODIFIED Requirements

### Requirement: Phase-Based Suggestions

The status command SHALL provide contextual next-step suggestions based on the
current project phase. When the project is in the design phase **and** has a
decision matrix, `wai status` SHALL additionally show the matrix's current
problem statement, its cell completion count, and a suggestion for the next
unfilled cell.

#### Scenario: Research phase

- **WHEN** project is in "research" phase
- **THEN** suggest adding research notes: `wai add research "..."`
- **AND** suggest advancing when ready: `wai phase next`

#### Scenario: Plan phase

- **WHEN** project is in "plan" phase
- **THEN** suggest adding a plan: `wai add plan "..."`
- **AND** suggest advancing to design: `wai phase next`

#### Scenario: Design phase with a matrix

- **GIVEN** a project in the design phase with a matrix having 5 of 6 cells
  filled
- **WHEN** `wai status` runs
- **THEN** suggestions include the problem statement, `5/6 cells filled`,
  and a pointer to the unfilled cell

#### Scenario: Design phase without a matrix

- **GIVEN** a project in the design phase that never initialized a matrix
- **WHEN** `wai status` runs
- **THEN** no matrix information or suggestion is shown
