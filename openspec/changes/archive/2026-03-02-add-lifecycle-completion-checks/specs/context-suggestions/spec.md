## ADDED Requirements

### Requirement: Stale Phase Detection

The status command SHALL detect when a project has not advanced phases in more
than 14 days and surface a suggestion to either advance or archive.

Detection criteria:
- Current phase is NOT `archive`
- Time elapsed since the current phase started exceeds 14 days
- Applies to all non-archive phases (research, design, plan, implement, review)

#### Scenario: Project stuck in implement phase

- **WHEN** a project is in `implement` phase
- **AND** the phase started more than 14 days ago
- **THEN** `wai status` includes a stale-phase suggestion
- **AND** the suggestion offers `wai phase next` to advance
- **AND** the suggestion offers `wai move <name> archives` to abandon

#### Scenario: Active project within threshold is not flagged

- **WHEN** a project's current phase started 13 days ago
- **THEN** `wai status` does NOT include a stale-phase suggestion for that project

#### Scenario: Archived project is not flagged

- **WHEN** a project is in `archive` phase regardless of elapsed time
- **THEN** `wai status` does NOT include a stale-phase suggestion for that project

#### Scenario: Stale signal in JSON output

- **WHEN** the user runs `wai status --json` and a project is stale
- **THEN** the suggestions array includes an entry with a label indicating staleness
- **AND** includes commands for advancing or archiving

---

### Requirement: Completion Readiness Signal

The status command SHALL detect when a project in `review` phase has at least
one handoff artifact and surface a suggestion to archive it.

Detection criteria:
- Current phase is `review`
- At least one handoff artifact exists in the project's `handoffs/` directory

#### Scenario: Review phase with handoff suggests archiving

- **WHEN** a project is in `review` phase
- **AND** the project has at least one handoff artifact
- **THEN** `wai status` includes a completion-readiness suggestion
- **AND** the suggestion offers `wai move <name> archives`
- **AND** the suggestion offers `wai phase next` as an alternative

#### Scenario: Review phase without handoff does not trigger

- **WHEN** a project is in `review` phase
- **AND** the project has zero handoff artifacts
- **THEN** `wai status` does NOT include a completion-readiness suggestion for that project

#### Scenario: Completion signal in JSON output

- **WHEN** the user runs `wai status --json` and a project looks complete
- **THEN** the suggestions array includes an entry indicating the project is
  ready to archive
