## ADDED Requirements

### Requirement: Available Pipelines in managed block

The generated WAI block SHALL include an "Available Pipelines" section when
pipelines with `[pipeline.metadata]` are installed, listing each pipeline's name,
`when` description, and start command. The section SHALL also note that pipeline
steps may have gates and reference `wai pipeline gates` for gate details.

#### Scenario: Pipelines with metadata installed

- **WHEN** `wai init` or `wai init --update` runs
- **AND** `.wai/resources/pipelines/` contains one or more pipelines with `[pipeline.metadata]`
- **THEN** the generated WAI block includes an "Available Pipelines" table with
  columns: Pipeline, When to Use, Start command
- **AND** a note: "Pipeline steps have gates that enforce artifact creation, review
  coverage, and oracle checks before advancement."

#### Scenario: No pipelines installed

- **WHEN** `wai init` runs
- **AND** no pipelines exist in `.wai/resources/pipelines/`
- **THEN** no "Available Pipelines" section is generated

#### Scenario: Pipeline without metadata

- **WHEN** a pipeline exists but lacks `[pipeline.metadata]`
- **THEN** that pipeline is NOT included in the "Available Pipelines" table
- **AND** `wai doctor` warns about the missing metadata

---

### Requirement: Managed block staleness detection

`wai doctor` SHALL detect when the managed block in CLAUDE.md or AGENTS.md is
out of date relative to the current workspace state (installed pipelines,
plugins, skills). Detection SHALL compare the generated block content against
the actual content between `WAI:START` and `WAI:END` markers.

#### Scenario: Block is stale after pipeline added

- **WHEN** `wai doctor` runs
- **AND** a pipeline with metadata has been added since the last `wai init --update`
- **AND** the CLAUDE.md managed block does not include that pipeline
- **THEN** doctor reports warning: "CLAUDE.md managed block outdated — run 'wai init --update' to refresh"

#### Scenario: Block is current

- **WHEN** `wai doctor` runs
- **AND** the managed block content matches what would be generated
- **THEN** no staleness warning is reported

#### Scenario: AGENTS.md also checked

- **WHEN** `wai doctor` runs
- **AND** AGENTS.md exists with WAI:START/WAI:END markers
- **THEN** staleness detection runs on AGENTS.md as well

#### Scenario: User-modified managed block always shows stale

- **WHEN** `wai doctor` runs
- **AND** the user has manually edited content between WAI:START and WAI:END markers
- **THEN** doctor reports the block as outdated
- **AND** running `wai init --update` will overwrite the user's modifications
  (content between markers is managed by wai)
