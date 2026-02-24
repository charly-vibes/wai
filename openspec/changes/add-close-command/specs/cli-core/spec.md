## ADDED Requirements

### Requirement: Session Close Command

The CLI SHALL provide `wai close` to automate the session-end checklist: create a handoff for the active project, surface uncommitted changes, and suggest the next steps.

The expected terminal output shape is:

```
✓ Handoff created: .wai/projects/<project>/handoffs/2026-02-24-session.md
! Uncommitted changes: src/main.rs, CLAUDE.md
→ Next: bd sync --from-main && git add <files> && git commit
```

The uncommitted-changes line is omitted when there are no changes. The `bd sync --from-main` prefix in the next-steps line is omitted when the beads plugin is not detected. When there are more than 10 uncommitted files, the line shows the first 10 and appends `… and N more`.

#### Scenario: Single project — close succeeds

- **WHEN** user runs `wai close` in a workspace with exactly one project
- **THEN** the system creates a handoff document for that project (identical to `wai handoff create <project>`)
- **AND** prints the handoff path on success

#### Scenario: No projects exist

- **WHEN** user runs `wai close` in a workspace with no projects
- **THEN** the system fails with a diagnostic error explaining that no projects exist
- **AND** suggests `wai new project <name>` to create one

#### Scenario: Multiple projects — prompts for selection

- **WHEN** user runs `wai close` in a workspace with multiple projects and no `--project` flag
- **THEN** the system presents an interactive prompt listing available projects
- **AND** creates a handoff for the selected project

#### Scenario: Multiple projects — non-interactive mode fails gracefully

- **WHEN** user runs `wai close` with multiple projects, no `--project` flag, and `--no-input`
- **THEN** the system fails with a diagnostic error
- **AND** suggests `wai close --project <name>` with the available project names

#### Scenario: Explicit project flag

- **WHEN** user runs `wai close --project <name>`
- **THEN** the system uses the specified project without prompting

#### Scenario: Project not found

- **WHEN** user runs `wai close --project <name>` and `<name>` does not exist
- **THEN** the system fails with a diagnostic error listing the available project names

#### Scenario: Uncommitted changes shown

- **WHEN** user runs `wai close` and the git plugin is detected and the workspace has uncommitted changes
- **THEN** the system lists the uncommitted files on a single line prefixed with `!`
- **AND** includes those filenames in the next-steps `git add` suggestion, each wrapped in double-quotes to handle filenames with spaces

#### Scenario: No uncommitted changes

- **WHEN** user runs `wai close` and the git plugin is detected and there are no uncommitted changes
- **THEN** the uncommitted-changes line is omitted from the output

#### Scenario: Git unavailable

- **WHEN** user runs `wai close` and git is not installed, or the workspace is not a git repository (git exits non-zero)
- **THEN** the git status section is silently skipped; the command still succeeds

#### Scenario: Repeated invocation on the same day

- **WHEN** user runs `wai close` more than once on the same calendar day
- **THEN** each invocation creates a new handoff file with an incrementing suffix (e.g. `session-end-1.md`, `session-end-2.md`)
- **AND** does not overwrite or error on the existing handoff

#### Scenario: Next-steps reminder includes bd sync when beads detected

- **WHEN** user runs `wai close` and the beads plugin is detected
- **THEN** the next-steps line reads: `→ Next: bd sync --from-main && git add <files> && git commit`

#### Scenario: Next-steps reminder without beads

- **WHEN** user runs `wai close` and the beads plugin is not detected
- **THEN** the next-steps line reads: `→ Next: git add <files> && git commit`
- **AND** does not mention `bd sync --from-main`
