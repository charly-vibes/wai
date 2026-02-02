# Spec: Progressive Disclosure for All Command Output

## Purpose

Define a system of progressive disclosure for the output of all CLI commands, and an "examples-first" approach for help text, to better serve users of all experience levels.

## Problem Statement

A "one-size-fits-all" output for CLI commands fundamentally fails to serve the diverse needs of its users. Novice users are often overwhelmed by technical details they don't need, making the tool feel intimidating. Expert users, on the other hand, are starved of the deep diagnostic information required for debugging complex issues. This architectural problem leads to a fragmented user experience. Furthermore, traditional help pages that prioritize syntax over practical examples hinder user adoption and understanding.

## Design Rationale

This design introduces two core principles: tailoring command output to the user's intent and structuring help around practical examples. These represent **Type 1 architectural decisions** that redefine how `wai` communicates with its users.

- **Intent-Based Verbosity (User-Facing Output):** The verbosity (`-v`) flag is dedicated to controlling the level of detail in the *user-facing* output. This is a **Type 1 decision** for the CLI's interaction pattern, allowing users to specify whether they want a simple confirmation or a more detailed report of the outcome without being overwhelmed by internal diagnostics.
- **Separation of Concerns (Logs vs. Output):** This is a critical **Type 1 correction** to avoid conflating user-facing detail with internal debugging information. Deeply technical diagnostics (execution traces, state machine transitions) are invaluable for developers but are noise for most users. This information will be controlled by a separate mechanism (e.g., a `--debug` flag or `WAI_LOG` environment variable), keeping the primary output clean and user-focused.
- **Examples-First Help:** Users learn most effectively by seeing a command in action. This is a **Type 1 decision** for help content structure. By placing practical, workflow-oriented examples at the top of help pages, we lower the cognitive barrier to entry and allow users to get started much faster.

## Scope and Requirements

This spec defines an architectural, cross-cutting change to command output and help text structure.

### Non-Goals

- **Immediate Implementation on All Commands:** The progressive disclosure for command output is an **architectural pattern**. While it should be applied to new commands, retrofitting it onto existing commands will be done opportunistically and iteratively. This represents a **phased approach** to a cross-cutting concern.
- **Specific Logging Framework:** This spec separates the *idea* of diagnostic logging from user-facing output, but does not mandate a specific logging library or implementation, allowing flexibility in the underlying technical choices.

## Requirements

### Requirement: User-Facing Verbosity Levels

The CLI SHALL support two levels of user-facing output detail for its commands.

#### Scenario: Default Output (Beginner)

- **WHEN** user runs a command without `-v`
- **THEN** output is a simple, high-level summary of the result
- **AND** may include a suggestion for the next logical step.
- **EXAMPLE:** `Bead 'my-bead' created.`

#### Scenario: Detailed Output (Intermediate)

- **WHEN** user runs a command with `-v`
- **THEN** output includes a more detailed log of the user-relevant outcomes.
- **AND** might list files created or modified, or key value changes.
- **EXAMPLE:** `✔ Bead 'my-bead' created at ./beads/my-bead.md`

### Requirement: Diagnostic Trace Mode

The CLI SHALL provide a separate mechanism for deep diagnostic tracing for experts and developers.

#### Scenario: Debug Mode

- **WHEN** a command is run with a `--debug` flag (or equivalent)
- **THEN** output includes a full execution trace, state machine transitions, performance timings, etc.
- **NOTE:** This output is intended for developers and is not considered part of the primary user interface.

### Requirement: Examples-First Help

Help pages SHALL show usage examples before explaining syntax.

#### Scenario: Command help format

- **WHEN** user runs `wai <command> --help`
- **THEN** the help shows a "COMMON EXAMPLES" section first.
- **AND** the detailed syntax and flag descriptions ("USAGE") follow the examples.

### Requirement: Main Help Overview

The main help SHALL show common workflows prominently.

#### Scenario: Main help

- **WHEN** user runs `wai --help`
- **THEN** help shows a "QUICK START" section with a common command workflow.
- **AND** the full command list follows this introductory section.
