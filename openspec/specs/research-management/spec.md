# Research Management

## Purpose

Define how research artifacts (notes, links, documents) are captured, organized, and associated with beads to support informed decision-making.

## Problem Statement

Valuable insights, notes, and external resources in development and research workflows are often scattered, making it challenging to consistently link supporting research directly to specific work items or "beads." This leads to lost context, hindered decision-making, and duplicated effort. A **Type 1 strategic decision** is made here to address this by tightly integrating a centralized, easily accessible research management system directly within `wai`, ensuring research directly supports ongoing work.

## Design Rationale

The design for research management within `wai` prioritizes simplicity, direct association with work items, and seamless integration with the CLI workflow, while carefully considering long-term data portability and optionality.

- **Integrated with CLI:** Managing research through `wai` commands provides a consistent experience and allows direct linking of research to `beads` within the project's context, making research an active part of the workflow.
- **File-based Storage:** Storing research as transparent, accessible files in the project structure (e.g., Markdown) is a **key optionality-preserving choice**. It simplifies data management, allows users to leverage existing file system tools and version control, and importantly, ensures **data portability and resilience**. This avoids the overhead of a dedicated database for initial use cases while keeping user data accessible outside `wai`.

## Scope and Requirements

This spec focuses on the core functionality for capturing, associating, and retrieving research within the `wai` project structure.

### Non-Goals

- **Editing and Deleting Research:** Initial focus is on capturing and retrieving; modification and removal of existing research entries are deferred for future iterations.
- **Advanced Search:** Beyond basic listing and tag-based filtering, full-text search capabilities are not in scope.
- **Rich Media & Complex Formatting:** Support for complex rich-text editing or embedded media within research content is not covered.
- **Integration with External Tools:** This spec focuses on `wai`'s native capabilities rather than direct integration with third-party research management software.
- **Version Control for Research Entries:** While the underlying files (due to file-based storage) are naturally amenable to user-managed source control (e.g., Git), explicit versioning features *within* `wai` for individual research entries are out of scope.

## Requirements

### Requirement: Adding Research

The CLI SHALL support capturing research content associated with a project or bead.

#### Scenario: Add research note

- **WHEN** user runs `wai add research <content>`
- **THEN** the system creates a research entry with the provided content
- **AND** stores it in the project's research directory

#### Scenario: Add research with bead association

- **WHEN** user runs `wai add research <content> --bead <id>`
- **THEN** the system creates a research entry linked to the specified bead
- **AND** the research is visible when viewing that bead

#### Scenario: Add research from file

- **WHEN** user runs `wai add research --file <path>`
- **THEN** the system imports the file content as a research entry

### Requirement: Viewing Research

The CLI SHALL support viewing and searching research entries.

#### Scenario: List all research

- **WHEN** user runs `wai show research`
- **THEN** the system lists all research entries in the project
- **AND** shows title/summary, creation date, and associated bead (if any)

#### Scenario: Filter research by bead

- **WHEN** user runs `wai show research --bead <id>`
- **THEN** the system shows only research entries linked to that bead

### Requirement: Research Organization

The CLI SHALL support organizing research with tags and categories.

#### Scenario: Tag research

- **WHEN** user runs `wai add research <content> --tags <tag1,tag2>`
- **THEN** the research entry is tagged with the specified labels
- **AND** can be filtered by those tags later
