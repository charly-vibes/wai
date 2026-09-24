# Spec delta: handoff-system

## MODIFIED Requirements

### Requirement: Handoff Format

Handoffs SHALL be markdown files with structured frontmatter, including dedicated
sections for operational nuances that serve as high-signal input to `wai reflect`.
When the project has a decision matrix with a recorded decision, the handoff
SHALL additionally include a decision-matrix summary: the problem statement, the
selected approach with rationale, and links to both the matrix directory and the
scaffolded design doc.

#### Scenario: Handoff file structure

- **WHEN** a handoff is generated
- **THEN** it follows this format:
  ```markdown
  ---
  date: 2026-01-22
  project: my-project
  phase: implement
  agent: claude
  ---

  # Session Handoff

  ## What Was Done
  <!-- Summary of completed work -->

  ## Key Decisions
  <!-- Decisions made and rationale -->

  ## Gotchas & Surprises
  <!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->

  ## What Took Longer Than Expected
  <!-- Steps that needed multiple attempts. Commands that failed before the right one. -->

  ## Open Questions
  <!-- Unresolved questions -->

  ## Next Steps
  <!-- Prioritized list of what to do next -->

  ## Context
  <!-- Plugin-enriched context (beads, git, etc.) -->
  ```

#### Scenario: Handoff includes matrix summary

- **GIVEN** a project with a matrix whose `decision.md` records a decision
- **WHEN** a handoff is generated
- **THEN** it includes the problem statement, the selected approach with
  rationale, and links to the matrix directory and the design doc

#### Scenario: Handoff without a matrix

- **GIVEN** a project that never initialized a matrix
- **WHEN** a handoff is generated
- **THEN** no matrix section is added and the format is unchanged
