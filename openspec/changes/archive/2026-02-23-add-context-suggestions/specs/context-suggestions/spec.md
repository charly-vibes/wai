# Spec: Advanced Contextual Suggestions & Interaction

## Purpose

Define a suite of advanced features to make the CLI more interactive and assistive, including post-command suggestions, workflow pattern detection, and interactive ambiguity resolution.

## Problem Statement

A command-line interface can often feel like a series of disconnected, atomic operations. Users are left to figure out the next logical step on their own, interrupting their flow. When commands are ambiguous, the tool typically fails, forcing a frustrating cycle of listing items and re-running the command. This creates a rigid, unforgiving user experience. This spec represents a **Type 1 commitment** to evolving `wai` from a passive instruction parser into an active, helpful, and interactive assistant.

## Design Rationale

This proposal aims to evolve `wai` from a simple instruction parser into a more helpful, interactive assistant. The features outlined here represent **Type 1 decisions** that establish core UX patterns for proactive guidance and error handling.

- **Post-Command Suggestions:** This is a **Type 1 decision** for guiding the user through a natural workflow. By suggesting next steps after a successful command, the CLI reduces cognitive load and improves feature discoverability, making the tool feel more like a conversational partner.
- **Interactive Ambiguity Resolution:** This is a **Type 1 decision** to transform moments of failure into efficient, successful interactions. Instead of failing, the tool helps the user succeed by prompting with choices when ambiguity is detected. The `--no-interactive` flag provides an essential escape hatch for scripting, preserving automation capabilities.
- **Workflow Pattern Detection:** This is a **Type 1 decision** to move towards a more intelligent agent. By recognizing the user's context at a higher level (e.g., "it looks like you're in a research phase"), `wai` can provide more tailored, insightful guidance, enhancing user effectiveness.

## Scope and Requirements

This spec covers the design for several advanced interactive features.

### Non-Goals

- **Suggestions for all commands:** Given the broad scope, the initial implementation will focus on a few high-value commands, representing a **phased approach** rather than an immediate, exhaustive implementation across the entire command suite.
- **Detecting all workflow anti-patterns:** The system will initially detect a small, well-defined set of common workflow patterns, incrementally expanding based on user need and feasibility.
- **Resolving all types of ambiguity:** The initial focus for interactive resolution is on project selection; expanding to other forms of ambiguity will be a future consideration.
- **Natural Language Processing (NLP):** All detection and suggestion logic will be based on defined heuristics, not complex NLP, keeping the implementation manageable and predictable.

## Requirements

### Requirement: Post-Command Suggestions

After each command, the CLI SHALL suggest logical next steps based on what just happened.

#### Scenario: After creating project

- **WHEN** user successfully runs `wai init` or `wai new project`
- **THEN** output suggests: add research, check phase, check status

#### Scenario: After adding research

- **WHEN** user successfully adds research
- **THEN** output suggests: add plan, review research, check phase

#### Scenario: After advancing to implement phase

- **WHEN** user advances to the implement phase
- **THEN** output suggests: show project details, add notes, check status

### Requirement: Workflow Pattern Detection

The CLI SHALL detect common workflow patterns and tailor suggestions.

#### Scenario: Implementation phase detected

- **WHEN** project is in plan or design phase
- **AND** has designs ready
- **THEN** status command highlights "Ready to implement" with next steps

#### Scenario: Research phase detected

- **WHEN** project is in research phase with minimal research
- **THEN** status command suggests adding research before advancing phases

### Requirement: Interactive Ambiguity Resolution

When a command is ambiguous, the CLI SHALL prompt for clarification instead of failing.

#### Scenario: Multiple matching projects

- **WHEN** user references a project with an ambiguous identifier
- **AND** multiple projects match
- **THEN** the system shows a selection prompt with matching options
- **AND** user can choose or cancel

#### Scenario: Non-interactive mode

- **WHEN** user passes --no-interactive
- **AND** a command would normally prompt
- **THEN** the system returns an error with all matching options listed
