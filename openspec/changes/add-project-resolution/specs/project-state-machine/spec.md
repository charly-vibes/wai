## MODIFIED Requirements

### Requirement: Phase Transitions

The system SHALL support flexible phase transitions with history tracking.
Phase commands SHALL resolve the target project via the unified project
resolution algorithm.

#### Scenario: Advance to next phase

- **WHEN** user runs `wai phase next`
- **THEN** the resolved project moves to the next sequential phase
- **AND** the transition is recorded with timestamp

#### Scenario: Advance specific project via flag

- **WHEN** user runs `wai phase next --project <name>`
- **THEN** the named project moves to the next sequential phase
- **AND** other projects are unaffected

#### Scenario: Set specific phase

- **WHEN** user runs `wai phase set <phase>`
- **THEN** the resolved project moves to the specified phase
- **AND** the transition is recorded with timestamp

#### Scenario: Go back to previous phase

- **WHEN** user runs `wai phase back`
- **THEN** the resolved project moves to the previous sequential phase
- **AND** the transition is recorded with timestamp

#### Scenario: Show current phase

- **WHEN** user runs `wai phase`
- **THEN** the system displays the resolved project's current phase and phase history
- **AND** shows a source indicator if project was resolved via flag or env var

#### Scenario: Phase with WAI_PROJECT

- **WHEN** `WAI_PROJECT` is set to a valid project name
- **AND** user runs `wai phase next` without `--project` flag
- **THEN** the system advances the phase of the project named by `WAI_PROJECT`
- **AND** the output includes `[via WAI_PROJECT]` next to the project name

#### Scenario: Flag overrides WAI_PROJECT

- **WHEN** `WAI_PROJECT` is set to project A
- **AND** user runs `wai phase next --project B`
- **THEN** the system advances the phase of project B, not project A
