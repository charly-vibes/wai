# Spec: Enhanced Onboarding Experience

## Purpose

Define a rich, interactive first-run experience that actively guides new users through the core `wai` workflow, building confidence and accelerating adoption.

## Problem Statement

While a basic welcome message is a good start, it is often insufficient for users new to a workflow-oriented tool like `wai`. Passive suggestions and help text still require the user to learn and synthesize information on their own. To truly overcome the initial learning curve, users need a more hands-on, guided experience that demonstrates the core value proposition of the tool without requiring them to read extensive documentation.

## Design Rationale

The proposed onboarding experience is based on the "learn-by-doing" principle.

- **Interactive Tutorial:** A built-in, interactive tutorial (`wai tutorial`) is the most effective way to teach the core workflow. It keeps the user in their terminal, allows them to run actual commands in a safe environment, and builds muscle memory. This is superior to passive video or web tutorials as it provides direct, hands-on experience.
- **Proactive Guidance:** The system proactively detects first-time users and offers guidance at key moments, such as during project initialization (`wai init`), ensuring they understand the structure and purpose of the tool from the beginning.

## Scope and Requirements

This spec covers the high-level design and flow of an enhanced, interactive onboarding process.

### Non-Goals

- **Exact Tutorial Script:** The detailed, word-for-word content of the tutorial is a content design task and is not defined in this technical spec.
- **Advanced User Tracking:** "First-run" detection is based on a simple configuration check, not complex user analytics.
- **Graphical Tutorial:** The entire experience is text-based and confined to the terminal.

## Requirements

### Requirement: First-Run Detection

The CLI SHALL detect first-time users and offer a guided experience.

#### Scenario: First time running wai

- **WHEN** user runs wai for the first time (no prior config exists)
- **THEN** the system offers to run a quickstart tutorial
- **AND** shows "Run 'wai tutorial' to learn the basics"

#### Scenario: Returning user

- **WHEN** user has run wai before (config exists)
- **THEN** the system shows normal welcome/status without tutorial prompt

### Requirement: Quickstart Tutorial

The CLI SHALL provide an interactive tutorial command.

#### Scenario: Tutorial flow

- **WHEN** user runs `wai tutorial`
- **THEN** the system walks through: create project → create bead → move through phases
- **AND** each step explains what's happening
- **AND** user can exit at any time

#### Scenario: Tutorial completion

- **WHEN** user completes the tutorial
- **THEN** the system congratulates them
- **AND** suggests next steps for their real project

### Requirement: Guided Project Creation

The `wai init` command SHALL provide extra guidance for new users.

#### Scenario: Init with guidance

- **WHEN** user runs `wai init`
- **AND** this is their first project
- **THEN** the system explains what `.para/` contains
- **AND** shows what they can do next with examples

### Requirement: Example Workflows

Early outputs SHALL include real workflow examples, not just syntax.

#### Scenario: Welcome screen examples

- **WHEN** welcome screen is shown (no project)
- **THEN** output includes a "Quick example" showing a typical 3-command workflow
- **AND** commands are copy-pasteable
