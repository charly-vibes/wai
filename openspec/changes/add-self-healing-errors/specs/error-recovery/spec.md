## ADDED Requirements

### Requirement: Typo Detection

The CLI SHALL detect typos in commands and suggest corrections.

#### Scenario: Unknown command with similar match

- **WHEN** user types an unknown command (e.g., `wai staus`)
- **AND** a similar valid command exists (e.g., `status`)
- **THEN** error says "Unknown command 'staus'"
- **AND** suggests "Did you mean 'wai status'?"

#### Scenario: Unknown subcommand with similar match

- **WHEN** user types an unknown subcommand (e.g., `wai new projet`)
- **AND** a similar valid subcommand exists (e.g., `project`)
- **THEN** error suggests the correct subcommand

### Requirement: Wrong Order Detection

The CLI SHALL detect reversed verb-noun patterns and suggest the correct order.

#### Scenario: Reversed command pattern

- **WHEN** user types `wai bead new "Title"`
- **THEN** error says "Unknown command 'bead'"
- **AND** suggests "Did you mean 'wai new bead \"Title\"'?"

### Requirement: Context Inference

The CLI SHALL infer project context from directory hierarchy.

#### Scenario: In project subdirectory

- **WHEN** user runs a project command from a subdirectory of a project
- **AND** `.para/` exists in a parent directory
- **THEN** the system uses the parent project context
- **OR** suggests "Run from project root: /path/to/project"

### Requirement: Sync Conflict Resolution

When sync conflicts occur, the CLI SHALL offer resolution strategies.

#### Scenario: Conflicting changes

- **WHEN** a sync operation detects conflicting changes
- **THEN** error explains "Changes conflict with remote"
- **AND** offers options: "keep local", "keep remote", "merge manually"
- **AND** shows command for each option

### Requirement: Conversational Error Tone

Error messages SHALL use friendly, conversational language.

#### Scenario: Error phrasing

- **WHEN** any error occurs
- **THEN** the message uses phrases like "Let's fix this" or "Here's how to continue"
- **AND** avoids technical jargon where possible
