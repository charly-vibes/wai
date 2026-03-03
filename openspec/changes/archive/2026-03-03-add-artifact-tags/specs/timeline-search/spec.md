## MODIFIED Requirements

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

#### Scenario: Tag-filtered search

- **WHEN** user runs `wai search <query> --tag <value>`
- **THEN** the system only returns artifacts whose YAML frontmatter contains a tag
  matching the given value
- **AND** artifacts without frontmatter or without the matching tag are excluded

#### Scenario: Malformed frontmatter treated as no tags

- **WHEN** an artifact file has absent, empty, or unparseable YAML frontmatter
- **AND** user runs a search with `--tag <value>`
- **THEN** the artifact is silently excluded from results (treated as having no tags)
- **AND** the search does not error or abort

#### Scenario: Latest match only

- **WHEN** user runs `wai search <query> --latest`
- **THEN** the system returns only the single most recently dated artifact among all matches
- **AND** "most recently dated" is determined by the date prefix in the filename
  (e.g., `2026-02-25-...` is more recent than `2026-01-10-...`)

#### Scenario: Combined tag, type, and latest filtering

- **WHEN** user runs `wai search "ant-forager" --tag topic:ant-forager --type plan --latest`
- **THEN** the system returns at most one result: the most recent plan artifact
  tagged with `topic:ant-forager` whose content matches "ant-forager"
