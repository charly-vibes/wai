# CLI Core

## Purpose

Define the core command structure and patterns for the wai CLI, including the verb-noun command hierarchy, global flags, and foundational commands like init and status.

See also: onboarding spec for first-run and no-args welcome behavior.

## Requirements

### Requirement: Command Structure

The CLI SHALL use consistent verb-noun command patterns with four primary verbs: `new`, `add`, `show`, `move`.

#### Scenario: Create new items

- **WHEN** user runs `wai new project <name>` or `wai new bead <title>`
- **THEN** the system creates the requested item

#### Scenario: Add items to context

- **WHEN** user runs `wai add research <content>` or `wai add plugin <name>`
- **THEN** the system adds the item to the current project

#### Scenario: Show information

- **WHEN** user runs `wai show project`, `wai show beads`, or `wai show phase`
- **THEN** the system displays the requested information

#### Scenario: Move items between states

- **WHEN** user runs `wai move bead <id> --to <phase>`
- **THEN** the system moves the bead to the target phase

### Requirement: Global Flags

The CLI SHALL support global verbosity and quiet flags that work with all commands.

#### Scenario: Verbose output (level 1)

- **WHEN** user passes `-v` or `--verbose`
- **THEN** output includes additional context and metadata

#### Scenario: Verbose output (level 2)

- **WHEN** user passes `-vv` or `--verbose --verbose`
- **THEN** output includes debug information

#### Scenario: Verbose output (level 3)

- **WHEN** user passes `-vvv` or `--verbose --verbose --verbose`
- **THEN** output includes trace-level details

#### Scenario: Quiet mode

- **WHEN** user passes `-q` or `--quiet`
- **THEN** only errors are shown

### Requirement: Project Initialization

The CLI SHALL provide `wai init` to initialize a project in the current directory.

#### Scenario: Interactive initialization

- **WHEN** user runs `wai init` without arguments
- **THEN** the system prompts for project name (defaulting to directory name)
- **AND** creates `.para/` structure with config, beads, research, and plugins directories

#### Scenario: Named initialization

- **WHEN** user runs `wai init --name my-project`
- **THEN** the system creates the project with the specified name without prompting

#### Scenario: Already initialized

- **WHEN** user runs `wai init` in an already-initialized directory
- **THEN** the system shows a warning and suggests `wai status`

### Requirement: Status Command

The CLI SHALL provide `wai status` to show project overview and suggest next steps.

#### Scenario: Show bead counts

- **WHEN** user runs `wai status`
- **THEN** the system displays counts by phase (draft, ready, in-progress, done)

#### Scenario: Contextual suggestions

See [context-suggestions](../context-suggestions/spec.md) for the complete suggestion logic.
