## ADDED Requirements

### Requirement: Check Command

The CLI SHALL provide `wai check` to run both repo hygiene and workspace health
checks in a single invocation, with distinct section headers separating the two
domains.

#### Scenario: Runs way checks then doctor checks

- **WHEN** user runs `wai check` in a directory with a `.wai/` workspace
- **THEN** the system runs all `wai way` checks and renders them under a
  "Repo hygiene (wai way)" section header
- **AND** runs all `wai doctor` checks and renders them under a
  "Workspace health (wai doctor)" section header
- **AND** both sections appear in order (way first, doctor second)

#### Scenario: Graceful degradation — no workspace

- **WHEN** user runs `wai check` in a directory without a `.wai/` workspace
- **THEN** the system runs all `wai way` checks and renders them normally
- **AND** prints `(workspace checks skipped — run \`wai init\` first)` in place
  of the doctor section
- **AND** exits with code 0 (absence of a workspace is not a failure)

### Requirement: Check Way-Only Flag

The CLI SHALL provide `--way-only` on `wai check` to run only repo hygiene
checks and skip workspace health checks.

#### Scenario: Way-only run

- **WHEN** user runs `wai check --way-only`
- **THEN** the system runs only the `wai way` checks
- **AND** the doctor section is absent from the output
- **AND** exits with code 0 (way checks never produce Fail status)

### Requirement: Check Doctor-Only Flag

The CLI SHALL provide `--doctor-only` on `wai check` to run only workspace
health checks and skip repo hygiene checks.

#### Scenario: Doctor-only run — workspace present

- **WHEN** user runs `wai check --doctor-only` in a directory with a `.wai/`
  workspace
- **THEN** the system runs only the `wai doctor` checks
- **AND** the way section is absent from the output
- **AND** exits with code 1 if any doctor check has Fail status

#### Scenario: Doctor-only run — no workspace

- **WHEN** user runs `wai check --doctor-only` in a directory without a `.wai/`
  workspace
- **THEN** the system prints `(workspace checks skipped — run \`wai init\` first)`
- **AND** exits with code 0

### Requirement: Check Combined Exit Code

`wai check` SHALL exit with a non-zero code when any check across either section
has status Fail.

#### Scenario: All checks pass

- **WHEN** `wai check` runs and no check has status Fail
- **THEN** the system exits with code 0

#### Scenario: Doctor section has a failure

- **WHEN** `wai check` runs and at least one doctor check has status Fail
- **THEN** the system exits with code 1

#### Scenario: Way section never causes failure

- **WHEN** `wai check` runs and all way checks are Pass or Info
- **AND** all doctor checks are Pass or Warn
- **THEN** the system exits with code 0

### Requirement: Check Section Headers

`wai check` human output SHALL use distinct section headers to separate the
two check domains.

#### Scenario: Section header format

- **WHEN** `wai check` renders human output
- **THEN** the way checks appear under a header identifying the "Repo hygiene
  (wai way)" domain
- **AND** the doctor checks appear under a header identifying the "Workspace
  health (wai doctor)" domain

#### Scenario: Skip message replaces header

- **WHEN** the doctor section is skipped due to no workspace
- **THEN** the skip message `(workspace checks skipped — run \`wai init\` first)`
  appears where the doctor section header would be

### Requirement: Check JSON Output

`wai check` SHALL support `--json` to emit machine-readable output combining
both check sections.

#### Scenario: JSON structure — both sections

- **WHEN** user runs `wai check --json` in a directory with a `.wai/` workspace
- **THEN** the system outputs a single JSON object with a `"way"` key and a
  `"doctor"` key
- **AND** each key's value is an object with `"checks"` (array) and `"summary"`
  (object) fields matching the existing JSON shapes of `wai way --json` and
  `wai doctor --json`

#### Scenario: JSON structure — no workspace

- **WHEN** user runs `wai check --json` in a directory without a `.wai/`
  workspace
- **THEN** the `"way"` key is present with its checks and summary
- **AND** the `"doctor"` key is `null`

#### Scenario: JSON structure — way-only

- **WHEN** user runs `wai check --way-only --json`
- **THEN** the output JSON contains only the `"way"` key (no `"doctor"` key)

#### Scenario: JSON structure — doctor-only

- **WHEN** user runs `wai check --doctor-only --json`
- **THEN** the output JSON contains only the `"doctor"` key (no `"way"` key)
