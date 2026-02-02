# Error Recovery

## Purpose

Define self-healing error message patterns that suggest fixes instead of just reporting problems, making errors educational and actionable.

## Problem Statement

Cryptic, unhelpful error messages are a major source of user frustration in command-line tools. When an error occurs and the application simply reports "failed" or "invalid input," it forces the user to halt their workflow, consult external documentation, and waste time diagnosing the issue. This increases cognitive load and creates a steep, frustrating learning curve. This spec represents a **Type 1 commitment** to transforming this experience by making errors educational and actionable, significantly enhancing user productivity and satisfaction.

## Design Rationale

The error recovery strategy for `wai` is to treat errors not as failures, but as opportunities to guide the user toward the correct action, representing **Type 1 decisions** that establish a core UX pattern and architectural dependency.

- **Actionable, "Self-Healing" Errors:** This is a **Type 1 decision** for `wai`'s core user experience. Instead of just reporting what went wrong, `wai` errors will explain the problem and suggest the specific command to fix it. This approach makes the tool feel more like a helpful assistant and less like a rigid instruction parser, significantly reducing user friction.
- **Use of `miette`:** The selection of the `miette` library is a **Type 1 architectural dependency**. It was chosen specifically for its ability to produce rich, diagnostic error messages with features like error codes, actionable help text, and code snippets, which are key components of our "self-healing" error philosophy.
- **Diagnostic Codes:** Every `wai`-specific error includes a stable, machine-readable code (e.g., `wai::project::not_initialized`). This is a **Type 1 decision** for error identification, aiding in debugging, enabling more specific documentation, and providing a reliable way to reference errors in tests or external tools.

## Scope and Requirements

This spec defines the user-facing format and content for common, recoverable errors in `wai`.

### Non-Goals

- **Internal Error Logging:** This spec focuses on the *user-facing error experience* and does not cover how errors are logged internally for developer analysis.
- **Automated Fixes:** The system will only *suggest* corrective actions; it will not execute commands automatically on the user's behalf.
- **Localization:** All error messages will be presented in a single language.
- **Exhaustive Error Catalog:** This document specifies the pattern for key errors but is not an exhaustive list of every possible error condition; rather, it defines the *approach* to error messaging.

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
