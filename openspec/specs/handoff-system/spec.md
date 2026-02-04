# Handoff System

## Purpose

Define first-class handoff artifacts that capture session context, decisions, and next steps for seamless transitions between work sessions or between human and AI collaborators.

## Problem Statement

Context loss between work sessions is one of the biggest productivity drains in software development. When a developer (or AI agent) picks up work, they spend significant time reconstructing what was done, what decisions were made, and what comes next. Handoff documents formalize this context transfer, but without tooling support they are inconsistently created and often incomplete. A built-in handoff system ensures that context is captured systematically and enriched with data from other tools.

## Design Rationale

### First-Class Artifact

Making handoffs a first-class artifact type rather than ad-hoc notes is a **Type 1 decision**. This ensures they have consistent structure, are stored in predictable locations, and can be enriched by plugins (e.g., beads issues, git status, openspec changes).

### Template-Based Generation

Handoffs are generated from templates with frontmatter metadata. This is a **Type 2 decision** — templates can evolve, but the pattern of structured generation with metadata is foundational.

### Plugin Enrichment

The handoff generation process calls plugin hooks to gather contextual data. This means a handoff can automatically include:
- Open beads issues and their status
- Recent git commits and uncommitted changes
- OpenSpec changes in progress

## Scope and Requirements

This spec covers the handoff artifact format, generation command, and plugin integration points.

### Non-Goals

- Automatic handoff generation (handoffs are explicitly created by the user)
- Handoff consumption or parsing by other tools
- Multi-format output (PDF, HTML, etc.)
- Collaborative editing of handoffs

### Requirement: Handoff Format

Handoffs SHALL be markdown files with structured frontmatter.

#### Scenario: Handoff file structure

- **WHEN** a handoff is generated
- **THEN** it follows this format:
  ```markdown
  ---
  date: 2026-01-22
  project: my-project
  phase: implement
  agent: claude
  ---

  # Session Handoff

  ## What Was Done
  <!-- Summary of completed work -->

  ## Key Decisions
  <!-- Decisions made and rationale -->

  ## Open Questions
  <!-- Unresolved questions -->

  ## Next Steps
  <!-- Prioritized list of what to do next -->

  ## Context
  <!-- Plugin-enriched context (beads, git, etc.) -->
  ```

### Requirement: Handoff Generation

The CLI SHALL provide a command to generate handoff documents.

#### Scenario: Create handoff

- **WHEN** user runs `wai handoff create <project>`
- **THEN** the system generates a handoff from the default template
- **AND** enriches it with data from enabled plugins
- **AND** stores it in `.wai/projects/<project>/handoffs/YYYY-MM-DD-session-end.md`

#### Scenario: Handoff with plugin data

- **WHEN** generating a handoff with the beads plugin enabled
- **THEN** the Context section includes open beads issues and their status
- **WHEN** generating a handoff with the git plugin enabled
- **THEN** the Context section includes recent commits and uncommitted changes

### Requirement: Handoff Storage

Handoffs SHALL be stored in the project's handoffs directory with date prefixes.

#### Scenario: Handoff location

- **WHEN** a handoff is created for project "my-app"
- **THEN** it is stored at `.wai/projects/my-app/handoffs/YYYY-MM-DD-session-end.md`
- **AND** multiple handoffs per day are differentiated by suffix
