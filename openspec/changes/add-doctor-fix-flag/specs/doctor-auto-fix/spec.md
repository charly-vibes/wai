# Spec: Doctor Auto-Fix

## ADDED Requirements

### Requirement: Doctor command accepts --fix flag

The `wai doctor` command SHALL accept a `--fix` flag that automatically applies fixes for diagnosed issues where safe to do so.

#### Scenario: User runs doctor with --fix

```bash
$ wai doctor --fix
```

**GIVEN** workspace has fixable issues
**WHEN** user runs `wai doctor --fix`
**THEN** system prompts for confirmation
**AND** applies fixes for all confirmed issues
**AND** displays success/failure for each fix
**AND** exits with code 0 if all succeeded, 1 if any failed

---

### Requirement: Auto-fix respects global flags

The doctor --fix flow SHALL respect global flags for confirmation and safety.

#### Scenario: Skip confirmation with --yes

```bash
$ wai doctor --fix --yes
```

**GIVEN** workspace has fixable issues
**WHEN** user runs `wai doctor --fix --yes`
**THEN** system skips confirmation prompt
**AND** applies all fixes immediately

#### Scenario: JSON output with --fix

```bash
$ wai doctor --fix --json
```

**GIVEN** workspace has fixable issues
**WHEN** user runs `wai doctor --fix --json`
**THEN** system outputs structured JSON with fixes_applied and fixes_failed arrays
**AND** skips interactive confirmation

#### Scenario: Refuse in safe mode

```bash
$ wai doctor --fix --safe
```

**GIVEN** workspace has any state
**WHEN** user runs `wai doctor --fix --safe`
**THEN** system refuses with SafeModeViolation error
**AND** exits with non-zero code

---

### Requirement: Missing PARA directories are auto-fixable

The doctor --fix flow SHALL create missing PARA directories (projects, areas, resources, archives, plugins).

#### Scenario: Fix missing directories

**GIVEN** `.wai/archives/` directory is missing
**WHEN** user runs `wai doctor --fix --yes`
**THEN** system creates `.wai/archives/` directory
**AND** reports success for "Directory structure" fix

---

### Requirement: Stale projections are auto-fixable

The doctor --fix flow SHALL re-sync stale projections using the appropriate strategy (symlink, inline, or reference).

#### Scenario: Fix stale inline projection

**GIVEN** a projection with strategy "inline" is out of sync
**WHEN** user runs `wai doctor --fix --yes`
**THEN** system calls sync_core::execute_inline() for that projection
**AND** reports success for "Projection → <target>" fix

---

### Requirement: Missing project .state files are auto-fixable

The doctor --fix flow SHALL create default .state files for projects that are missing them (Warn status only, not corrupted files with Fail status).

#### Scenario: Fix missing .state file

**GIVEN** project "my-project" exists without `.state` file
**WHEN** user runs `wai doctor --fix --yes`
**THEN** system creates `.wai/projects/my-project/.state` with ProjectState::default()
**AND** reports success for "Project state: my-project" fix

#### Scenario: Skip corrupted .state file

**GIVEN** project has corrupted `.state` file (invalid YAML)
**WHEN** user runs `wai doctor --fix --yes`
**THEN** system does NOT overwrite the corrupted file
**AND** reports no fix available

---

### Requirement: Missing AGENTS.md managed block is auto-fixable

The doctor --fix flow SHALL inject the wai managed block into AGENTS.md if it exists but is missing the block.

#### Scenario: Fix missing managed block

**GIVEN** AGENTS.md exists without wai managed block
**WHEN** user runs `wai doctor --fix --yes`
**THEN** system calls inject_managed_block() with detected plugins
**AND** reports success for "Agent instructions" fix

---

### Requirement: Non-fixable issues are identified

The doctor --fix flow SHALL NOT attempt to fix issues that require manual intervention or external dependencies.

#### Scenario: Cannot fix missing plugin tools

**GIVEN** check reports missing CLI tool (e.g., `bd` not in PATH)
**WHEN** user runs `wai doctor --fix --yes`
**THEN** system does NOT attempt to install the tool
**AND** reports "No fixable issues" (if only non-fixable issues exist)

#### Scenario: Cannot fix corrupted config.toml

**GIVEN** config.toml is missing or invalid
**WHEN** user runs `wai doctor --fix --yes`
**THEN** system does NOT attempt to create/fix config
**AND** suggests running `wai init` instead
