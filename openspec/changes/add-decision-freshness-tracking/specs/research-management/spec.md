# Spec delta: research-management

## MODIFIED Requirements

### Requirement: Decision artifacts support tracks frontmatter

Research, design, and plan artifacts SHALL support two optional frontmatter
fields:

- `tracks`: YAML list of repo-relative file paths or glob patterns that this
  artifact describes. Absence means the artifact opts out of freshness tracking.
- `decision_point`: optional slug grouping related artifacts under one concern.
  Reserved for future rollup features; ignored in v1 processing.

#### Scenario: tracks field is preserved on re-read

- **GIVEN** an artifact with `tracks: [src/foo.rs]` in frontmatter
- **WHEN** wai reads the artifact for any purpose
- **THEN** the `tracks` list is available to the freshness scanner without modification

#### Scenario: Missing tracks field is treated as empty

- **GIVEN** an artifact with no `tracks` key in frontmatter
- **WHEN** the freshness scanner inspects the artifact
- **THEN** it treats the tracked-path set as empty and skips the artifact
