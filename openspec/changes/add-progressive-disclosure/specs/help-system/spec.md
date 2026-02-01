## ADDED Requirements

### Requirement: Verbosity Levels

The CLI SHALL support three verbosity levels controlled by the -v flag.

#### Scenario: Beginner mode (default)

- **WHEN** user runs any command without -v flags
- **THEN** output shows simple success message
- **AND** shows 3-4 next step suggestions
- **AND** includes link to examples or docs

#### Scenario: Intermediate mode

- **WHEN** user runs any command with -v
- **THEN** output shows detailed execution log
- **AND** shows plugin hooks that ran
- **AND** lists files created or modified
- **AND** shows contextual next steps

#### Scenario: Expert mode

- **WHEN** user runs any command with -vv or higher
- **THEN** output shows full execution trace
- **AND** shows state machine transitions
- **AND** shows file system operations
- **AND** shows performance metrics (timing)

### Requirement: Examples-First Help

Help pages SHALL show usage examples before explaining syntax.

#### Scenario: Command help format

- **WHEN** user runs `wai <command> --help`
- **THEN** the help shows "Examples:" section first
- **AND** examples demonstrate common workflows
- **AND** syntax/options follow examples

### Requirement: Main Help Overview

The main help SHALL show common workflows prominently.

#### Scenario: Main help

- **WHEN** user runs `wai --help`
- **THEN** help shows "Quick Start" section with common commands
- **AND** groups commands by workflow stage
