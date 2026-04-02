## ADDED Requirements

### Requirement: Project Resolution

All project-scoped commands SHALL resolve the target project using a unified
algorithm with a deterministic priority order: explicit flag, then environment
variable, then auto-detection.

Project-scoped commands are: `phase`, `add`, `close`, `prime`, `reflect`,
`handoff create`, `timeline`, and `search --in`.

#### Scenario: Explicit flag wins

- **WHEN** user passes `--project <name>` to any project-scoped command
- **THEN** the system uses that project regardless of `WAI_PROJECT` env var
- **AND** validates the project exists in `.wai/projects/`
- **AND** fails with a diagnostic error listing available projects if not found

#### Scenario: Environment variable fallback

- **WHEN** no `--project` flag is provided
- **AND** `WAI_PROJECT` environment variable is set to a non-empty value
- **THEN** the system uses the project named by `WAI_PROJECT`
- **AND** validates the project exists in `.wai/projects/`
- **AND** fails with a diagnostic error if the named project does not exist

#### Scenario: Empty environment variable treated as unset

- **WHEN** `WAI_PROJECT` is set to an empty string
- **THEN** the system treats it as if the variable is not set
- **AND** proceeds to auto-detection

#### Scenario: Auto-detect single project

- **WHEN** no `--project` flag is provided
- **AND** `WAI_PROJECT` is not set (or empty)
- **AND** exactly one project exists in `.wai/projects/`
- **THEN** the system uses that project automatically

#### Scenario: Multiple projects without context — interactive

- **WHEN** no `--project` flag is provided
- **AND** `WAI_PROJECT` is not set
- **AND** more than one project exists in `.wai/projects/`
- **AND** stdin is a terminal and `--no-input` is not set
- **THEN** the system presents an interactive project selector

#### Scenario: Multiple projects without context — non-interactive

- **WHEN** no `--project` flag is provided
- **AND** `WAI_PROJECT` is not set
- **AND** more than one project exists in `.wai/projects/`
- **AND** stdin is not a terminal or `--no-input` is set
- **THEN** the system fails with an error listing available projects
- **AND** the error suggests `--project <name>` or `export WAI_PROJECT=<name>`

#### Scenario: No projects

- **WHEN** no projects exist in `.wai/projects/`
- **THEN** the system fails with a diagnostic error suggesting `wai new project <name>`

#### Scenario: Resolution source displayed

- **WHEN** a project-scoped command resolves a project
- **AND** the command displays the project name in its output
- **THEN** the output includes a source indicator when resolution was not via
  auto-detect: `[via --project]` or `[via WAI_PROJECT]`

#### Scenario: Auto-detect searches projects only

- **WHEN** the system auto-detects projects for resolution
- **THEN** it counts only directories in `.wai/projects/`
- **AND** does not count `.wai/areas/`, `.wai/resources/`, or `.wai/archives/`

### Requirement: Project Use Command

The CLI SHALL provide `wai project use <name>` to print a shell export statement
for session-scoped project binding.

#### Scenario: Valid project

- **WHEN** user runs `wai project use <name>`
- **AND** the named project exists in `.wai/projects/`
- **THEN** the system prints the appropriate export statement to stdout
- **AND** the export syntax matches the user's shell: `export WAI_PROJECT=<name>`
  for bash/zsh, `set -gx WAI_PROJECT <name>` for fish

#### Scenario: Valid project — terminal hint

- **WHEN** user runs `wai project use <name>` and stdout is a terminal
- **THEN** the system prints the export statement to stdout
- **AND** prints a usage hint to stderr: `# Paste the line above, or run: eval $(wai project use <name>)`

#### Scenario: Invalid project

- **WHEN** user runs `wai project use <name>`
- **AND** the named project does not exist
- **THEN** the system fails with a diagnostic error listing available projects

#### Scenario: No arguments

- **WHEN** user runs `wai project use` without a name
- **THEN** the system lists available projects with their current phases

## MODIFIED Requirements

### Requirement: Command Structure

The CLI SHALL use consistent verb-noun command patterns with primary verbs: `new`, `add`, `show`, `move`, plus dedicated top-level commands for `phase`, `sync`, `config`, `handoff`, `search`, `timeline`, `why`, `reflect`, and `doctor`.

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
- **THEN** the system shows or transitions the resolved project's phase
- **AND** the target project is resolved via the unified project resolution algorithm
- **AND** all phase subcommands accept `--project <name>` as an optional flag

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

#### Scenario: Select project for session

- **WHEN** user runs `wai project use <name>`
- **THEN** the system prints a shell export statement for session-scoped project binding
