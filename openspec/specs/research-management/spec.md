# Research Management

## Purpose

Define how research artifacts (notes, links, documents) are captured, organized, and stored within project and area subdirectories to support informed decision-making.

## Problem Statement

Valuable insights, notes, and external resources in development and research workflows are often scattered, making it challenging to consistently link supporting research directly to specific projects or areas of work. This leads to lost context, hindered decision-making, and duplicated effort. A **Type 1 strategic decision** is made here to address this by integrating research management directly within wai's PARA structure, ensuring research is organized alongside the work it supports.

## Design Rationale

- **Within PARA Structure:** Research lives inside project or area subdirectories (e.g., `.wai/projects/my-app/research/`), not in a top-level directory. This keeps research co-located with its context and naturally scoped.
- **Date-Prefixed Files:** Research entries use date-prefixed markdown filenames (e.g., `2026-01-20-api-analysis.md`) for chronological sorting and easy timeline integration.
- **Optional Association:** Research can optionally reference external tracking (e.g., a beads issue ID via frontmatter) without requiring it. This keeps research useful even without external tools.
- **File-based Storage:** Storing research as transparent, accessible files ensures **data portability and resilience**. This avoids the overhead of a dedicated database and keeps user data accessible outside `wai`.

## Scope and Requirements

This spec focuses on the core functionality for capturing and organizing research within the PARA structure.

### Non-Goals

- Editing and deleting existing research (use standard file operations)
- Full-text search (covered by timeline-search spec)
- Rich media or complex formatting
- Integration with external research management software
- Version control for individual entries (use git)

## Requirements

### Requirement: Adding Research

The CLI SHALL support capturing research content within a project or area.

#### Scenario: Add research to current project

- **WHEN** user runs `wai add research <content>`
- **THEN** the system creates a date-prefixed research file in the current project's `research/` directory
- **AND** stores the content as markdown

#### Scenario: Add research to specific project

- **WHEN** user runs `wai add research <content> --project <name>`
- **THEN** the system creates the research entry in the specified project's `research/` directory

#### Scenario: Add research from file

- **WHEN** user runs `wai add research --file <path>`
- **THEN** the system copies the file content as a research entry with date prefix

#### Scenario: Research with frontmatter

- **WHEN** user runs `wai add research <content> --tags <tag1,tag2>`
- **THEN** the research file includes YAML frontmatter with tags
- **AND** optionally includes a `bead` field for external tracking association
