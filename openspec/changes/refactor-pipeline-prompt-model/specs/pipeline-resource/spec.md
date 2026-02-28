## ADDED Requirements

### Requirement: Pipeline Step Format

A pipeline definition SHALL be a TOML file at
`.wai/resources/pipelines/<name>.toml` containing an ordered list of steps,
each with a unique `id` and a `prompt` string. The `{topic}` placeholder in
any prompt SHALL be substituted with the topic value at render time.

#### Scenario: Valid pipeline file loaded

- **WHEN** a file `.wai/resources/pipelines/tdd.toml` exists with valid `[[steps]]`
  entries each having `id` and `prompt` fields
- **THEN** `wai pipeline start tdd --topic=auth-feature` loads it successfully
- **AND** renders `{topic}` as `auth-feature` in all prompt strings

#### Scenario: Duplicate step IDs rejected

- **WHEN** a pipeline TOML file contains two `[[steps]]` blocks with the same `id`
- **THEN** `wai pipeline start` fails with an error naming the duplicate ID

#### Scenario: Empty prompt rejected

- **WHEN** a pipeline TOML file contains a step with an empty or missing `prompt`
- **THEN** `wai pipeline start` fails with an error naming the step ID

### Requirement: Pipeline Initialization

The CLI SHALL provide a `pipeline init <name>` command that scaffolds a
minimal starter TOML template at `.wai/resources/pipelines/<name>.toml`,
creating the pipelines directory if it does not exist.

#### Scenario: Init creates starter template

- **WHEN** user runs `wai pipeline init tdd`
- **AND** `.wai/resources/pipelines/tdd.toml` does not exist
- **THEN** the system creates the file with a two-step starter template
- **AND** prints the file path and a prompt to edit it

#### Scenario: Init fails if file exists

- **WHEN** user runs `wai pipeline init tdd`
- **AND** `.wai/resources/pipelines/tdd.toml` already exists
- **THEN** the system errors with a clear message and does not overwrite the file

### Requirement: Pipeline Session Recovery

The CLI SHALL provide a `pipeline current` command that re-prints the active
step's prompt. The active run SHALL be resolved from `WAI_PIPELINE_RUN`
environment variable first, then from a `.last-run` pointer file at
`.wai/resources/pipelines/.last-run`.

#### Scenario: Re-orient after context loss via env var

- **WHEN** user runs `wai pipeline current`
- **AND** `WAI_PIPELINE_RUN` is set in the environment
- **THEN** the system prints the current step's prompt with all substitutions applied
- **AND** shows step position (e.g., "Step 2/4: tests")

#### Scenario: Re-orient using last-run pointer

- **WHEN** user runs `wai pipeline current`
- **AND** `WAI_PIPELINE_RUN` is not set
- **AND** `.wai/resources/pipelines/.last-run` exists
- **THEN** the system resolves the run ID from the pointer file
- **AND** prints the current step's prompt as above

#### Scenario: No active run

- **WHEN** user runs `wai pipeline current`
- **AND** neither `WAI_PIPELINE_RUN` nor `.last-run` exists
- **THEN** the system errors with a message explaining how to start a run

### Requirement: Pipeline Run Lifecycle

The CLI SHALL support starting a pipeline run with `pipeline start` and
advancing through steps with `pipeline next`. `pipeline start` writes a
`.last-run` pointer. `pipeline next` resolves the active run from
`WAI_PIPELINE_RUN` or `.last-run` and prints the next step's prompt.
On the final step, `pipeline next` prints a completion block.

#### Scenario: Start a pipeline run

- **WHEN** user runs `wai pipeline start tdd --topic="auth-feature"`
- **THEN** the system generates a run ID (e.g., `tdd-2026-02-27-auth-feature`)
- **AND** stores initial run state in `.wai/resources/pipelines/tdd/runs/<run-id>.yml`
- **AND** writes the run ID to `.wai/resources/pipelines/.last-run`
- **AND** prints the env export line: `export WAI_PIPELINE_RUN=<run-id>`
- **AND** prints the first step's prompt with `{topic}` substituted

#### Scenario: Stage artifact tagged with run ID

- **WHEN** `WAI_PIPELINE_RUN=<run-id>` is set in the environment
- **AND** user calls any `wai add <type>` command
- **THEN** the artifact is automatically tagged with `pipeline-run:<run-id>`

#### Scenario: Advance to next step

- **WHEN** user runs `wai pipeline next`
- **THEN** the system resolves the active run from `WAI_PIPELINE_RUN` or `.last-run`
- **AND** marks the current step complete, recording any tagged artifact path
- **AND** increments to the next step
- **AND** prints the next step's prompt with substitutions applied

#### Scenario: Completion on final step

- **WHEN** user runs `wai pipeline next` on the last step
- **THEN** the system marks the step complete
- **AND** prints a completion block showing all steps complete
- **AND** suggests `wai close` as the next action

#### Scenario: Next called on already-complete run

- **WHEN** user runs `wai pipeline next`
- **AND** the resolved run is already complete
- **THEN** the system errors: "Run <id> is already complete. Start a new run with `wai pipeline start`."

### Requirement: Pipeline Status

The CLI SHALL provide a status view showing per-run, per-step completion state,
and SHALL indicate the active run when a `.last-run` pointer or `WAI_PIPELINE_RUN`
is present.

#### Scenario: View all runs for a pipeline

- **WHEN** user runs `wai pipeline status <name>`
- **THEN** the system lists all runs with per-step status and artifact paths

#### Scenario: View single run detail

- **WHEN** user runs `wai pipeline status <name> --run <run-id>`
- **THEN** the system shows detailed state for that run only

#### Scenario: Active run highlighted

- **WHEN** user runs `wai pipeline status <name>`
- **AND** a run matching `.last-run` or `WAI_PIPELINE_RUN` is in the list
- **THEN** that run is visually marked as active
- **AND** the current step's prompt is shown at the bottom
