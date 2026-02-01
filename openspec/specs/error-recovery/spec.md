# Error Recovery

## Purpose

Define self-healing error message patterns that suggest fixes instead of just reporting problems, making errors educational and actionable.

## Requirements

### Requirement: Diagnostic Error Format

All errors SHALL include a diagnostic code and actionable help text using miette.

#### Scenario: Error structure

- **WHEN** any error occurs
- **THEN** the error includes a code (e.g., `wai::project::not_initialized`)
- **AND** the error includes help text with a suggested fix

### Requirement: Project Not Initialized Error

When commands require a project context but none exists, the error SHALL suggest initialization.

#### Scenario: Missing project context

- **WHEN** user runs a project-scoped command outside a project
- **THEN** error message is "No project initialized in current directory"
- **AND** help suggests "Run `wai init` or `wai new project <name>` first"

### Requirement: Bead Not Found Error

When a referenced bead doesn't exist, the error SHALL suggest how to find valid beads.

#### Scenario: Invalid bead reference

- **WHEN** user references a bead ID that doesn't exist
- **THEN** error message is "Bead '{id}' not found"
- **AND** help suggests "Run `wai show beads` to see available beads"

### Requirement: Invalid Phase Transition Error

When a phase transition is invalid, the error SHALL show valid options.

#### Scenario: Invalid transition

- **WHEN** user attempts to move a bead to an invalid phase
- **THEN** error message is "Invalid phase transition from '{from}' to '{to}'"
- **AND** help lists valid target phases from the current phase

### Requirement: Plugin Not Found Error

When a plugin doesn't exist, the error SHALL suggest how to find available plugins.

#### Scenario: Missing plugin

- **WHEN** user references a plugin that isn't installed
- **THEN** error message is "Plugin '{name}' not found"
- **AND** help suggests "Run `wai show plugins --available` to see installable plugins"
