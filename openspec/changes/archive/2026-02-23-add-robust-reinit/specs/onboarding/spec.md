## ADDED Requirements

### Requirement: Comprehensive re-initialization

When `wai init` is run in an already-initialized workspace, it SHALL comprehensively repair the workspace to match the current version's expectations, not just update config and managed blocks.

#### Scenario: Re-init creates missing directories

- **WHEN** user runs `wai init` in an initialized workspace
- **AND** some expected directories are missing (e.g. `.wai/resources/agent-config/skills/`)
- **THEN** system creates all missing directories
- **AND** reports what was created

#### Scenario: Re-init creates missing default files

- **WHEN** user runs `wai init` in an initialized workspace
- **AND** `.wai/.gitignore` or `.wai/resources/agent-config/.projections.yml` is missing
- **THEN** system creates the missing default files
- **AND** does not overwrite existing files

#### Scenario: Re-init updates version

- **WHEN** user runs `wai init` in an initialized workspace
- **AND** config.toml version differs from binary version
- **THEN** system updates config.toml version to current binary version

#### Scenario: Re-init is idempotent

- **WHEN** user runs `wai init` twice in succession
- **THEN** the second run produces no errors
- **AND** the workspace state is identical after both runs
