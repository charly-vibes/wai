# Plugin System

## Purpose

Define the plugin architecture that allows extending wai with additional capabilities, including installation, management, and discovery of plugins.

## Problem Statement

As the `wai` tool matures, the diversity of user needs for specialized functionality and integrations will grow. Without a robust extension mechanism, `wai` risks becoming either **bloated** from incorporating every niche feature request, or **insufficient** for users with specific workflow demands. A plugin system is a foundational **Type 1 architectural decision** needed to prevent core bloat, meet community-driven and user-specific extension needs, and keep the core tool lean, focused, and maintainable.

## Design Rationale

The plugin management system is designed to be simple and align with the existing `wai` CLI patterns, carefully balancing immediate user needs with preserving architectural optionality for the future.

- **CLI-Based Management:** Managing plugins via `add`, `show`, and `remove` commands establishes a **consistent user interface** for extensibility. This decision focuses on the user-facing experience, **preserving optionality** for the internal plugin architecture (how plugins are developed, executed, etc.) to be defined later.
- **Flexible Installation:** Supporting installation by `name` (implying a future registry) and by local `path` offers a practical balance. It provides easy discovery of public plugins while streamlining development of private or local ones, without locking into a single distribution model.

## Scope and Requirements

This document specifies the user-facing CLI for managing plugins. It does not cover the internal architecture of the plugin system itself.

### Non-Goals

- **Plugin Development API:** This spec explicitly defers defining how plugins are authored, what hooks they can use, or the API they must conform to. This **preserves flexibility** to design a robust API based on future needs and early feedback.
- **Plugin Execution & Security:** The runtime environment, sandboxing, and security model for plugins are complex topics. These are explicitly out of scope for this spec, **preserving optionality** for future architectural decisions in these critical areas.
- **Plugin Registry Implementation:** While the spec assumes a registry for name-based installation, the design and implementation of that service are not covered here, **allowing the ecosystem to evolve** before committing to a specific registry architecture.

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
