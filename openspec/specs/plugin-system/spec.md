# Plugin System

## Purpose

Define the plugin architecture that allows extending wai with additional capabilities, including YAML-based configuration, hook points, command pass-through, and built-in plugins for common tools.

## Problem Statement

As the `wai` tool matures, the diversity of user needs for specialized functionality and integrations will grow. Without a robust extension mechanism, `wai` risks becoming either **bloated** from incorporating every niche feature request, or **insufficient** for users with specific workflow demands. A plugin system is a foundational **Type 1 architectural decision** needed to prevent core bloat, meet community-driven and user-specific extension needs, and keep the core tool lean, focused, and maintainable.

## Design Rationale

### YAML-Based Plugin Configuration

Plugins are configured via YAML files in `.wai/plugins/`, replacing the earlier TOML-based manifest approach. YAML provides better support for complex nested structures like hook definitions and command mappings. Each plugin declares:

- **Detector patterns**: How to detect if the plugin's tool is present (e.g., `.beads/` directory, `.git/` directory)
- **Commands**: CLI commands the plugin provides or passes through to external tools
- **Hooks**: Event handlers for project lifecycle events

### Hook Points

Plugins can respond to lifecycle events through defined hook points. This is a **Type 1 decision** establishing the integration pattern:

- `on_project_create` — called when a new project is created
- `on_phase_transition` — called when a project changes phase
- `on_handoff_generate` — called when generating a handoff document
- `on_status` — called when displaying project status

### Built-in Plugins

Three plugins ship with wai as built-ins:

- **beads** — detects `.beads/` directory, provides issue data for handoffs and status
- **openspec** — detects `openspec/` directory, provides spec change data
- **git** — detects `.git/` directory, provides commit history and status

### Command Pass-Through

Plugins can register commands that pass through to external CLIs. For example, `wai beads list` delegates to `bd list --json`. This provides a unified interface while leveraging existing tools.

## Scope and Requirements

This spec covers the plugin configuration format, lifecycle hooks, command routing, and built-in plugin definitions.

### Non-Goals

- Plugin registry or marketplace
- Plugin sandboxing or security model
- Plugin versioning or dependency resolution
- Remote plugin installation

### Requirement: Plugin Configuration

Plugins SHALL be configured via YAML files in `.wai/plugins/`.

#### Scenario: Plugin config format

- **WHEN** a plugin is defined
- **THEN** its configuration file follows this format:
  ```yaml
  name: beads
  description: Integration with beads issue tracker
  detector:
    type: directory
    path: .beads/
  commands:
    - name: list
      description: List beads issues
      passthrough: bd list --json
    - name: show
      description: Show beads issue details
      passthrough: bd show
  hooks:
    on_handoff_generate:
      command: bd list --status=open --json
      inject_as: open_issues
    on_status:
      command: bd stats --json
      inject_as: beads_status
  ```

### Requirement: Plugin Detection

Plugins SHALL auto-detect whether their associated tool is present.

#### Scenario: Auto-detect on init

- **WHEN** user runs `wai init`
- **THEN** the system checks each built-in plugin's detector pattern
- **AND** enables plugins whose tools are detected

#### Scenario: Detector types

- **WHEN** a plugin has a directory detector
- **THEN** the system checks for the specified directory relative to project root
- **WHEN** a plugin has a file detector
- **THEN** the system checks for the specified file

### Requirement: Plugin Hooks

Plugins SHALL respond to lifecycle events through hook points.

#### Scenario: Hook execution

- **WHEN** a lifecycle event occurs (e.g., project creation, phase transition)
- **THEN** the system calls all registered hooks for that event
- **AND** collects hook output for use by the triggering command

#### Scenario: Handoff enrichment

- **WHEN** generating a handoff
- **THEN** the system calls `on_handoff_generate` hooks on all enabled plugins
- **AND** injects the collected data into the handoff template

### Requirement: Command Pass-Through

Plugins SHALL support routing CLI commands to external tools.

#### Scenario: Plugin command execution

- **WHEN** user runs `wai <plugin-name> <command>`
- **THEN** the system looks up the command in the plugin's command list
- **AND** executes the passthrough command

### Requirement: Plugin Management

The CLI SHALL support listing, enabling, and disabling plugins.

#### Scenario: List plugins

- **WHEN** user runs `wai plugin list`
- **THEN** the system shows all known plugins with their status (enabled/disabled/not detected)

#### Scenario: Enable plugin

- **WHEN** user runs `wai plugin enable <name>`
- **THEN** the system enables the specified plugin

#### Scenario: Disable plugin

- **WHEN** user runs `wai plugin disable <name>`
- **THEN** the system disables the specified plugin without removing its configuration
