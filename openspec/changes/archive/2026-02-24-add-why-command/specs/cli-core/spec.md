## MODIFIED Requirements

### Requirement: Command Structure

The CLI SHALL use consistent verb-noun command patterns with primary verbs: `new`, `add`, `show`, `move`, plus dedicated top-level commands for `phase`, `sync`, `config`, `handoff`, `search`, `timeline`, `why`, and `doctor`.

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

#### Scenario: Ask reasoning questions

- **WHEN** user runs `wai why <query>`
- **THEN** the system uses LLM synthesis to answer why decisions were made
- **AND** displays relevant artifacts, decision chains, and suggestions
- **AND** gracefully falls back to `wai search` if no LLM available

#### Scenario: Diagnose workspace health

- **WHEN** user runs `wai doctor`
- **THEN** the system runs diagnostic checks against the workspace
- **AND** reports pass/warn/fail status for each check with fix suggestions
- **AND** exits with code 0 when all checks pass, 1 when any check fails
