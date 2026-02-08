## MODIFIED Requirements

### Requirement: Command Structure

The CLI SHALL use consistent verb-noun command patterns with primary verbs: `new`, `add`, `show`, `move`, plus dedicated top-level commands for `phase`, `sync`, `config`, `handoff`, `search`, `timeline`, and `doctor`.

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

#### Scenario: Diagnose workspace health

- **WHEN** user runs `wai doctor`
- **THEN** the system runs diagnostic checks against the workspace
- **AND** reports pass/warn/fail status for each check with fix suggestions
- **AND** exits with code 0 when all checks pass, 1 when any check fails

## ADDED Requirements

### Requirement: Doctor Command

The CLI SHALL provide `wai doctor` to diagnose workspace health and report issues with actionable fix suggestions.

#### Scenario: Directory structure check

- **WHEN** `wai doctor` runs
- **THEN** it verifies that all expected `.wai/` subdirectories exist (projects, areas, resources, archives, plugins)
- **AND** reports pass if all present, fail with `mkdir` suggestion for each missing directory

#### Scenario: Configuration validation

- **WHEN** `wai doctor` runs
- **THEN** it attempts to parse `.wai/config.toml`
- **AND** reports pass if valid, fail with the parse error and suggestion to check the file

#### Scenario: Plugin tool availability

- **WHEN** `wai doctor` runs and plugins are detected
- **THEN** it checks whether each detected plugin's CLI tool is installed (e.g., `git`, `bd`, `openspec`)
- **AND** reports pass if reachable, warn if not installed with install guidance

#### Scenario: Agent config sync status

- **WHEN** `wai doctor` runs and `.projections.yml` exists
- **THEN** it validates the projections file parses correctly
- **AND** checks whether each projection target exists and is up to date
- **AND** reports pass if synced, warn if targets are missing with `wai sync` suggestion

#### Scenario: Project state integrity

- **WHEN** `wai doctor` runs and projects exist
- **THEN** it validates each project's `.state` file parses as valid YAML with a recognized phase
- **AND** reports pass if valid, fail with the error for each invalid state file

#### Scenario: Custom plugin validation

- **WHEN** `wai doctor` runs and `.wai/plugins/` contains YAML files
- **THEN** it validates each plugin YAML parses correctly as a PluginDef
- **AND** reports pass if valid, fail with the parse error for each invalid file

#### Scenario: Summary output

- **WHEN** all diagnostic checks complete
- **THEN** the system prints a summary line with total pass, warn, and fail counts
- **AND** exits with code 0 if no failures, code 1 if any failures

#### Scenario: Not initialized

- **WHEN** user runs `wai doctor` outside a wai workspace
- **THEN** the system reports the standard not-initialized error with `wai init` suggestion
