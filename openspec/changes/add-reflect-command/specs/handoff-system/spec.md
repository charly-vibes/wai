## MODIFIED Requirements

### Requirement: Handoff Format

Handoffs SHALL be markdown files with structured frontmatter, including dedicated
sections for operational nuances that serve as high-signal input to `wai reflect`.

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
