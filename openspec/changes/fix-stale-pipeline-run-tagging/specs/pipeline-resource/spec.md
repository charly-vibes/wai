## MODIFIED Requirements

### Requirement: Step-Level Artifact Tagging

Artifacts created during a pipeline run SHALL be tagged with both
`pipeline-run:<run-id>` and `pipeline-step:<step-id>` where `step-id` is the
ID of the currently active step in the run state. When the run is resolved
from the `.last-run` pointer file, the tag SHALL be stamped only when the
run state file exists, parses, and shows the run still in progress
(`current_step` below the final step count); when the run is resolved from
the `WAI_PIPELINE_RUN` environment variable, the tag SHALL be stamped
unconditionally (explicit intent).

#### Scenario: Artifact created during active step

- **WHEN** `wai add research "findings"` is called
- **AND** a pipeline run is active with run ID `sci-2026-04-02-qcd`
- **AND** the current step is `describe`
- **THEN** the artifact frontmatter includes:
  `tags: [pipeline-run:sci-2026-04-02-qcd, pipeline-step:describe]`

#### Scenario: No active pipeline run

- **WHEN** `wai add research "findings"` is called
- **AND** no pipeline run is active
- **THEN** no pipeline tags are injected (existing behavior unchanged)

#### Scenario: Retroactive tagging not supported

- **WHEN** `wai pipeline next` advances from step `decompose` to step `generate`
- **AND** the user then creates an artifact intended for step `decompose`
- **THEN** the artifact is tagged with `pipeline-step:generate` (the current step)
- **AND** the user must use `--tags pipeline-step:decompose` to override manually

#### Scenario: Pointer to completed run not tagged

- **WHEN** `wai add research "findings"` is called
- **AND** no `WAI_PIPELINE_RUN` environment variable is set
- **AND** `.last-run` names a run whose state file shows the final step
  reached (`current_step` at or beyond the step count)
- **THEN** no pipeline tags are injected (a completed run is stale provenance)
- **AND** the stale `.last-run` pointer is removed so later adds and
  `wai status` stop advertising the run

#### Scenario: Pointer to unverifiable run not tagged

- **WHEN** `wai add research "findings"` is called
- **AND** no `WAI_PIPELINE_RUN` environment variable is set
- **AND** `.last-run` names a run whose state file is missing or unreadable
- **THEN** no pipeline tags are injected (fail-closed)
- **AND** the `.last-run` pointer is kept (removal is reserved for runs
  proven complete)
