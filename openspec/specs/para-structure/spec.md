# PARA Structure

## Purpose

Define the PARA-based organizational structure for wai, where artifacts are categorized into Projects, Areas, Resources, and Archives within the `.wai/` directory.

## Problem Statement

Development workflows generate diverse artifacts — research notes, design documents, plans, handoffs, and configuration files. Without a consistent organizational framework, these artifacts become scattered and hard to find, leading to context loss between sessions and across team members. The PARA method (Projects, Areas, Resources, Archives) provides a proven categorization framework that maps naturally to how developers think about their work: active projects with defined outcomes, ongoing areas of responsibility, reference resources, and completed/inactive items.

## Design Rationale

### PARA Method

The PARA categorization (Projects, Areas, Resources, Archives) is a **Type 1 foundational decision** for wai's organizational model. It was selected because:

- **Projects** have defined outcomes and timelines, matching active development work
- **Areas** represent ongoing responsibilities without end dates (e.g., infrastructure, documentation standards)
- **Resources** hold reference material used across projects (agent configs, templates, patterns)
- **Archives** preserve completed or inactive items for future reference

This four-category model is simple enough to learn immediately yet expressive enough to handle real-world complexity.

### Per-Project Subdirectories

Each project contains standard subdirectories (`research/`, `plans/`, `designs/`, `handoffs/`) to provide consistent internal structure. This is a **Type 2 decision** — the specific subdirectories may evolve, but the pattern of structured subdirectories is foundational.

### Date-Prefixed Filenames

Artifacts use date-prefixed filenames (e.g., `2026-01-20-initial-research.md`) for chronological sorting. This convention enables timeline views and makes recency immediately visible in file listings.

## Scope and Requirements

This spec defines the directory structure and organizational rules for artifacts within `.wai/`.

### Non-Goals

- Content format of individual artifacts (covered by other specs)
- Full-text search across artifacts (covered by timeline-search spec)
- State machine for project phases (covered by project-state-machine spec)
- Plugin storage and configuration (covered by plugin-system spec)

### Requirement: PARA Directory Structure

The `.wai/` directory SHALL organize artifacts into four PARA categories.

#### Scenario: Initialize PARA structure

- **WHEN** user runs `wai init`
- **THEN** the system creates the following directory structure:
  ```
  .wai/
  ├── config.toml
  ├── projects/
  ├── areas/
  ├── resources/
  │   ├── agent-config/
  │   ├── templates/
  │   └── patterns/
  ├── archives/
  └── plugins/
  ```

### Requirement: Project Structure

Each project SHALL have a consistent internal directory structure.

#### Scenario: Create new project

- **WHEN** user runs `wai new project <name>`
- **THEN** the system creates:
  ```
  .wai/projects/<name>/
  ├── .state
  ├── research/
  ├── plans/
  ├── designs/
  └── handoffs/
  ```
- **AND** initializes `.state` with the default phase

#### Scenario: Create new area

- **WHEN** user runs `wai new area <name>`
- **THEN** the system creates:
  ```
  .wai/areas/<name>/
  ├── research/
  └── plans/
  ```

#### Scenario: Create new resource

- **WHEN** user runs `wai new resource <name>`
- **THEN** the system creates the directory `.wai/resources/<name>/`

### Requirement: Date-Prefixed Filenames

Artifacts SHALL use date-prefixed filenames for chronological sorting.

#### Scenario: Add artifact with date prefix

- **WHEN** user adds a research note, plan, design, or handoff
- **THEN** the filename follows the pattern `YYYY-MM-DD-<slug>.md`
- **AND** the artifact is sorted chronologically in directory listings

### Requirement: Archival

Items SHALL be movable to archives to keep active categories clean.

#### Scenario: Archive a project

- **WHEN** user runs `wai move <project> archives`
- **THEN** the system moves the project directory from `projects/` to `archives/`
- **AND** preserves all project contents and history

#### Scenario: Archive an area

- **WHEN** user runs `wai move <area> archives`
- **THEN** the system moves the area directory from `areas/` to `archives/`
