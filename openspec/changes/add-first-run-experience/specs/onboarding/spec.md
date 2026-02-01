## ADDED Requirements

### Requirement: First-Run Detection

The CLI SHALL detect first-time users and offer a guided experience.

#### Scenario: First time running wai

- **WHEN** user runs wai for the first time (no prior config exists)
- **THEN** the system offers to run a quickstart tutorial
- **AND** shows "Run 'wai tutorial' to learn the basics"

#### Scenario: Returning user

- **WHEN** user has run wai before (config exists)
- **THEN** the system shows normal welcome/status without tutorial prompt

### Requirement: Quickstart Tutorial

The CLI SHALL provide an interactive tutorial command.

#### Scenario: Tutorial flow

- **WHEN** user runs `wai tutorial`
- **THEN** the system walks through: create project → create bead → move through phases
- **AND** each step explains what's happening
- **AND** user can exit at any time

#### Scenario: Tutorial completion

- **WHEN** user completes the tutorial
- **THEN** the system congratulates them
- **AND** suggests next steps for their real project

### Requirement: Guided Project Creation

The `wai init` command SHALL provide extra guidance for new users.

#### Scenario: Init with guidance

- **WHEN** user runs `wai init`
- **AND** this is their first project
- **THEN** the system explains what `.para/` contains
- **AND** shows what they can do next with examples

### Requirement: Example Workflows

Early outputs SHALL include real workflow examples, not just syntax.

#### Scenario: Welcome screen examples

- **WHEN** welcome screen is shown (no project)
- **THEN** output includes a "Quick example" showing a typical 3-command workflow
- **AND** commands are copy-pasteable
