## ADDED Requirements
### Requirement: Explicit session loop guidance in managed block
The generated WAI block SHALL include concise session-start and session-close guidance that works across supported agent tools, not only inside Claude Code-style environments.

#### Scenario: Managed block includes explicit session-start loop
- **WHEN** `wai init` or `wai init --update` generates the managed block
- **THEN** the block includes a concise session-start sequence centered on `wai status`, `wai search`, and `wai prime`
- **AND** the wording is suitable for tools that do not share Claude Code's dogfooding context

#### Scenario: Managed block includes explicit session-close loop
- **WHEN** `wai init` or `wai init --update` generates the managed block
- **THEN** the block includes a concise session-close sequence centered on `wai close`
- **AND** clarifies that `wai close` should be preferred over ad hoc `/clear` or equivalent reset commands when preserving continuity matters
