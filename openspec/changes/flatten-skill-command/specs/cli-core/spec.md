## MODIFIED Requirements

### Requirement: Command Structure

The CLI SHALL use consistent verb-noun command patterns with primary verbs: `new`,
`add`, `show`, `move`, plus dedicated top-level commands for `phase`, `sync`,
`config`, `handoff`, `search`, `timeline`, `why`, `reflect`, and `doctor`.

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
