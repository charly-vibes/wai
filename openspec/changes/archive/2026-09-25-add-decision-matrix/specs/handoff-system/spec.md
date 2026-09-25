# Spec delta: handoff-system

## ADDED Requirements

### Requirement: Handoff Decision-Matrix Summary

When the project has a decision matrix, handoffs SHALL additionally include a
decision-matrix summary: the problem statement, the selected approach with
rationale (or "not decided yet"), the matrix directory link with cell
completion count, and the scaffolded design doc link when a decision exists.

#### Scenario: Handoff includes matrix summary

- **GIVEN** a project with a matrix whose `decision.md` records a decision
- **WHEN** a handoff is generated
- **THEN** it includes the problem statement, the selected approach with
  rationale, and links to the matrix directory and the design doc

#### Scenario: Handoff without a matrix

- **GIVEN** a project that never initialized a matrix
- **WHEN** a handoff is generated
- **THEN** no matrix section is added and the format is unchanged
