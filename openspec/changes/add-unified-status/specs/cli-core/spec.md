## MODIFIED Requirements

### Requirement: Status Command

The CLI SHALL provide `wai status` to show project overview, OpenSpec implementation status, and suggest next steps.

#### Scenario: Show project phase and status

- **WHEN** user runs `wai status`
- **THEN** the system displays the current project's phase
- **AND** shows detected plugin statuses
- **AND** shows contextual suggestions based on current phase

#### Scenario: Show OpenSpec summary

- **WHEN** user runs `wai status` and an `openspec/` directory exists
- **THEN** the system displays the count of implemented specs and proposed changes
- **AND** lists each active change with its task completion count (e.g. `[3/15]`)

#### Scenario: Show OpenSpec verbose detail

- **WHEN** user runs `wai status -v` and an `openspec/` directory exists
- **THEN** the system lists all implemented spec names
- **AND** for each active change, shows per-section task completion (e.g. `1. CLI Wiring [0/3]`)

#### Scenario: Graceful skip without OpenSpec

- **WHEN** user runs `wai status` and no `openspec/` directory exists
- **THEN** the OpenSpec section is omitted entirely

#### Scenario: Contextual suggestions

See [context-suggestions](../context-suggestions/spec.md) for the complete suggestion logic.
