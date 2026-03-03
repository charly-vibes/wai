## MODIFIED Requirements

### Requirement: Sync Command

The CLI SHALL provide a `wai sync` command to execute projections.

#### Scenario: Execute projections

- **WHEN** user runs `wai sync`
- **THEN** the system reads `.projections.yml`
- **AND** generates projected files according to each target's strategy
- **AND** reports which files were created or updated

#### Scenario: Check sync status

- **WHEN** user runs `wai sync --status`
- **THEN** the system shows which projections are up-to-date and which need syncing
- **AND** does not modify any files

#### Scenario: Dry-run preview

- **WHEN** user runs `wai sync --dry-run`
- **THEN** the system prints all operations that would be performed
- **AND** does not create, modify, or delete any files
- **AND** indicates the strategy and source/target paths for each projection

### Requirement: Projections Configuration

A `.projections.yml` file SHALL define how configs are projected to tool-specific locations.

#### Scenario: Projections file format

- **WHEN** configuring projections
- **THEN** `.projections.yml` follows this format:
  ```yaml
  projections:
    - target: .claude/
      strategy: symlink
      sources:
        - skills/
        - rules/
    - target: .cursorrules
      strategy: inline
      sources:
        - rules/
        - context/
    - target: AGENTS.md
      strategy: inline
      sources:
        - context/
    - target: .cursor/rules/wai.mdc
      strategy: copy
      sources:
        - rules/my-rule.md
  ```

#### Scenario: Copy strategy available

- **WHEN** symlinks are unavailable (e.g., Windows without developer mode)
- **THEN** `strategy: copy` copies each source file to the target location
- **AND** overwrites any existing file at the target path

## ADDED Requirements

### Requirement: Projection Parent Directory Creation

The sync system SHALL automatically create missing parent directories for all
projection targets before executing the projection.

#### Scenario: Target parent directory missing

- **WHEN** user runs `wai sync`
- **AND** the parent directory of a target path does not exist
- **THEN** the system creates all intermediate directories (equivalent to `mkdir -p`)
- **AND** proceeds to execute the projection without error

### Requirement: Symlink Projection Correctness

The symlink strategy SHALL create real filesystem symlinks, not empty directories,
and SHALL correctly handle both file and directory sources.

#### Scenario: File-to-file symlink

- **WHEN** a projection uses `strategy: symlink` with a source file and a target file path
- **THEN** the system creates a symlink at the target path pointing to the source file
- **AND** does NOT create a directory at the target path

#### Scenario: Directory source mirroring

- **WHEN** a projection uses `strategy: symlink` with a source directory
- **THEN** the system creates the target directory
- **AND** symlinks each file inside the source directory into the target directory
- **AND** does not recurse into subdirectories of the source

#### Scenario: Existing target replaced

- **WHEN** a target path already exists (file or directory)
- **THEN** the system removes it before creating the new symlink
