## MODIFIED Requirements

### Requirement: Basic Help

All commands SHALL provide `--help` with usage examples.

#### Scenario: Root help

- **WHEN** user runs `wai --help` or `wai -h`
- **THEN** the system shows a brief description of wai
- **AND** lists all top-level commands with one-line descriptions
- **AND** shows global flags (`-v`, `-q`, `--help`, `--version`, `--json`, `--no-input`, `--yes`, `--safe`)

#### Scenario: Command help

- **WHEN** user runs `wai <command> --help`
- **THEN** the system shows command description, arguments, and options
- **AND** includes at least one usage example

#### Scenario: Subcommand help

- **WHEN** user runs `wai <command> <subcommand> --help`
- **THEN** the system shows subcommand-specific description and options
- **AND** includes examples relevant to that subcommand

#### Scenario: No arguments shows contextual help

- **WHEN** user runs `wai` with no arguments
- **THEN** the system defers to onboarding behavior for context-aware output
