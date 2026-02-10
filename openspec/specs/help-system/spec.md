# Help System

## Purpose

Define progressive disclosure patterns for help and command output, showing simple information by default with more detail available on demand.

## Problem Statement

Effective command-line tools require an intuitive and helpful user assistance system. Without one, users face a steep learning curve, leading to frustration and inefficient use. This spec addresses the need for a help system that serves both audiences effectively by being clear, concise, and progressively more detailed on demand, representing a **Type 1 commitment** to an improved user experience. A "one-size-fits-all" help output is insufficient, being either too verbose for novices trying to perform a simple task, or too sparse for experts needing advanced details.

## Design Rationale

The design of the help system is centered on the principle of progressive disclosure, which represents a **Type 1 architectural decision** for help content structure and rendering logic, significantly enhancing usability for all user levels.

- **Progressive Disclosure:** By default, help is concise and focused on common use cases. Users can request more detail using verbosity flags (`-v`, `-vv`). This **commits to a layered content structure** and respects the user's attention, avoiding overwhelming them with irrelevant information while still making advanced details accessible. This pattern is consistent with the CLI's global verbosity flags.
- **In-line Usage Examples:** Including practical examples directly within the help output is a highly effective way to teach users how to perform common tasks, reducing the need to consult external documentation.
- **Adherence to Convention:** Using the standard `--help` and `-h` flags ensures that the help system is immediately familiar and predictable to anyone with prior CLI experience.

## Scope and Requirements

This spec defines the user-facing behavior and output structure of the CLI's help system.

### Non-Goals

- **Help Content Generation:** This spec defines the desired *output and behavior* of the help system. The internal mechanism for authoring and maintaining help text is an implementation detail not covered here.
- **Interactive Help Systems:** Features like a `man`-style pager, guided tutorials, or interactive prompts are out of scope.
- **Localization:** Translating help content into multiple languages is not covered by this specification.
- **Web-based Documentation:** This spec is limited to the help available directly within the CLI application.

## Requirements

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
