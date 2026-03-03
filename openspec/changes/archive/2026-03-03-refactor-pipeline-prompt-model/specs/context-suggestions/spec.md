## ADDED Requirements

### Requirement: Pipeline Run Awareness

The status command SHALL surface an active pipeline run when a `.last-run`
pointer file exists or `WAI_PIPELINE_RUN` is set in the environment, so
agents can re-orient after context loss without human intervention.

#### Scenario: Active run surfaced in status output

- **WHEN** user runs `wai status`
- **AND** `.wai/resources/pipelines/.last-run` exists with a valid run ID
- **THEN** the status output includes a pipeline section:
  "⚡ PIPELINE ACTIVE: <pipeline-name> (step N/M: <step-id>)"
- **AND** the suggestions block includes: `wai pipeline current`

#### Scenario: No active run — no pipeline section

- **WHEN** user runs `wai status`
- **AND** no `.last-run` file exists and `WAI_PIPELINE_RUN` is not set
- **THEN** the status output does not include a pipeline section
