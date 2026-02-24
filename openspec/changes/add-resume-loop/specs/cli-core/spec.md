## MODIFIED Requirements

### Requirement: Session Close Command

`wai close` SHALL write a `.pending-resume` signal after every successful handoff
creation.

#### Scenario: Pending-resume written on success

- **WHEN** `wai close` successfully creates a handoff document
- **THEN** the system writes `.wai/projects/<project>/.pending-resume` containing
  the path to the new handoff, relative to the project directory
- **AND** this file is not mentioned in the command's terminal output
- **AND** the file appears in the uncommitted-changes list (it is a tracked
  workspace artifact, committed with other `.wai/` changes)

---

### Requirement: Session Orientation Command

`wai prime` SHALL detect a `.pending-resume` signal and render a `⚡ RESUMING`
block when the referenced handoff is dated today.

#### Scenario: Resume mode — today's handoff

- **WHEN** user runs `wai prime`
- **AND** `.wai/projects/<project>/.pending-resume` exists
- **AND** the referenced handoff file exists and its frontmatter `date` equals
  today's date
- **THEN** the system renders a `⚡ RESUMING` block before the plugin status lines
- **AND** the block shows the handoff date and one-line snippet on the first line
- **AND** the block shows the contents of the handoff's `## Next Steps` section
  immediately below, rendered as described in "Resume mode — next steps rendering"
- **AND** the normal `• Handoff:` line is omitted (replaced by the RESUMING block)
- **AND** the `.pending-resume` file is NOT modified or deleted

#### Scenario: Resume mode — next steps rendering

- **WHEN** the RESUMING block is rendered
- **THEN** the `  Next Steps:` label is printed indented two spaces from the
  left margin, with no `##` heading markers and with a trailing colon
- **AND** each content line from the `## Next Steps` section is printed indented
  four spaces from the left margin
- **AND** blank lines and lines starting with `<!--` within the section are skipped

#### Scenario: Resume mode — next steps present

- **WHEN** the handoff referenced by `.pending-resume` contains a `## Next Steps`
  section with renderable content (non-blank, non-comment lines)
- **THEN** the RESUMING block shows the `  Next Steps:` label followed by the items

#### Scenario: Resume mode — no next steps section

- **WHEN** the handoff referenced by `.pending-resume` does not contain a
  `## Next Steps` section, or the section contains only blank lines and HTML
  comments
- **THEN** the RESUMING block shows only the `⚡ RESUMING: {date} — '{snippet}'`
  header line with no indented items

#### Scenario: Signal not consumed by prime

- **WHEN** user runs `wai prime` and a RESUMING block is rendered
- **THEN** the `.pending-resume` file is NOT modified or deleted
- **AND** a subsequent `wai prime` call in the same session renders the same
  RESUMING block again

#### Scenario: Stale signal — not today's handoff

- **WHEN** user runs `wai prime`
- **AND** `.wai/projects/<project>/.pending-resume` exists
- **AND** the referenced handoff's frontmatter `date` is before today, or the
  date field is missing or unparseable
- **THEN** the system ignores the signal entirely
- **AND** renders the normal `• Handoff:` line using the latest handoff

#### Scenario: Missing handoff — signal ignored

- **WHEN** user runs `wai prime`
- **AND** `.pending-resume` exists but the referenced file does not exist on disk
- **THEN** the system ignores the signal
- **AND** renders the normal `• Handoff:` line (or omits the line if no handoffs
  exist at all)
