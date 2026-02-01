# Research Management

## Purpose

Define how research artifacts (notes, links, documents) are captured, organized, and associated with beads to support informed decision-making.

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
