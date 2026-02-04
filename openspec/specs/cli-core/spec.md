# CLI Core

## Purpose

Define the core command structure and patterns for the wai CLI, including the verb-noun command hierarchy, global flags, and foundational commands for managing PARA-organized artifacts, project phases, agent config sync, handoffs, and cross-artifact search.

See also: onboarding spec for first-run and no-args welcome behavior.

## Problem Statement

For `wai` to effectively support development and research projects, it requires a **stable and predictable foundation**. Without a clearly defined and consistent core command structure and project organization, users would face a steep learning curve, inconsistent interactions, and an unstable platform for automation. This lack of a standardized and predictable interface would hinder adoption and make it difficult to build reliable workflows around `wai`.

## Design Rationale

The design of the CLI core follows a few key principles to establish a **stable foundation** that is intuitive, consistent, and extensible, making it a reliable platform for future growth.

### Command Structure: Verb-Noun

The chosen `verb-noun` pattern (e.g., `wai new project`) is a foundational **Type 1 decision** for `wai`'s grammar. It was selected for its readability and similarity to natural language, establishing a **predictable and consistent rhythm** for the user. This stable grammar makes commands easy to discover and remember, and crucially, **enables future extensibility** by providing a clear framework for applying existing verbs to new nouns. An alternative `noun-verb` pattern (e.g., `wai project new`) was considered but deemed less intuitive for `wai`'s action-oriented approach.

### Core Verbs

The primary verbs (`new`, `add`, `show`, `move`) provide a minimal, orthogonal set of operations. Additional top-level commands (`phase`, `sync`, `config`, `handoff`, `search`, `timeline`) provide direct access to frequently-used workflows that don't fit the verb-noun pattern naturally.

### PARA-Based Organization

Wai organizes artifacts using the PARA method (Projects, Areas, Resources, Archives). This replaces the previous bead-centric model with a proven organizational framework. Beads (`.beads/`) is an external tool that wai detects via its plugin system but does not manage directly.

## Scope and Requirements

This spec covers the foundational elements of the CLI.

### Non-Goals

- The detailed implementation of every command's functionality.
- The internal plugin execution model (covered in plugin-system spec).
- Specific output formats like JSON or YAML, beyond the standard text output.
- A graphical user interface.

### Requirement: Command Structure

The CLI SHALL use consistent verb-noun command patterns with primary verbs: `new`, `add`, `show`, `move`, plus dedicated top-level commands for `phase`, `sync`, `config`, `handoff`, `search`, and `timeline`.

#### Scenario: Create new items

- **WHEN** user runs `wai new project <name>`, `wai new area <name>`, or `wai new resource <name>`
- **THEN** the system creates the requested PARA item with appropriate directory structure

#### Scenario: Add artifacts to a project or area

- **WHEN** user runs `wai add research <content>`, `wai add plan <content>`, or `wai add design <content>`
- **THEN** the system creates a date-prefixed artifact file in the appropriate directory

#### Scenario: Show information

- **WHEN** user runs `wai show <item>`
- **THEN** the system displays the requested information

#### Scenario: Move items between PARA categories

- **WHEN** user runs `wai move <item> archives`
- **THEN** the system moves the item to the archives category

#### Scenario: Manage project phases

- **WHEN** user runs `wai phase`, `wai phase next`, `wai phase set <phase>`, or `wai phase back`
- **THEN** the system shows or transitions the current project's phase

#### Scenario: Sync agent configurations

- **WHEN** user runs `wai sync`
- **THEN** the system projects agent configs to tool-specific locations

#### Scenario: Manage agent configs

- **WHEN** user runs `wai config add skill <file>`, `wai config list`, or `wai config edit <path>`
- **THEN** the system manages agent configuration files

#### Scenario: Generate handoffs

- **WHEN** user runs `wai handoff create <project>`
- **THEN** the system generates a handoff document enriched with plugin data

#### Scenario: Search artifacts

- **WHEN** user runs `wai search <query>`
- **THEN** the system searches across all `.wai/` artifacts

#### Scenario: View timeline

- **WHEN** user runs `wai timeline <project>`
- **THEN** the system displays a chronological view of the project's artifacts

### Requirement: Global Flags

The CLI SHALL support global verbosity and quiet flags that work with all commands.

#### Scenario: Verbose output (level 1)

- **WHEN** user passes `-v` or `--verbose`
- **THEN** output includes additional context and metadata

#### Scenario: Verbose output (level 2)

- **WHEN** user passes `-vv` or `--verbose --verbose`
- **THEN** output includes debug information

#### Scenario: Verbose output (level 3)

- **WHEN** user passes `-vvv` or `--verbose --verbose --verbose`
- **THEN** output includes trace-level details

#### Scenario: Quiet mode

- **WHEN** user passes `-q` or `--quiet`
- **THEN** only errors are shown

### Requirement: Project Initialization

The CLI SHALL provide `wai init` to initialize a project in the current directory.

#### Scenario: Interactive initialization

- **WHEN** user runs `wai init` without arguments
- **THEN** the system prompts for project name (defaulting to directory name)
- **AND** creates `.wai/` structure with PARA directories (projects, areas, resources, archives, plugins)
- **AND** creates default agent-config directory with `.projections.yml`
- **AND** auto-detects available plugins (beads, openspec, git)

#### Scenario: Named initialization

- **WHEN** user runs `wai init --name my-project`
- **THEN** the system creates the project with the specified name without prompting

#### Scenario: Already initialized

- **WHEN** user runs `wai init` in an already-initialized directory
- **THEN** the system shows a warning and suggests `wai status`

### Requirement: Status Command

The CLI SHALL provide `wai status` to show project overview and suggest next steps.

#### Scenario: Show project phase and status

- **WHEN** user runs `wai status`
- **THEN** the system displays the current project's phase
- **AND** shows plugin status summaries (beads issues, openspec changes, git status)
- **AND** shows contextual suggestions based on current phase

#### Scenario: Contextual suggestions

See [context-suggestions](../context-suggestions/spec.md) for the complete suggestion logic.
