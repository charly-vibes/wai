## MODIFIED Requirements

### Requirement: Reflect Output Storage

The system SHALL write `wai reflect` output to a versioned resource file in
`.wai/resources/reflections/` instead of injecting a full block into `CLAUDE.md`
or `AGENTS.md`.

#### Scenario: Reflect writes to resource file

- **WHEN** user confirms a `wai reflect` run
- **THEN** the system writes the synthesized content to
  `.wai/resources/reflections/YYYY-MM-DD-<project>.md`
- **AND** the file is created (never overwritten; a second run on the same day
  appends a `-2`, `-3` suffix)
- **AND** the file begins with YAML front-matter:
  ```yaml
  ---
  date: YYYY-MM-DD
  project: <project-name>
  sessions_analyzed: N
  type: reflection
  ---
  ```
- **AND** the LLM-generated content follows the front-matter

#### Scenario: Resource file is searchable

- **WHEN** user runs `wai search "<topic>"` (without `--project`)
- **THEN** reflection files in `.wai/resources/reflections/` are included in
  search results alongside project artifacts (wai search without --project
  scans the entire `.wai/` tree, which includes resources/)
- **AND** results display the reflection date and project name via the file path

- **WHEN** user runs `wai search "<topic>" --project <name>`
- **THEN** search is scoped to that project's artifact directories only
- **AND** reflection files in `.wai/resources/reflections/` are NOT included
- **NOTE** The WAI:REFLECT:REF block instructs agents to use bare
  `wai search "<topic>"` (without --project) to ensure reflections are found

#### Scenario: Previous reflections included as context

- **WHEN** gathering context for a new `wai reflect` run
- **THEN** existing files in `.wai/resources/reflections/` for the current project
  are included as an additional context tier (lower priority than handoffs)
- **AND** the LLM prompt instructs the LLM to extend and correct existing patterns
  rather than repeat them

### Requirement: Slim Reference Block in CLAUDE.md / AGENTS.md

The system SHALL maintain a `WAI:REFLECT:REF` block in `CLAUDE.md` and/or
`AGENTS.md` that contains only a brief pointer to the resource files and a
mandatory search instruction.

#### Scenario: wai init injects WAI:REFLECT:REF block

- **WHEN** user runs `wai init` (fresh or refresh)
- **THEN** the system injects or refreshes a `WAI:REFLECT:REF:START/END` block
  in `CLAUDE.md` (and `AGENTS.md` if present) after the `WAI:END` marker
- **AND** the block contains:
  ```markdown
  ## Accumulated Project Patterns

  Project-specific conventions, gotchas, and architecture notes live in
  `.wai/resources/reflections/`. Run `wai search "<topic>"` to retrieve relevant
  context before starting research or creating tickets.

  > **Before research or ticket creation**: always run `wai search "<topic>"` to
  > check for known patterns. Do not rediscover what is already documented.
  ```
- **AND** the block is injected even if no reflection files exist yet (the
  search instruction is always valid)

#### Scenario: wai reflect does NOT touch CLAUDE.md

- **WHEN** user runs `wai reflect`
- **THEN** the system writes only to `.wai/resources/reflections/<date>-<project>.md`
- **AND** does NOT modify `CLAUDE.md` or `AGENTS.md`
- **AND** does NOT inject, update, or remove any managed block in those files

#### Scenario: Migration from old WAI:REFLECT block

- **WHEN** user runs `wai reflect`
- **AND** `CLAUDE.md` or `AGENTS.md` contains an existing `WAI:REFLECT:START/END` block
- **THEN** the system applies this unified rule:
  - If no `<project>-migrated.md` resource file exists yet: read the block
    content from the first target file that has it and write it to
    `.wai/resources/reflections/<today>-<project>-migrated.md`
  - If multiple target files have old blocks (e.g. both CLAUDE.md and AGENTS.md),
    only the first detected file's content is migrated (content is expected to
    be identical)
  - Replace the `WAI:REFLECT:START/END` block in ALL target files that have it
    with the slim `WAI:REFLECT:REF:START/END` block
- **AND** prints: "Migrated existing REFLECT block to .wai/resources/reflections/"
- **AND** proceeds with the normal reflect run (writing a new dated resource file)

### Requirement: Search-Before-Research Instruction in Managed Block

The `wai init` managed block template SHALL include an explicit instruction for
agents to search for known patterns before beginning research or creating tickets.

#### Scenario: Managed block includes search instruction when companions detected

- **WHEN** `wai init` generates the managed block
- **AND** beads or openspec companion tools are detected
- **THEN** the managed block includes (after the TDD/Tidy First disclaimer):
  ```
  > **Before research or ticket creation**: run `wai search "<topic>"` to check
  > for known patterns in `.wai/resources/reflections/` before writing new content.
  ```
- **AND** this instruction appears in every managed block refresh (`wai init --refresh`)

#### Scenario: No instruction without companions

- **WHEN** `wai init` generates the managed block
- **AND** no companion tools are detected
- **THEN** the managed block does NOT include the search instruction
  (no reflections accumulate in wai-only projects)

## REMOVED Requirements

### Requirement: Reflect Managed Block (superseded)

The previous requirement that `wai reflect` injects a `WAI:REFLECT:START/END`
block into `CLAUDE.md` and/or `AGENTS.md` is superseded by the resource-based
storage and slim reference block described above.

The `WAI:REFLECT:START/END` marker pair is deprecated. `wai init` will no longer
generate or update content within those markers. The migration scenario above
handles existing deployments.
