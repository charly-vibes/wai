# Spec: Advanced Contextual Suggestions & Interaction

## Purpose

Define a suite of advanced features to make the CLI more interactive and assistive, including post-command suggestions, workflow pattern detection, and interactive ambiguity resolution.

## Problem Statement

A command-line interface can often feel like a series of disconnected, atomic operations. Users are left to figure out the next logical step on their own, interrupting their flow. Furthermore, when a command is ambiguous (e.g., a name matches multiple items), the tool typically fails, forcing a frustrating cycle of listing items and re-running the command. This creates a rigid, unforgiving user experience.

## Design Rationale

This proposal aims to evolve `wai` from a simple instruction parser into a more helpful, interactive assistant.

- **Post-Command Suggestions:** By suggesting next steps after a successful command, the CLI guides the user through a natural workflow, reducing cognitive load and improving discoverability of features.
- **Interactive Ambiguity Resolution:** Instead of failing, the tool should help the user succeed. When ambiguity is detected, prompting the user with a list of choices turns a moment of failure into an efficient, successful interaction. The `--no-interactive` flag provides an escape hatch for scripting.
- **Workflow Pattern Detection:** This is a move towards a more intelligent agent that can recognize the user's context at a higher level (e.g., "it looks like you're in a research phase") and provide more tailored, insightful guidance.

## Scope and Requirements

This spec covers the design for several advanced interactive features.

### Non-Goals

- **Suggestions for all commands:** The initial implementation will focus on a few high-value commands, not the entire command suite.
- **Detecting all workflow anti-patterns:** The system will only be designed to detect a small, well-defined set of common workflow patterns.
- **Resolving all types of ambiguity:** The initial focus for interactive resolution is on bead selection, not all possible forms of ambiguity.
- **Natural Language Processing (NLP):** All detection and suggestion logic will be based on defined heuristics, not complex NLP.

## Requirements

### Requirement: Post-Command Suggestions

After each command, the CLI SHALL suggest logical next steps based on what just happened.

#### Scenario: After creating project

- **WHEN** user successfully runs `wai init` or `wai new project`
- **THEN** output suggests: create first bead, add research, check status

#### Scenario: After adding research

- **WHEN** user successfully adds research
- **THEN** output suggests: create bead from research, add more research, show beads

#### Scenario: After moving bead to in-progress

- **WHEN** user moves a bead to in-progress phase
- **THEN** output suggests: show bead details, add notes, complete bead

### Requirement: Workflow Pattern Detection

The CLI SHALL detect common workflow patterns and tailor suggestions.

#### Scenario: Implementation phase detected

- **WHEN** project has beads in ready phase
- **AND** no beads are in-progress
- **THEN** status command highlights "Ready to implement" with specific beads

#### Scenario: Research phase detected

- **WHEN** project has draft beads but no research
- **THEN** status command suggests adding research before moving to ready

### Requirement: Interactive Ambiguity Resolution

When a command is ambiguous, the CLI SHALL prompt for clarification instead of failing.

#### Scenario: Multiple matching beads

- **WHEN** user references a bead with an ambiguous identifier
- **AND** multiple beads match
- **THEN** the system shows a selection prompt with matching options
- **AND** user can choose or cancel

#### Scenario: Non-interactive mode

- **WHEN** user passes --no-interactive
- **AND** a command would normally prompt
- **THEN** the system returns an error with all matching options listed
