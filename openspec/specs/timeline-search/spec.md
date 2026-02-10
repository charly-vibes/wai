# Timeline & Search

## Purpose

Define cross-artifact search and chronological timeline views that help users find and navigate artifacts across the entire `.wai/` structure.

## Problem Statement

As a project accumulates research notes, plans, designs, and handoffs across multiple projects and areas, finding specific information becomes increasingly difficult. Users need two complementary navigation modes: chronological (what happened when?) and content-based (where is this mentioned?). Without built-in search and timeline capabilities, users fall back to manual file browsing or external grep commands that lack context about wai's organizational structure.

## Design Rationale

### Dual Navigation Modes

Providing both timeline and search views is a **Type 2 decision** based on two fundamental ways developers navigate information:

- **Timeline** answers "what happened?" — useful for reconstructing context, reviewing progress, and generating reports
- **Search** answers "where is this?" — useful for finding specific decisions, references, or artifacts

### Project-Scoped and Global

Both timeline and search support project-scoped queries (most common) and global queries (cross-project discovery). This dual scope matches how developers think: usually focused on one project, occasionally needing to find something across all work.

## Scope and Requirements

This spec covers the timeline and search commands and their filtering capabilities.

### Non-Goals

- Full-text indexing or database-backed search
- Search ranking or relevance scoring
- Real-time search updates or watch mode
- Integration with external search tools

### Requirement: Timeline View

The CLI SHALL provide a chronological view of artifacts within a project.

#### Scenario: Project timeline

- **WHEN** user runs `wai timeline <project>`
- **THEN** the system scans all dated artifacts in the project directory
- **AND** displays them chronologically with date, type, and title

#### Scenario: Timeline output format

- **WHEN** displaying a timeline
- **THEN** each entry shows:
  - Date (from filename prefix)
  - Artifact type (research, plan, design, handoff)
  - Title or filename
- **AND** entries are sorted newest-first by default

#### Scenario: Timeline output as JSON

- **WHEN** user runs `wai timeline <project> --json`
- **THEN** the system outputs JSON entries with `date`, `type`, `title`, and `path`

### Requirement: Search

The CLI SHALL provide full-text search across all artifacts.

#### Scenario: Global search

- **WHEN** user runs `wai search <query>`
- **THEN** the system searches across all `.wai/` artifacts
- **AND** displays matching files with context lines

#### Scenario: Type-filtered search

- **WHEN** user runs `wai search <query> --type research`
- **THEN** the system searches only within research artifacts

#### Scenario: Project-scoped search

- **WHEN** user runs `wai search <query> --in <project>`
- **THEN** the system searches only within the specified project's artifacts

#### Scenario: Search output format

- **WHEN** displaying search results
- **THEN** each result shows:
  - File path (relative to `.wai/`)
  - Matching line with query highlighted
  - Context (lines before and after the match)

#### Scenario: Search output as JSON

- **WHEN** user runs `wai search <query> --json`
- **THEN** the system outputs JSON results with `path`, `line_number`, `line`, and `context`
