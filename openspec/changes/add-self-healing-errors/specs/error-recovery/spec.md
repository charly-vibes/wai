# Spec: Resilient and Self-Healing CLI

## Purpose

This document specifies a suite of "self-healing" and resilient features designed to make the CLI more forgiving and intelligent. It aims to understand user intent even when commands are slightly incorrect.

*(Note: This document collects several related but distinct major features. For implementation, each requirement should likely be broken out into its own detailed design document.)*

## Problem Statement

Most command-line tools are brittle. They demand perfect syntax, word order, and execution context. A simple typo, a reversed command, or running a command from a subdirectory often results in a hard, unhelpful failure. This forces the user to carry a high cognitive load—remembering exact syntax and file locations—and leads to a frustrating, pedantic experience where the user is serving the tool, not the other way around.

## Design Rationale

The design philosophy is to create a resilient, forgiving CLI that understands user intent.

- **Anticipate and Correct Common Errors:** Instead of failing on simple mistakes, the tool should be smart enough to detect typos or reversed command structures and offer a correction. This turns a failure into a success.
- **Reduce User Burden:** The CLI should handle environmental context automatically. It should find the project root so the user doesn't have to think about it. It should present clear choices during complex operations like sync conflicts. The goal is to let the user focus on their work, not on appeasing the tool.
- **Conversational Tone:** Using friendly, helpful language in error messages reinforces the idea that the tool is an assistant, not an obstacle.

## Scope and Requirements

This spec covers the high-level design for several features that enhance CLI resilience.

### Non-Goals

- **Specific Algorithms:** This spec does not prescribe a specific string-similarity algorithm for typo detection.
- **The Sync Engine:** The "Sync Conflict Resolution" requirement defines how to handle conflicts, but the underlying sync mechanism is out of scope.
- **Artificial Intelligence:** All "intelligent" behavior is based on defined heuristics, not complex AI or NLP.

## Requirements

### Requirement: Typo Detection

The CLI SHALL detect typos in commands and suggest corrections.

#### Scenario: Unknown command with similar match

- **WHEN** user types an unknown command (e.g., `wai staus`)
- **AND** a similar valid command exists (e.g., `status`)
- **THEN** error says "Unknown command 'staus'"
- **AND** suggests "Did you mean 'wai status'?"

#### Scenario: Unknown subcommand with similar match

- **WHEN** user types an unknown subcommand (e.g., `wai new projet`)
- **AND** a similar valid subcommand exists (e.g., `project`)
- **THEN** error suggests the correct subcommand

### Requirement: Wrong Order Detection

The CLI SHALL detect reversed verb-noun patterns and suggest the correct order.

#### Scenario: Reversed command pattern

- **WHEN** user types `wai bead new "Title"`
- **THEN** error says "Unknown command 'bead'"
- **AND** suggests "Did you mean 'wai new bead \"Title\"'?"

### Requirement: Context Inference

The CLI SHALL infer project context from directory hierarchy.

#### Scenario: In project subdirectory

- **WHEN** user runs a project command from a subdirectory of a project
- **AND** `.para/` exists in a parent directory
- **THEN** the system automatically uses the project context from the parent directory
- **AND** informs the user which context is being used (e.g., "Running command in project '/path/to/project'").

### Requirement: Sync Conflict Resolution

When sync conflicts occur, the CLI SHALL offer resolution strategies.

#### Scenario: Conflicting changes

- **WHEN** a sync operation detects conflicting changes
- **THEN** error explains "Changes conflict with remote"
- **AND** offers options: "keep local", "keep remote", "merge manually"
- **AND** shows command for each option

### Requirement: Conversational Error Tone

Error messages SHALL use friendly, conversational language.

#### Scenario: Error phrasing

- **WHEN** any error occurs
- **THEN** the message uses phrases like "Let's fix this" or "Here's how to continue"
- **AND** avoids technical jargon where possible
