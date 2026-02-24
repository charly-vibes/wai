## ADDED Requirements

### Requirement: Session Orientation Command

The CLI SHALL provide `wai prime` to give the user a single at-a-glance orientation of
the current work session: active project, current phase, last handoff summary, plugin
status summaries, and a suggested next action.

The expected terminal output shape is:

```
◆ wai prime — 2026-02-24
• Project: why-command [review]
• Handoff: 2026-02-23 — 'Completed Phase 9.1 verbosity levels...'
• Beads:   3 open issues (2 ready)
• Spec:    add-why-command: 49/54 (91%)
→ Suggested next: bd show wai-abc
```

The handoff line is omitted when no handoff exists for the project. Plugin summary lines
appear in plugin-detection order, one per detected plugin. The suggested-next line is
omitted when beads is not detected or when there are no ready issues.

#### Scenario: Single project — prime renders full view

- **WHEN** user runs `wai prime` in a workspace with exactly one project
- **THEN** the system displays the project name, current phase, and plugin summaries
- **AND** includes the most recent handoff date and one-line snippet

#### Scenario: No handoff exists — handoff line omitted

- **WHEN** user runs `wai prime` and no handoff files exist for the project
- **THEN** the handoff line is omitted from the output
- **AND** the rest of the view renders normally

#### Scenario: Beads detected — suggested next shown

- **WHEN** user runs `wai prime` and the beads plugin is detected and at least one ready issue exists
- **THEN** the suggested-next line shows `bd show <id>` for the highest-priority ready issue

#### Scenario: No ready issues — suggested next omitted

- **WHEN** user runs `wai prime` and there are no ready beads issues (or beads not detected)
- **THEN** the suggested-next line is omitted from the output

#### Scenario: Multiple projects — prompts for selection

- **WHEN** user runs `wai prime` in a workspace with multiple projects and no `--project` flag
- **THEN** the system presents an interactive prompt listing available projects
- **AND** renders the prime view for the selected project

#### Scenario: Multiple projects — non-interactive mode fails gracefully

- **WHEN** user runs `wai prime` with multiple projects, no `--project` flag, and `--no-input`
- **THEN** the system fails with a diagnostic error
- **AND** suggests `wai prime --project <name>` with the available project names

#### Scenario: Explicit project flag

- **WHEN** user runs `wai prime --project <name>`
- **THEN** the system renders the prime view for the specified project without prompting

#### Scenario: No projects exist

- **WHEN** user runs `wai prime` in a workspace with no projects
- **THEN** the system fails with a diagnostic error
- **AND** suggests `wai new project <name>` to create one
