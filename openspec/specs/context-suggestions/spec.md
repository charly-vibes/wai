# Context Suggestions

## Purpose

Define context-aware suggestion patterns that help users discover the next logical step based on current project state and phase.

## Problem Statement

A status command that only presents data often leaves the user with the question, "What should I do next?" Without proactive guidance, users experience decision fatigue, make suboptimal choices about where to focus effort, and lose workflow momentum. This spec proposes a **Type 1 commitment** to integrating a proactive suggestion system within the `wai status` command, bridging the gap between information and action, and making the user more effective.

## Design Rationale

The suggestion system is designed to be proactive, integrated, and based on project phase awareness, representing **Type 1 decisions** that establish a core UX pattern.

- **Integrated with `status`:** Placing suggestions directly within the `wai status` output is a deliberate **Type 1 decision**. It establishes a proactive UX pattern where `wai` guides the user immediately after reviewing the project's state, reducing cognitive load and driving workflow.
- **Phase-Based Priority:** Suggestions are driven by the current project phase (research, plan, design, implement, review, archive) and available plugin data. Each phase has natural next actions that wai can suggest contextually.

## Scope, Dependencies, and Requirements

This spec defines the logic for generating suggestions within the `wai status` command.

### Dependencies

- **Project State Machine:** The suggestion logic depends on the phase definitions outlined in the `project-state-machine` spec.
- **Plugin System:** Plugins (beads, git, openspec) provide additional context for richer suggestions.

### Non-Goals

- Implementation of suggested actions — suggestions are informational only
- Suggestions in commands other than `wai status`
- Exact UI/formatting — this spec defines logic, not presentation

## Requirements

### Requirement: Phase-Based Suggestions

The status command SHALL provide contextual next-step suggestions based on the current project phase.

#### Scenario: Research phase

- **WHEN** project is in "research" phase
- **THEN** suggest adding research notes: `wai add research "..."`
- **AND** suggest advancing when ready: `wai phase next`

#### Scenario: Plan phase

- **WHEN** project is in "plan" phase
- **THEN** suggest adding a plan: `wai add plan "..."`
- **AND** suggest advancing to design: `wai phase next`

#### Scenario: Design phase

- **WHEN** project is in "design" phase
- **THEN** suggest adding designs: `wai add design "..."`
- **AND** suggest advancing to implementation: `wai phase next`

#### Scenario: Implement phase

- **WHEN** project is in "implement" phase
- **THEN** suggest creating a handoff when pausing: `wai handoff create`
- **AND** suggest advancing to review: `wai phase next`

#### Scenario: Review phase

- **WHEN** project is in "review" phase
- **THEN** suggest completing and archiving: `wai phase next`
- **AND** suggest going back if issues found: `wai phase back`

#### Scenario: Archive phase

- **WHEN** project is in "archive" phase
- **THEN** suggest starting a new project: `wai new project`

### Requirement: Plugin-Enhanced Suggestions

When plugins are active, suggestions SHALL incorporate plugin context.

#### Scenario: Beads plugin active

- **WHEN** beads plugin reports open issues
- **THEN** status includes issue count and suggests reviewing: `wai beads list`

#### Scenario: Git plugin active

- **WHEN** git plugin reports uncommitted changes
- **THEN** status includes change summary

### Requirement: Empty Project Suggestions

When a project has no artifacts, suggestions SHALL guide initial setup.

#### Scenario: New empty project

- **WHEN** project has no research, plans, or designs
- **THEN** suggest starting with research: `wai add research "..."`
- **AND** suggest checking phase: `wai phase`
