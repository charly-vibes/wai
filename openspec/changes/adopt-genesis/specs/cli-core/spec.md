# cli-core spec delta: adopt genesis envelope

## MODIFIED Requirements

### Requirement: JSON Output

wai's `--json` output SHALL wrap every payload in `genesis::envelope::Envelope`
(the shared suite envelope) so that `--json` across all charly-vibes tools
shares the same top-level keys.

#### Scenario: status emits shared envelope

- **WHEN** `wai status --json` is run
- **THEN** the emitted JSON SHALL have top-level keys `ok`, `envelope_version`,
  `cli_version`, `envelope_kind`, `data`, `warnings`, `hints`, `meta`
- **AND** the wai-specific payload (phase, artifacts, suggestions) SHALL be
  nested under `data`
- **AND** the output SHALL be byte-compatible in envelope shape with the same
  command in dont, pretender, espectacular, and testaruda once they adopt.

#### Scenario: prime emits shared envelope

- **WHEN** `wai prime --json` is run
- **THEN** `PrimePayload` SHALL be wrapped in `genesis::envelope::Envelope`
  rather than emitted as a bare domain object.

#### Scenario: Status as JSON
- **WHEN** user runs `wai status --json`
- **THEN** the system outputs JSON containing phase, plugin statuses, and suggestion lists

#### Scenario: Search as JSON
- **WHEN** user runs `wai search <query> --json`
- **THEN** the system outputs JSON containing matches with file paths, line numbers, and context

#### Scenario: Timeline as JSON
- **WHEN** user runs `wai timeline <project> --json`
- **THEN** the system outputs JSON containing entries with date, type, title, and path

#### Scenario: Plugin list as JSON
- **WHEN** user runs `wai plugin list --json`
- **THEN** the system outputs JSON containing plugin name, status, and detector metadata