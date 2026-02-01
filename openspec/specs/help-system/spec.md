# Help System

## Purpose

Define progressive disclosure patterns for help and command output, showing simple information by default with more detail available on demand.

## Requirements

### Requirement: Basic Help

All commands SHALL provide `--help` with usage examples.

#### Scenario: Root help

- **WHEN** user runs `wai --help` or `wai -h`
- **THEN** the system shows a brief description of wai
- **AND** lists all top-level commands with one-line descriptions
- **AND** shows global flags (`-v`, `-q`, `--help`, `--version`)

#### Scenario: Command help

- **WHEN** user runs `wai <command> --help`
- **THEN** the system shows command description, arguments, and options
- **AND** includes at least one usage example

#### Scenario: Subcommand help

- **WHEN** user runs `wai <command> <subcommand> --help`
- **THEN** the system shows subcommand-specific description and options
- **AND** includes examples relevant to that subcommand

#### Scenario: No arguments shows help

- **WHEN** user runs `wai` with no arguments
- **THEN** the system shows the same output as `wai --help`

### Requirement: Usage Examples

All help output SHALL include practical usage examples.

#### Scenario: Command examples

- **WHEN** user views help for any command
- **THEN** the help includes an "Examples" section
- **AND** examples show realistic use cases with expected outcomes

#### Scenario: Example formatting

- **WHEN** examples are displayed
- **THEN** each example shows the command invocation
- **AND** optionally includes a brief explanation of what it does

### Requirement: Progressive Disclosure

Help SHALL support brief and detailed modes for different user needs.

#### Scenario: Brief help (default)

- **WHEN** user runs `wai <command> --help`
- **THEN** the system shows concise help focused on common usage
- **AND** omits advanced options and edge cases

#### Scenario: Detailed help

- **WHEN** user runs `wai <command> --help --verbose` or `wai <command> --help -v`
- **THEN** the system shows comprehensive help including all options
- **AND** includes advanced usage patterns and configuration details

#### Scenario: Help verbosity levels

- **WHEN** user increases verbosity (`-v`, `-vv`, `-vvv`)
- **THEN** help output includes progressively more detail:
  - `-v`: all options including advanced ones
  - `-vv`: adds environment variables and configuration
  - `-vvv`: adds internal details and debugging information
