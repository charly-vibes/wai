## ADDED Requirements

### Requirement: Post-Command Suggestions

After each command, the CLI SHALL suggest logical next steps based on what just happened.

#### Scenario: After creating project

- **WHEN** user successfully runs `wai init` or `wai new project`
- **THEN** output suggests: create first bead, add research, check status

#### Scenario: After adding research

- **WHEN** user successfully adds research
- **THEN** output suggests: create bead from research, add more research, show beads

#### Scenario: After moving bead to in-progress

- **WHEN** user moves a bead to in-progress phase
- **THEN** output suggests: show bead details, add notes, complete bead

### Requirement: Workflow Pattern Detection

The CLI SHALL detect common workflow patterns and tailor suggestions.

#### Scenario: Implementation phase detected

- **WHEN** project has beads in ready phase
- **AND** no beads are in-progress
- **THEN** status command highlights "Ready to implement" with specific beads

#### Scenario: Research phase detected

- **WHEN** project has draft beads but no research
- **THEN** status command suggests adding research before moving to ready

### Requirement: Interactive Ambiguity Resolution

When a command is ambiguous, the CLI SHALL prompt for clarification instead of failing.

#### Scenario: Multiple matching beads

- **WHEN** user references a bead with an ambiguous identifier
- **AND** multiple beads match
- **THEN** the system shows a selection prompt with matching options
- **AND** user can choose or cancel

#### Scenario: Non-interactive mode

- **WHEN** user passes --no-interactive
- **AND** a command would normally prompt
- **THEN** the system returns an error with all matching options listed
