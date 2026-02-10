## MODIFIED Requirements

### Requirement: Phase-Based Suggestions

The status command SHALL provide contextual next-step suggestions based on the current project phase.

#### Scenario: Research phase

- **WHEN** project is in "research" phase
- **THEN** suggest adding research notes: `wai add research "..."`
- **AND** suggest advancing when ready: `wai phase next`

#### Scenario: Plan phase

- **WHEN** project is in "plan" phase
- **THEN** suggest adding a plan: `wai add plan "..."`
- **AND** suggest advancing to design: `wai phase next`

#### Scenario: Design phase

- **WHEN** project is in "design" phase
- **THEN** suggest adding designs: `wai add design "..."`
- **AND** suggest advancing to implementation: `wai phase next`

#### Scenario: Implement phase

- **WHEN** project is in "implement" phase
- **THEN** suggest creating a handoff when pausing: `wai handoff create`
- **AND** suggest advancing to review: `wai phase next`

#### Scenario: Review phase

- **WHEN** project is in "review" phase
- **THEN** suggest completing and archiving: `wai phase next`
- **AND** suggest going back if issues found: `wai phase back`

#### Scenario: Archive phase

- **WHEN** project is in "archive" phase
- **THEN** suggest starting a new project: `wai new project`

### Requirement: Suggestion Output Blocks

Status output SHALL include a clearly labeled suggestion block for human and machine parsing.

#### Scenario: Suggestion block formatting

- **WHEN** status suggestions are shown
- **THEN** they appear under a `Suggestions:` heading
- **AND** each suggestion includes a short label and a suggested command

#### Scenario: Suggestions as JSON

- **WHEN** user runs `wai status --json`
- **THEN** the suggestions are returned as an array with `label` and `command` fields
