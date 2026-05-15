# Spec delta: cli-core

## ADDED Requirements

### Requirement: wai artifacts subcommand group

`wai artifacts` SHALL be a new top-level subcommand group for artifact-level
inspection and management. Initial subcommand: `stale`.

#### Scenario: Help text is discoverable

- **GIVEN** a user runs `wai artifacts --help`
- **THEN** output lists the `stale` subcommand with a one-line description

### Requirement: wai artifacts stale command

`wai artifacts stale [--json]` SHALL scan all tracked decision artifacts and
report which have stale or untracked freshness state. Exit code MUST always
be 0 (stale is advisory). The command SHALL delegate to the
`decision-freshness` scanner.

#### Scenario: Command is reachable

- **GIVEN** any workspace
- **WHEN** `wai artifacts stale` runs
- **THEN** exit code is 0 and output is produced

## MODIFIED Requirements

### Requirement: wai add accepts --tracks flag

`wai add research|design|plan` SHALL accept an optional
`--tracks <path>[,<path>...]` flag. When provided, the paths MUST be written as
the `tracks` YAML list in the artifact frontmatter and a freshness sidecar MUST
be written immediately using the current mtime and SHA-256 of each path.

#### Scenario: tracks flag writes frontmatter and sidecar

- **GIVEN** `wai add research --tracks src/commands/status.rs,src/commands/doctor.rs`
- **WHEN** the command completes
- **THEN** the created artifact contains `tracks:` with both paths in frontmatter
- **AND** a `.fresh.lock` sidecar exists adjacent to the artifact
- **AND** the sidecar records current mtime and SHA-256 for each path
