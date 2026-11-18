## MODIFIED Requirements
### Requirement: Command Structure
The CLI SHALL use consistent verb-noun command patterns with primary verbs: `new`, `add`, `show`, `move`, plus dedicated top-level commands for `phase`, `sync`, `config`, `handoff`, `search`, `timeline`, `why`, `reflect`, `doctor`, and `trace`.

#### Scenario: Create new items
- **WHEN** user runs `wai new project <name>`, `wai new area <name>`, or `wai new resource <name>`
- **THEN** the system creates the requested PARA item with appropriate directory structure

#### Scenario: Add artifacts and resources to a project
- **WHEN** user runs `wai add research <content>`, `wai add plan <content>`,
  `wai add design <content>`, or `wai add skill <name>`
- **THEN** the system creates the requested artifact or resource file in the
  appropriate directory

#### Scenario: Add skill — creates skill file
- **WHEN** user runs `wai add skill <name>`
- **THEN** the system creates a skill file at
  `.wai/resources/skills/<name>.md` (or `.wai/resources/skills/<category>/<action>.md`
  for hierarchical names)
- **AND** the same name validation rules apply as for `wai resource add skill`:
  flat or one-level-hierarchical, lowercase letters/digits/hyphens only

#### Scenario: Add skill with template
- **WHEN** user runs `wai add skill <name> --template <template>`
- **THEN** the system creates the skill file pre-populated with the named template
- **AND** valid built-in templates are: `gather`, `create`, `tdd`, `rule-of-5`

#### Scenario: Deprecated alias still works
- **WHEN** user runs `wai resource add skill <name>`
- **THEN** the system emits a one-line deprecation warning to stderr:
  `⚠ 'wai resource add skill' is deprecated. Use: wai add skill <name>`
- **AND** creates the skill file identically to `wai add skill <name>`
- **AND** exits with the same exit code as `wai add skill`

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

#### Scenario: Reflect on session history
- **WHEN** user runs `wai reflect`
- **THEN** the system reads accumulated handoffs and artifacts
- **AND** optionally accepts a conversation transcript via `--conversation <file>`
- **AND** uses LLM synthesis to surface project-specific conventions, gotchas, and patterns
- **AND** shows a unified diff of old vs proposed REFLECT block content
- **AND** requires user confirmation before writing to CLAUDE.md and/or AGENTS.md
- **AND** updates whichever AI config files exist in the repo root by default
- **AND** fails with a clear diagnostic if no LLM is available (does not fall back)

#### Scenario: Diagnose workspace health
- **WHEN** user runs `wai doctor`
- **THEN** the system runs diagnostic checks against the workspace
- **AND** reports pass/warn/fail status for each check with fix suggestions
- **AND** exits with code 0 when all checks pass, 1 when any check fails

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
