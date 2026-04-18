## MODIFIED Requirements
### Requirement: Command Structure
The CLI SHALL use consistent verb-noun command patterns with primary verbs: `new`, `add`, `show`, `move`, plus dedicated top-level commands for `phase`, `sync`, `config`, `handoff`, `search`, `timeline`, `why`, `reflect`, `doctor`, and `trace`.

#### Scenario: Create new items
- **WHEN** user runs `wai new project <name>`, `wai new area <name>`, or `wai new resource <name>`
- **THEN** the system creates the requested PARA item with appropriate directory structure

#### Scenario: Trace discovery
- **WHEN** user runs `wai trace list`
- **THEN** the system lists discovered local traces for the current repository

#### Scenario: Trace import
- **WHEN** user runs `wai trace import <trace-id>`
- **THEN** the system imports the selected trace into `.wai/resources/traces/`

#### Scenario: Reflect auto-selects recent trace
- **WHEN** user runs `wai reflect`
- **AND** no `--conversation <file>` is provided
- **AND** one or more local traces are available for the current repository
- **THEN** the system selects the highest-ranked recent trace automatically for reflection context
- **AND** reports which source trace was used

#### Scenario: Reflect explicit conversation override
- **WHEN** user runs `wai reflect --conversation <file>`
- **THEN** the explicit file remains higher priority than any auto-detected local trace
