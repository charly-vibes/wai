## ADDED Requirements

### Requirement: Pipeline Definition

The CLI SHALL support defining ordered multi-skill pipelines where each stage
declares the skill to invoke and the artifact type it produces.

#### Scenario: Create a pipeline

- **WHEN** user runs:
  `wai pipeline create issue-pipeline --stages="issue/gather:research,issue/create:tickets,issue/review:audit"`
- **THEN** the system saves the pipeline definition in
  `.wai/resources/pipelines/issue-pipeline.yml`
- **AND** validates that each stage references a known skill name

#### Scenario: Invalid stage format rejected

- **WHEN** a `--stages` value contains an entry without the `skill:artifact` format
- **THEN** the system rejects it with a clear error showing the expected format

### Requirement: Pipeline Run Lifecycle

The CLI SHALL support running a pipeline and tracking per-stage completion.

#### Scenario: Start a pipeline run

- **WHEN** user runs `wai pipeline run issue-pipeline --topic="ant-forager"`
- **THEN** the system generates a run ID (e.g., `issue-pipeline-2026-02-25-ant-forager`)
- **AND** stores initial run state in
  `.wai/resources/pipelines/issue-pipeline/runs/<run-id>.yml`
- **AND** outputs the run ID and a hint to invoke the Stage 1 skill

#### Scenario: Stage artifact tagged with run ID

- **WHEN** a skill runs in the context of a pipeline run
- **AND** the environment variable `WAI_PIPELINE_RUN=<run-id>` is set
- **AND** the skill calls any `wai add <type>` command (research, plan, design, or handoff)
- **THEN** the artifact is automatically tagged with `pipeline-run:<run-id>`

#### Scenario: Advance pipeline to next stage

- **WHEN** user runs `wai pipeline advance <run-id>`
- **THEN** the system marks the current stage complete
- **AND** records the artifact path created in that stage (via tag lookup)
- **AND** outputs the run ID and a hint to invoke the next stage skill

#### Scenario: Advance past last stage rejected

- **WHEN** user runs `wai pipeline advance <run-id>`
- **AND** all stages are already marked complete
- **THEN** the system errors with a message indicating the pipeline run is finished
- **AND** suggests starting a new run with `wai pipeline run`

#### Scenario: Advance with unknown run ID

- **WHEN** user runs `wai pipeline advance <run-id>`
- **AND** no run with that ID exists
- **THEN** the system errors with a clear message and lists active run IDs for the pipeline

### Requirement: Pipeline Status

The CLI SHALL provide a status view showing per-run, per-stage completion state.

#### Scenario: View pipeline status

- **WHEN** user runs `wai pipeline status issue-pipeline`
- **THEN** the system lists all runs with per-stage status:
  - Stage name and skill
  - Status: pending / complete
  - Artifact path (if complete)

#### Scenario: View single run detail

- **WHEN** user runs `wai pipeline status issue-pipeline --run <run-id>`
- **THEN** the system shows detailed state for that run only
