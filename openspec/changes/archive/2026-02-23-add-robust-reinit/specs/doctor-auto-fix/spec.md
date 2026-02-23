## ADDED Requirements

### Requirement: Version staleness check

The `wai doctor` command SHALL compare the workspace version (from `config.toml`) against the running binary version and report when they differ.

#### Scenario: Workspace version matches binary

- **WHEN** `config.toml` `project.version` equals `env!("CARGO_PKG_VERSION")`
- **THEN** check reports Pass with message "Version current"

#### Scenario: Workspace version is stale

- **WHEN** `config.toml` `project.version` differs from `env!("CARGO_PKG_VERSION")`
- **THEN** check reports Warn with message showing both versions
- **AND** fix suggestion says "Run: wai init (to update workspace)"

#### Scenario: Fix version staleness

- **WHEN** user runs `wai doctor --fix --yes` with stale version
- **THEN** system calls `ensure_workspace_current` to repair workspace
- **AND** config.toml version is updated to current binary version
- **AND** any missing directories or files are also created

## MODIFIED Requirements

### Requirement: Missing PARA directories are auto-fixable

The doctor --fix flow SHALL create missing directories including PARA directories, agent-config subdirectories, and resource subdirectories.

#### Scenario: Fix missing directories

- **GIVEN** `.wai/archives/` directory is missing
- **WHEN** user runs `wai doctor --fix --yes`
- **THEN** system creates `.wai/archives/` directory
- **AND** reports success for "Directory structure" fix

#### Scenario: Fix missing agent-config subdirectories

- **GIVEN** `.wai/resources/agent-config/skills/` directory is missing
- **WHEN** user runs `wai doctor --fix --yes`
- **THEN** system creates the missing agent-config subdirectory
- **AND** reports success for "Directory structure" fix

#### Scenario: Fix missing resource subdirectories

- **GIVEN** `.wai/resources/templates/` directory is missing
- **WHEN** user runs `wai doctor --fix --yes`
- **THEN** system creates the missing resource subdirectory
- **AND** reports success for "Directory structure" fix

#### Scenario: Fix missing default files

- **GIVEN** `.wai/.gitignore` is missing
- **WHEN** user runs `wai doctor --fix --yes`
- **THEN** system creates the default `.gitignore` file
- **AND** reports success for "Directory structure" fix
