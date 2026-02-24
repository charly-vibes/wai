## ADDED Requirements

### Requirement: Pending-Resume Signal

`wai close` SHALL write a `.pending-resume` file to the project directory after
every successful handoff creation, enabling `wai prime` to detect resume context
without requiring the user to manually open the handoff.

#### Scenario: Signal written on close

- **WHEN** `wai close` successfully creates a handoff
- **THEN** the system writes `.wai/projects/<project>/.pending-resume`
- **AND** the file contains the path to the new handoff, relative to the project
  directory (e.g. `handoffs/2026-02-24-session-end.md`)

#### Scenario: Signal overwritten on repeated close

- **WHEN** user runs `wai close` more than once (same day or different days)
- **THEN** the `.pending-resume` file is overwritten with the path of the newest
  handoff
