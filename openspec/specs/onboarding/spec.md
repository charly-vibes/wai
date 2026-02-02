# Onboarding

## Purpose

Define the first-run experience and welcome flow that helps new users get started quickly with wai.

This spec owns the no-args welcome behavior for both first-run (no project) and in-project contexts.

## Problem Statement

The initial user experience with a new command-line tool can be a significant barrier to adoption. If a user runs `wai` for the first time without arguments and is met with a cryptic error, an overwhelming help page, or an unhelpful generic message, it can lead to frustration, confusion, and ultimately, abandonment of the tool. Even experienced users inside a project might run `wai` without arguments and benefit from quick, context-relevant reminders or next steps rather than a full help display.

## Design Rationale

The onboarding experience is designed to be intuitive, context-aware, and minimalist, providing immediate value without unnecessary complexity.

-   **Context-Aware "No-Args" Behavior:** Tailoring the output based on whether a project exists (`.wai/` directory) provides immediate, relevant guidance. For first-time users, it suggests how to begin; for in-project users, it offers quick access to common next actions. This avoids generic responses and reduces user friction.
-   **Minimalist Guidance:** Instead of an interactive wizard or verbose documentation, the system offers direct, actionable command suggestions. This respects the CLI user's preference for efficiency and direct control, allowing them to quickly engage with the tool.

## Scope and Requirements

This spec exclusively defines the output and suggested actions when the `wai` command is executed without any arguments.

### Non-Goals

-   **Full Implementation of Suggested Commands:** This spec outlines what to suggest, but not the detailed implementation of commands like `wai init` or `wai new project`.
-   **Interactive Setup Wizards:** The onboarding process is non-interactive, relying on clear textual suggestions.
-   **Complex User Preference Tracking:** This spec does not cover tracking user preferences or progress beyond detecting the presence of a project.

## Requirements

### Requirement: Welcome Screen

When wai is run without arguments and no project is detected, it SHALL show a welcoming entry point.

#### Scenario: No project detected

- **WHEN** user runs `wai` with no arguments
- **AND** no `.wai/` directory exists in current or parent directories
- **THEN** the system shows "wai - Workflow manager for AI-driven development"
- **AND** shows context: "No project detected in current directory"
- **AND** suggests: `wai init`, `wai new project`, `wai --help`

### Requirement: Project Context Welcome

When wai is run without arguments inside a project, it SHALL show project-relevant suggestions.

#### Scenario: Inside project

- **WHEN** user runs `wai` with no arguments
- **AND** a `.wai/` directory exists
- **THEN** the system suggests: `wai status`, `wai show beads`, `wai new bead`
