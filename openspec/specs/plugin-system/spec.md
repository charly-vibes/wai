# Plugin System

## Purpose

Define the plugin architecture that allows extending wai with additional capabilities, including installation, management, and discovery of plugins.

## Problem Statement

As the `wai` tool matures, users will require specialized functionality and integrations that are not part of the core vision. Adding every feature request to the core application would lead to bloat and increased maintenance complexity. A plugin system is needed to allow for community-driven and user-specific extensions while keeping the core tool lean and focused.

## Design Rationale

The plugin management system is designed to be simple and align with the existing `wai` CLI patterns.

- **CLI-Based Management:** Managing plugins via `add`, `show`, and `remove` commands provides a consistent user experience for developers already familiar with `wai` and other command-line tools.
- **Flexible Installation:** Supporting installation by `name` (implying a future registry) and by local `path` offers a balance between easy discovery of public plugins and streamlined development of private or local ones.

## Scope and Requirements

This document specifies the user-facing CLI for managing plugins. It does not cover the internal architecture of the plugin system itself.

### Non-Goals

- **Plugin Development API:** This spec does not define how plugins are authored, what hooks they can use, or the API they must conform to.
- **Plugin Execution & Security:** The runtime environment, sandboxing, and security model for plugins are complex topics and are explicitly out of scope.
- **Plugin Registry Implementation:** While the spec assumes a registry for name-based installation, the design and implementation of that service are not covered here.

## Requirements

### Requirement: Plugin Installation

The CLI SHALL support adding plugins to extend functionality.

#### Scenario: Install plugin by name

- **WHEN** user runs `wai add plugin <name>`
- **THEN** the system downloads and installs the plugin
- **AND** the plugin is available for use in the current project

#### Scenario: Install plugin from path

- **WHEN** user runs `wai add plugin --path <local-path>`
- **THEN** the system installs the plugin from the local directory
- **AND** the plugin is linked to the project

### Requirement: Plugin Listing

The CLI SHALL support viewing installed and available plugins.

#### Scenario: Show installed plugins

- **WHEN** user runs `wai show plugins`
- **THEN** the system lists all plugins installed in the current project
- **AND** shows plugin name, version, and status

#### Scenario: Show available plugins

- **WHEN** user runs `wai show plugins --available`
- **THEN** the system lists plugins available for installation
- **AND** shows plugin name and brief description

### Requirement: Plugin Removal

The CLI SHALL support removing plugins from a project.

#### Scenario: Remove installed plugin

- **WHEN** user runs `wai remove plugin <name>`
- **THEN** the system removes the plugin from the project
- **AND** confirms the removal to the user
