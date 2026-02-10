# Error Recovery

## Purpose

Define self-healing error message patterns that suggest fixes instead of just reporting problems, making errors educational and actionable.

## Problem Statement

Cryptic, unhelpful error messages are a major source of user frustration in command-line tools. When an error occurs and the application simply reports "failed" or "invalid input," it forces the user to halt their workflow, consult external documentation, and waste time diagnosing the issue. This increases cognitive load and creates a steep, frustrating learning curve. This spec represents a **Type 1 commitment** to transforming this experience by making errors educational and actionable, significantly enhancing user productivity and satisfaction.

## Design Rationale

The error recovery strategy for `wai` is to treat errors not as failures, but as opportunities to guide the user toward the correct action, representing **Type 1 decisions** that establish a core UX pattern and architectural dependency.

- **Actionable, "Self-Healing" Errors:** This is a **Type 1 decision** for `wai`'s core user experience. Instead of just reporting what went wrong, `wai` errors will explain the problem and suggest the specific command to fix it. This approach makes the tool feel more like a helpful assistant and less like a rigid instruction parser, significantly reducing user friction.
- **Use of `miette`:** The selection of the `miette` library is a **Type 1 architectural dependency**. It was chosen specifically for its ability to produce rich, diagnostic error messages with features like error codes, actionable help text, and code snippets, which are key components of our "self-healing" error philosophy.
- **Diagnostic Codes:** Every `wai`-specific error includes a stable, machine-readable code (e.g., `wai::project::not_found`). This is a **Type 1 decision** for error identification, aiding in debugging, enabling more specific documentation, and providing a reliable way to reference errors in tests or external tools.

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
- **THEN** the error includes a code (e.g., `wai::project::not_found`)
- **AND** the error includes help text with a suggested fix
- **AND** when `--json` is provided, the system outputs a JSON error object with `code`, `message`, `help`, and `details`

### Requirement: Project Not Initialized Error

When commands require a project context but none exists, the error SHALL suggest initialization.

#### Scenario: Missing project context

- **WHEN** user runs a project-scoped command outside a project
- **THEN** error message is "No project initialized in current directory"
- **AND** help suggests "Run `wai init` or `wai new project <name>` first"
- **AND** the diagnostic code is `wai::project::not_initialized`

### Requirement: Project Not Found Error

When a referenced project doesn't exist, the error SHALL suggest how to find valid projects.

#### Scenario: Invalid project reference

- **WHEN** user references a project name that doesn't exist
- **THEN** error message is "Project '{name}' not found"
- **AND** help suggests "Run `wai show projects` to see available projects"
- **AND** the diagnostic code is `wai::project::not_found`

### Requirement: Area Not Found Error

When a referenced area doesn't exist, the error SHALL suggest alternatives.

#### Scenario: Invalid area reference

- **WHEN** user references an area name that doesn't exist
- **THEN** error message is "Area '{name}' not found"
- **AND** help suggests "Run `wai show areas` to see available areas"
- **AND** the diagnostic code is `wai::area::not_found`

### Requirement: Resource Not Found Error

When a referenced resource doesn't exist, the error SHALL suggest alternatives.

#### Scenario: Invalid resource reference

- **WHEN** user references a resource name that doesn't exist
- **THEN** error message is "Resource '{name}' not found"
- **AND** help suggests "Run `wai show resources` to see available resources"
- **AND** the diagnostic code is `wai::resource::not_found`

### Requirement: Invalid Phase Transition Error

When a phase transition is invalid, the error SHALL show valid options.

#### Scenario: Invalid transition

- **WHEN** user attempts an invalid phase transition (e.g., already at the last phase and running `phase next`)
- **THEN** error message is "Invalid phase transition from '{from}' to '{to}'"
- **AND** help lists valid target phases from the current phase
- **AND** the diagnostic code is `wai::phase::invalid_transition`

### Requirement: Config Sync Error

When agent config sync fails, the error SHALL explain what went wrong and suggest remediation.

#### Scenario: Sync failure

- **WHEN** a projection fails during `wai sync`
- **THEN** error message describes which projection failed and why
- **AND** help suggests checking `.projections.yml` configuration
- **AND** the diagnostic code is `wai::config::sync_failed`

### Requirement: Handoff Error

When handoff generation fails, the error SHALL explain the issue.

#### Scenario: Handoff generation failure

- **WHEN** `wai handoff create` fails
- **THEN** error message describes the failure (e.g., project not found, template missing)
- **AND** help suggests the corrective action
- **AND** the diagnostic code is `wai::handoff::failed`

### Requirement: Plugin Not Found Error

When a plugin doesn't exist, the error SHALL suggest how to find available plugins.

#### Scenario: Missing plugin

- **WHEN** user references a plugin that isn't installed
- **THEN** error message is "Plugin '{name}' not found"
- **AND** help suggests "Run `wai plugin list` to see available plugins"
- **AND** the diagnostic code is `wai::plugin::not_found`
