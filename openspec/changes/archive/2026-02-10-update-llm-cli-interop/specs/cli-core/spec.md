## MODIFIED Requirements

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

#### Scenario: Non-interactive mode

- **WHEN** user passes `--no-input`
- **THEN** the system disables interactive prompts and fails with a diagnostic error if input is required

#### Scenario: Auto-confirm

- **WHEN** user passes `--yes`
- **THEN** the system proceeds with default choices for confirmations

#### Scenario: Safe mode

- **WHEN** user passes `--safe`
- **THEN** the system runs in read-only mode and refuses operations that mutate state, returning a diagnostic error with a suggested non-safe command

### Requirement: JSON Output

Commands that return multi-line structured information SHALL support `--json` output for machine parsing.

#### Scenario: Status as JSON

- **WHEN** user runs `wai status --json`
- **THEN** the system outputs JSON containing phase, plugin statuses, and suggestion lists

#### Scenario: Search as JSON

- **WHEN** user runs `wai search <query> --json`
- **THEN** the system outputs JSON containing matches with file paths, line numbers, and context

#### Scenario: Timeline as JSON

- **WHEN** user runs `wai timeline <project> --json`
- **THEN** the system outputs JSON containing entries with date, type, title, and path

#### Scenario: Plugin list as JSON

- **WHEN** user runs `wai plugin list --json`
- **THEN** the system outputs JSON containing plugin name, status, and detector metadata
