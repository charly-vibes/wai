## MODIFIED Requirements

### Requirement: Command Pass-Through

Plugins SHALL support routing CLI commands to external tools.

#### Scenario: Plugin command execution

- **WHEN** user runs `wai <plugin-name> <command>`
- **THEN** the system looks up the command in the plugin's command list
- **AND** executes the passthrough command

#### Scenario: Safe mode passthrough

- **WHEN** user passes `--safe`
- **THEN** the system refuses passthrough commands unless the plugin command is explicitly marked `read_only: true`
- **AND** returns a diagnostic error that suggests rerunning without `--safe`

#### Scenario: Non-interactive passthrough

- **WHEN** user passes `--no-input`
- **THEN** the system refuses passthrough commands that would prompt for input and returns a diagnostic error

### Requirement: Plugin Management

The CLI SHALL support listing, enabling, and disabling plugins.

#### Scenario: List plugins

- **WHEN** user runs `wai plugin list`
- **THEN** the system shows all known plugins with their status (enabled/disabled/not detected)

#### Scenario: List plugins as JSON

- **WHEN** user runs `wai plugin list --json`
- **THEN** the system outputs JSON containing plugin name, status, and detector metadata
