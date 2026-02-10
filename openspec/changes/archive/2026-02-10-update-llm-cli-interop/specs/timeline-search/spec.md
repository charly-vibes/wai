## MODIFIED Requirements

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
