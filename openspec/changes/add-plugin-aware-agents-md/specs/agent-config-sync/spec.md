## ADDED Requirements

### Requirement: Plugin-Aware Managed Block

The managed block injected into `AGENTS.md` and `CLAUDE.md` SHALL generate content conditional on which plugins are detected in the project.

#### Scenario: Wai-only project

- **WHEN** `wai init` runs in a project with no companion tools detected
- **THEN** the managed block contains wai instructions only
- **AND** does not include tool landscape or disambiguation sections

#### Scenario: Wai with beads detected

- **WHEN** `wai init` runs and `.beads/` directory exists
- **THEN** the managed block includes beads in the tool landscape
- **AND** includes beads commands in the quick reference
- **AND** includes beads in session start/end protocols
- **AND** distinguishes wai (reasoning/why) from beads (tasks/what)

#### Scenario: Wai with openspec detected

- **WHEN** `wai init` runs and `openspec/` directory exists
- **THEN** the managed block includes openspec in the tool landscape
- **AND** references `openspec/AGENTS.md` for detailed spec instructions
- **AND** distinguishes wai (reasoning/why) from openspec (specs/requirements)

#### Scenario: All tools detected

- **WHEN** `wai init` runs with beads, openspec, and git all detected
- **THEN** the managed block includes a "When to Use What" decision table
- **AND** lists all detected tools with their purpose
- **AND** provides a unified session start and end protocol

#### Scenario: Re-init updates block with current plugins

- **WHEN** `wai init` runs in an already-initialized project
- **THEN** the system re-detects plugins
- **AND** regenerates the managed block content based on current detection
- **AND** preserves content outside the `<!-- WAI:START -->` / `<!-- WAI:END -->` markers

### Requirement: Tool Landscape Section

When companion tools are detected, the managed block SHALL begin with a tool landscape overview.

#### Scenario: Tool landscape content

- **WHEN** at least one companion tool (beads or openspec) is detected
- **THEN** the managed block starts with a list of detected tools and their roles
- **AND** provides a brief description of each tool's purpose
- **AND** clearly states the boundary: wai = reasoning/why, beads = tasks/what, openspec = specs/requirements

### Requirement: Unified Session Protocols

The managed block SHALL provide unified session start and end protocols that reference all detected tools.

#### Scenario: Session start with all tools

- **WHEN** beads and openspec are both detected
- **THEN** the session start section includes `wai status`, `bd ready`, and `openspec list`
- **AND** presents them as a single coherent workflow

#### Scenario: Session end with beads

- **WHEN** beads is detected
- **THEN** the session end section includes `wai handoff create`, issue status updates via `bd close`, and filing new issues via `bd create`
- **AND** presents a single ordered checklist instead of competing protocols
