# pipeline-resume Specification

## ADDED Requirements

### Requirement: Prime surfaces resume-the-orchestration

`wai prime` output MUST include an explicit resume-the-orchestration line
when a pipeline run is active for the current project.

#### Scenario: Active pipeline run

- **WHEN** a pipeline run is active for the current project
- **THEN** `wai prime` output includes a resume-the-orchestration line

### Requirement: Prime suggestion is authoritative over epics flow

`wai prime` output MUST NOT include the older epics/`bd ready` flow suggestion — the resume
line is the single authoritative next step.

#### Scenario: Pipeline run wins over epics flow

- **WHEN** a pipeline run is active for the current project
- **THEN** `wai prime` output has no epics-flow suggestion
- **AND** the resume-the-orchestration line is the single next step

### Requirement: Sync includes orchestration breadcrumb

`bd sync` output MUST include the resume-the-orchestration breadcrumb when
a pipeline run is active for the current project.

#### Scenario: Sync surfaces breadcrumb

- **WHEN** a pipeline run is active for the current project
- **THEN** `bd sync` output includes the resume-the-orchestration breadcrumb
