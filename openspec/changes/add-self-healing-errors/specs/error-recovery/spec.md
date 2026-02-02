# Spec: Resilient and Self-Healing CLI

## Purpose

This document specifies a suite of "self-healing" and resilient features designed to make the CLI more forgiving and intelligent. It aims to understand user intent even when commands are slightly incorrect.

*(Note: This document collects several related but distinct major features. For implementation, each requirement should likely be broken out into its own detailed design document.)*

## Problem Statement

Most command-line tools are brittle, demanding perfect syntax, word order, and execution context. Simple mistakes like typos or reversed commands often result in hard, unhelpful failures. This forces the user to carry a high cognitive load, remembering exact syntax and file locations, leading to a frustrating, pedantic experience. This spec represents a **Type 1 commitment** to transforming `wai` into a resilient, forgiving CLI that anticipates and helps correct common user errors, empowering the user rather than frustrating them.

## Design Rationale

The design philosophy is to create a resilient, forgiving CLI that understands user intent. The features outlined here represent **Type 1 decisions** for building a truly intelligent and user-centric command-line experience.

- **Anticipate and Correct Common Errors:** This is a **Type 1 decision** to transform user errors into learning opportunities. Instead of failing on simple mistakes, `wai` will proactively detect typos or reversed command structures and offer corrections, turning a potential failure into a quick success.
- **Reduce User Burden:** The CLI should handle environmental context automatically. This is a **Type 1 decision** to reduce the cognitive load on the user. `wai` will infer the project root and present clear choices during complex operations like sync conflicts, allowing the user to focus on their primary task.
- **Conversational Tone:** Using friendly, helpful language in error messages reinforces the idea that the tool is an assistant, not an obstacle, enhancing the overall user experience.

## Scope and Requirements

This spec covers the high-level design for several features that enhance CLI resilience.

### Non-Goals

- **Specific Algorithms:** This spec defines the desired *behavior* for features like typo detection but does not prescribe a specific string-similarity algorithm or its implementation details.
- **The Sync Engine:** The "Sync Conflict Resolution" requirement defines how to handle conflicts, but the underlying sync mechanism and its implementation are explicitly out of scope for this high-level design.
- **Artificial Intelligence:** All "intelligent" behavior in this spec is based on defined heuristics and pattern matching, not complex AI or Natural Language Processing (NLP), allowing for a more predictable and auditable system.

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
- **AND** `.wai/` exists in a parent directory
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
