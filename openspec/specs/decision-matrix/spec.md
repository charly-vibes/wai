# decision-matrix Specification

## Purpose
TBD - created by archiving change add-decision-matrix. Update Purpose after archive.
## Requirements
### Requirement: Matrix location and identity

The decision matrix SHALL be a single directory tree per project at
`.wai/projects/<project>/designs/matrix/`. It SHALL contain `problem.md`
(the decision being deliberated), `criteria/`, `approaches/`, and
`decision.md`. The filesystem is the source of truth; the CLI is sugar over
plain file operations.

#### Scenario: Init scaffolds the matrix

- **GIVEN** a project without a matrix
- **WHEN** `wai matrix init` runs with a problem statement
- **THEN** `designs/matrix/` contains `problem.md`, `criteria/`,
  `approaches/01-status-quo/` with `_description.md`, and an empty
  `decision.md`

#### Scenario: One matrix per project

- **GIVEN** a project that already has `designs/matrix/`
- **WHEN** `wai matrix init` runs
- **THEN** the command fails with an error naming the existing matrix

### Requirement: Cell encoding

Each cell SHALL be the directory
`approaches/<approach>/<criterion>/` containing `fact.md` (the aspect: facts,
not judgment) and exactly one judgment marker file named `neutral`, `green`,
`yellow`, or `red`. A cell is **incomplete** if and only if `fact.md` is
missing or empty. `neutral` is a legitimate judgment, distinct from
incomplete.

#### Scenario: Neutral judgment is explicit

- **GIVEN** a cell with a non-empty `fact.md` and a marker file named
  `neutral`
- **WHEN** `wai matrix lint` runs
- **THEN** the cell passes validation

#### Scenario: Multiple markers are a structural error

- **GIVEN** a cell containing both `green` and `red` marker files
- **WHEN** `wai matrix lint` runs
- **THEN** lint exits non-zero and names the offending cell

#### Scenario: Missing fact means incomplete, not judged

- **GIVEN** a cell directory with a marker but no `fact.md`
- **WHEN** `wai matrix lint` runs
- **THEN** the cell is reported as incomplete

### Requirement: Status quo first

`wai matrix init` SHALL create `approaches/01-status-quo/` first, and lint
SHALL report an error if it is missing.

#### Scenario: Init creates status quo

- **GIVEN** a project without a matrix
- **WHEN** `wai matrix init` runs
- **THEN** `approaches/01-status-quo/_description.md` exists

### Requirement: Multi-operation helpers

`wai matrix criterion add <name>` SHALL create the criterion definition file
and cell directories in every existing approach.
`wai matrix approach add <name>` SHALL create the approach directory with
`_description.md` and empty cell directories for every existing criterion.
These exist because one logical action spans multiple directories; no command
is provided for single-cell writes, which are direct file edits.

#### Scenario: Criterion add touches every approach

- **GIVEN** a matrix with two approaches
- **WHEN** `wai matrix criterion add "operational-cost"` runs
- **THEN** `criteria/NN-operational-cost.md` exists
- **AND** `approaches/*/03-operational-cost/` exists for every approach

### Requirement: Deciding produces a current-answer marker and a design doc

`wai matrix decide <approach> <rationale>` SHALL validate the approach
directory exists, write `decision.md` (selected approach, rationale, date,
pointer to the design doc), and scaffold a design artifact in
`designs/<date>-<slug>.md` with standard frontmatter (`tracks` supported) and
a decision-time snapshot of the winning column's cell texts.

#### Scenario: Decide writes both artifacts

- **GIVEN** a matrix with two approaches and filled cells
- **WHEN** `wai matrix decide "02-event-sourcing" "rationale text"` runs
- **THEN** `decision.md` names approach `02-event-sourcing`
- **AND** a new design artifact exists in `designs/` containing the rationale
  and the winning column's aspect texts

#### Scenario: Decide rejects unknown approach

- **GIVEN** no approach directory matches the argument
- **WHEN** `wai matrix decide` runs
- **THEN** the command fails, listing valid approach names

### Requirement: Deterministic HTML render on demand

`wai matrix render` SHALL generate a self-contained `matrix.html` (inline
CSS, no JS) as a pure function of the directory state: `problem.md` banner,
criteria as rows, approaches as columns, marker files as judgment chips,
assessment key included. Output is deterministic (byte-identical for equal
input) and is a generated view — it SHALL NOT be committed or treated as a
source of truth. Empty cells render as a gray "not yet assessed" placeholder,
never a judgment color.

#### Scenario: Same state, same output

- **GIVEN** an unchanged matrix directory
- **WHEN** `wai matrix render` runs twice
- **THEN** both runs produce byte-identical HTML

#### Scenario: Unfilled cell is not a judgment

- **GIVEN** an approach/criterion pair with no `fact.md`
- **WHEN** `wai matrix render` runs
- **THEN** that cell renders with a neutral "not yet assessed" placeholder
  that is not colored green, yellow, or red

### Requirement: Structural lints

`wai matrix lint` SHALL enforce, as errors (non-zero exit):

- Rectangularity: every approach directory contains a cell directory for
  every criterion
- Exactly one non-empty-named marker from {neutral, green, yellow, red} per
  filled cell
- No empty `fact.md` files
- `01-status-quo` present as first column

#### Scenario: Missing cell in one approach fails lint

- **GIVEN** an approach directory lacking a cell directory present in the
  other approaches
- **WHEN** `wai matrix lint` runs
- **THEN** exit code is non-zero and the missing path is named

### Requirement: Methodology lints are warnings

`wai matrix lint` SHALL report, as warnings that never affect exit code:

- All-green column (possible rationalization)
- Undistinguished columns (identical cell texts across approaches for all
  criteria — likely a missing criterion)
- Subjective judgment words as the body of `fact.md`
- Link-only cells (bare URL, no summary text)
- Criteria definitions phrased as questions (trailing `?`)
- Status-quo column with no red cell (something important was missed —
  per Miller, "Design in Practice in Practice")
- Decision recorded while the selected approach has unfilled cells
- Empty `problem.md`
- Matrix files newer than the decision timestamp in `decision.md`
  (stale decision; inconclusive timestamp comparison after a fresh clone
  downgrades to this warning rather than a gate block)

#### Scenario: All-green column warns but passes

- **GIVEN** one approach whose filled cells all have `green` markers
- **WHEN** `wai matrix lint` runs
- **THEN** a warning names the approach
- **AND** exit code is 0

#### Scenario: Undistinguished columns warn

- **GIVEN** two approaches whose cell texts are identical for every criterion
- **WHEN** `wai matrix lint` runs
- **THEN** a warning suggests a distinguishing criterion is missing

### Requirement: Design-to-plan phase gate

The `design → plan` phase transition SHALL be blocked when a matrix exists
and any of the following hold: `problem.md` is empty, `decision.md` does not
record a selected approach that exists under `approaches/`, or any file under
`approaches/` or `criteria/` has an mtime strictly newer than the decision
UTC timestamp recorded inside `decision.md`. Staleness checks whose
comparison is inconclusive (equal or missing timestamps, e.g. after a fresh
clone) SHALL downgrade to a warning instead of blocking. The failure message
SHALL be self-healing: it names the offending files and points at
`wai matrix decide`. Projects without a matrix are never gated.

#### Scenario: Current decision passes the gate

- **GIVEN** a matrix whose `decision.md` names an existing approach and no
  matrix file is newer than the recorded decision timestamp
- **WHEN** `wai phase next` advances from design
- **THEN** the transition proceeds

#### Scenario: Scaffolded decision does not pass the gate

- **GIVEN** a freshly initialized matrix whose `decision.md` is still the
  template (no approach recorded)
- **WHEN** the design → plan transition is attempted
- **THEN** the transition is blocked
- **AND** the message indicates no decision has been made

#### Scenario: Edited-after-decision blocks the gate

- **GIVEN** a cell file modified after `decision.md` was written
- **WHEN** the design → plan transition is attempted
- **THEN** the transition is blocked
- **AND** the message names the stale-decision condition and suggests
  `wai matrix decide`

#### Scenario: Ambiguous timestamps after a fresh clone warn instead of blocking

- **GIVEN** a decided matrix checked out fresh from git so that all matrix
  files share the checkout timestamp
- **WHEN** the design → plan transition is attempted
- **THEN** the transition proceeds
- **AND** a stale-decision warning is reported with a pointer to
  `wai matrix lint`

#### Scenario: No matrix means no gate

- **GIVEN** a project that never ran `wai matrix init`
- **WHEN** the design → plan transition is attempted
- **THEN** the transition proceeds as before this change

### Requirement: Status and handoff matrix awareness

`wai status` in the design phase SHALL show the current problem statement,
cell completion count, and a suggestion for the next unfilled cell.
`wai close` handoffs SHALL include the problem statement, the selected
approach with rationale, and links to both the matrix directory and the
scaffolded design doc.

#### Scenario: Status shows completion

- **GIVEN** a matrix with 5 of 6 cells filled
- **WHEN** `wai status` runs during the design phase
- **THEN** output includes the problem statement, `5/6 cells filled`, and a
  pointer to the unfilled cell

