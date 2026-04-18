## ADDED Requirements
### Requirement: Local Trace Discovery
The system SHALL discover local agent traces for the current repository from supported tool stores without requiring network access.

#### Scenario: Discover traces for current repo
- **WHEN** user runs `wai trace list`
- **THEN** the system scans known local trace roots for supported tools
- **AND** returns only traces whose recorded workspace path matches the current repository
- **AND** displays source tool, session identifier, time range, and fidelity tier for each match

#### Scenario: No traces found
- **WHEN** user runs `wai trace list`
- **AND** no supported tool store contains traces for the current repository
- **THEN** the system reports that no local traces were found for the repository
- **AND** suggests using `wai reflect --conversation <file>` if the user has an external transcript

### Requirement: Normalized Trace Model
The system SHALL normalize supported source formats into a common trace model with explicit fidelity metadata.

#### Scenario: Full transcript source normalized
- **WHEN** the source store contains user, assistant, and tool-call events
- **THEN** the normalized trace records the session as `full-transcript`
- **AND** preserves event ordering, timestamps, and source tool metadata

#### Scenario: Diff-only source normalized
- **WHEN** the source store contains only file-change snapshots and no reliable conversation log
- **THEN** the normalized trace records the session as `diff-only`
- **AND** captures touched files, before/after evidence, and timestamps
- **AND** does NOT fabricate user or assistant utterances

### Requirement: Trace Import
The system SHALL allow a selected normalized trace to be imported into `.wai/` as a first-class resource.

#### Scenario: Import selected trace
- **WHEN** user runs `wai trace import <trace-id>`
- **THEN** the system writes a trace resource under `.wai/resources/traces/`
- **AND** records source tool, source path, import timestamp, and fidelity tier in front matter
- **AND** preserves normalized event content or diff evidence in the body

#### Scenario: Import latest trace for repo
- **WHEN** user runs `wai trace import --latest`
- **THEN** the system selects the most recent trace matching the current repository
- **AND** imports it without requiring an explicit trace id

### Requirement: Source-specific Adapters
The system SHALL support source adapters for Claude Code, Codex, Gemini CLI, and AmpCode.

#### Scenario: Claude Code adapter
- **WHEN** the local Claude Code store contains repository-matching sessions
- **THEN** the system can discover and normalize those sessions as `full-transcript`

#### Scenario: Codex adapter
- **WHEN** the local Codex store contains repository-matching sessions
- **THEN** the system can discover and normalize those sessions as `full-transcript` or `partial-transcript`, depending on available events

#### Scenario: Gemini CLI adapter
- **WHEN** the local Gemini CLI store contains repository-matching sessions
- **THEN** the system can discover and normalize those sessions using available chat and log data

#### Scenario: AmpCode adapter
- **WHEN** the local AmpCode store contains repository-matching file-change evidence
- **THEN** the system can discover and normalize those sessions as `diff-only`
- **AND** marks them explicitly as reduced-fidelity input
