## ADDED Requirements

### Requirement: Way Command

The CLI SHALL provide `wai way` to validate repository best practices and provide opinionated recommendations ("the wai way") using a dedicated status model separate from `wai doctor`.

#### Scenario: Repository standards check

- **WHEN** `wai way` runs
- **THEN** it checks for repository best practices as defined in [repository-best-practices](../repository-best-practices/spec.md)
- **AND** includes checks for task runner, git hook manager (prek/pre-commit), EditorConfig, documentation, AI instructions, llm.txt, agent skills, CI/CD, and dev containers
- **AND** reports each check as WayStatus::Pass (✓) or WayStatus::Info (ℹ) with actionable suggestions

#### Scenario: Works without wai initialization

- **WHEN** user runs `wai way` in any directory (wai workspace or not)
- **THEN** the command succeeds and runs all repository checks
- **AND** does NOT require `.wai/` to exist
- **AND** helps users prepare repositories before running `wai init`

#### Scenario: Summary output

- **WHEN** all checks complete
- **THEN** the system prints a summary: "X/Y best practices adopted"
- **AND** suggests quick-start priorities if many checks are info status
- **AND** always exits with code 0 (recommendations never fail)

#### Scenario: Output format

- **WHEN** `wai way` runs
- **THEN** output is grouped under "The wai way" header
- **AND** uses ✓ (green) for WayStatus::Pass, ℹ (blue) for WayStatus::Info
- **AND** each check follows format: "Category: Status (details)"
- **AND** includes actionable fix text with URLs (in parentheses) for each WayStatus::Info
- **AND** supports `--json` flag for machine-readable output
- **AND** critical recommendations (missing .gitignore/README.md) display with ⚠️ marker
