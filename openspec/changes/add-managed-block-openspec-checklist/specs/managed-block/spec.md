## ADDED Requirements

### Requirement: Openspec step in session-close checklist

The generated WAI block's session-close checklist SHALL include a step to mark
completed openspec tasks `[x]` when the openspec plugin is detected. The step
SHALL appear after the beads steps (if present) and before `wai reflect`.

#### Scenario: Openspec detected at init time

- **WHEN** `wai init` or `wai reflect` runs
- **AND** the openspec plugin is detected
- **THEN** the generated WAI block's "Ending a Session" checklist includes:
  `[ ] openspec tasks.md — mark completed tasks [x]`

#### Scenario: Openspec not detected

- **WHEN** `wai init` or `wai reflect` runs
- **AND** the openspec plugin is NOT detected
- **THEN** the generated WAI block does NOT include the openspec checklist step

---

### Requirement: Cross-tool tracking convention

When both beads and openspec plugins are detected, the generated WAI block SHALL
include a "Tracking Work Across Tools" section that defines the cross-reference
convention for linking beads tickets to openspec task IDs.

The section SHALL appear after the "Capturing Work" section and before "Ending
a Session".

The convention SHALL specify:
- When creating a beads ticket for an openspec task, include the task reference
  (format: `<change-id>:<phase>.<task>`, e.g. `add-why-command:7.1`) in the
  ticket description.
- When closing a beads ticket linked to an openspec task, also check the
  corresponding box (`[x]`) in the change's `tasks.md`.

#### Scenario: Both plugins detected

- **WHEN** `wai init` or `wai reflect` runs
- **AND** both beads and openspec plugins are detected
- **THEN** the generated WAI block includes a "Tracking Work Across Tools"
  section with the cross-reference format and sync rule

#### Scenario: Only one plugin detected

- **WHEN** `wai init` or `wai reflect` runs
- **AND** only one of beads or openspec is detected (not both)
- **THEN** the "Tracking Work Across Tools" section is NOT included
