# Help System

## Purpose

Define progressive disclosure patterns for help and command output, showing simple information by default with more detail available on demand.

## Requirements

### Requirement: Basic Help

All commands SHALL provide `--help` with usage examples.

#### Scenario: Command help

- **WHEN** user runs `wai <command> --help`
- **THEN** the system shows command description, arguments, and options
- **AND** includes at least one usage example
