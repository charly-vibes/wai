## ADDED Requirements

### Requirement: Plugin Hook Trust Model

Repository plugin hooks MUST be trusted before execution. Trust is content-addressed:
the SHA-256 digest of the plugin manifest and hook definition is stored in a user-owned
XDG state directory outside the repository. Any content change invalidates approval.
Unapproved hooks MUST be skipped with a warning.

#### Scenario: Unapproved hook is skipped
- **WHEN** a lifecycle event triggers hook execution
- **AND** the hook's digest is not in the trust store
- **THEN** the hook is skipped without executing
- **AND** a warning is emitted (human mode) or a structured warning is output (machine mode)

#### Scenario: Approved hook is executed
- **WHEN** a lifecycle event triggers hook execution
- **AND** the hook's digest is in the trust store
- **THEN** the hook is executed normally

#### Scenario: Content modification invalidates trust
- **WHEN** a previously approved hook's manifest or command definition changes
- **THEN** its digest changes
- **AND** the hook is treated as unapproved until re-approved

#### Scenario: Built-in plugins are always trusted
- **WHEN** a hook from a built-in plugin is executed
- **THEN** it is always trusted without requiring explicit approval

#### Scenario: Non-interactive mode is fail-closed
- **WHEN** `--no-input` is active
- **AND** a hook is unapproved
- **THEN** the hook is skipped without prompting
- **AND** a structured warning is output

#### Scenario: Safe mode is fail-closed
- **WHEN** `--safe` is active
- **AND** a hook is unapproved
- **THEN** the hook is skipped without prompting

### Requirement: Plugin Trust Management

The CLI SHALL support approving, listing, and revoking plugin hook trust.

#### Scenario: Approve a plugin hook
- **WHEN** user runs `wai plugin trust <name> [--hook <hook-name>]`
- **THEN** the hook's digest is computed and stored in the trust store
- **AND** the hook becomes approved for future execution

#### Scenario: List approved digests
- **WHEN** user runs `wai plugin trust --list`
- **THEN** all approved digests are displayed with their associated plugin name, hook name, and command

#### Scenario: Revoke approval
- **WHEN** user runs `wai plugin trust --revoke <digest>`
- **THEN** the digest is removed from the trust store
- **AND** the associated hook is no longer approved

## MODIFIED Requirements

### Requirement: Plugin Configuration

Plugins SHALL be configured via TOML files in `.wai/plugins/`.

#### Scenario: Plugin config format
- **WHEN** a plugin is defined
- **THEN** its configuration file follows this format:
  ```toml
  name = "beads"
  description = "Integration with beads issue tracker"
  [detector]
  type = "directory"
  path = ".beads"
  [commands.list]
  name = "list"
  description = "List beads issues"
  passthrough = "bd list --json"
  read_only = true
  [hooks.on_handoff_generate]
  command = "bd list --status=open --json"
  inject_as = "open_issues"
  [hooks.on_status]
  command = "bd stats --json"
  inject_as = "beads_status"
  ```

### Requirement: Plugin Hooks

Plugins SHALL respond to lifecycle events through hook points. Hook execution
MUST be subject to the trust model.

#### Scenario: Hook execution with trust gate
- **WHEN** a lifecycle event occurs (e.g., project creation, phase transition)
- **THEN** the system checks each hook's trust status
- **AND** executes only approved hooks
- **AND** collects output from approved hooks
- **AND** reports skipped hooks as warnings

#### Scenario: Malicious-repository protection
- **WHEN** a repository contains a `.wai/plugins/` directory with hook definitions
- **AND** those hooks are not in the trust store
- **THEN** `wai status`, `wai prime`, `wai new`, `wai phase`, and `wai handoff` do NOT execute them
- **AND** each skipped hook is reported as a warning