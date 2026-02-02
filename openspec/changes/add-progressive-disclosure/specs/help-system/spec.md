# Spec: Progressive Disclosure for All Command Output

## Purpose

Define a system of progressive disclosure for the output of all CLI commands, and an "examples-first" approach for help text, to better serve users of all experience levels.

## Problem Statement

A "one-size-fits-all" output for CLI commands serves no one well. Novice users are often overwhelmed by technical details they don't need, making the tool feel intimidating. Expert users, on the other hand, are starved of the deep diagnostic information they require for debugging complex issues. Furthermore, traditional help pages that lead with syntax force users to learn the "how" before they understand the "what" and "why" from a practical example.

## Design Rationale

This design introduces two core principles: tailoring command output to the user's intent and structuring help around practical examples.

- **Intent-Based Verbosity:** The verbosity (`-v`) flag should control the level of detail in the *user-facing* output. This allows a user to specify whether they want a simple confirmation or a more detailed report of the outcome.
- **Separation of Concerns (Logs vs. Output):** Deeply technical diagnostics (execution traces, state machine transitions) are invaluable for debugging but are noise for most users. This information should not be tied to the `-v` flag, but instead controlled by a separate mechanism, such as a `--debug` flag or a `WAI_LOG` environment variable. This keeps the primary output clean and user-focused.
- **Examples-First Help:** Users learn most effectively by seeing a command in action. By placing practical, workflow-oriented examples at the top of help pages, we lower the cognitive barrier to entry and allow users to get started much faster.

## Scope and Requirements

This spec defines an architectural, cross-cutting change to command output and help text structure.

### Non-Goals

- **Immediate Implementation on All Commands:** The verbosity levels are an architectural pattern that should be applied to new commands, and retrofitted onto existing commands opportunistically. A single PR will not implement this everywhere.
- **Specific Logging Framework:** This spec separates the *idea* of diagnostic logging from user-facing output, but does not mandate a specific logging library or implementation.

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
