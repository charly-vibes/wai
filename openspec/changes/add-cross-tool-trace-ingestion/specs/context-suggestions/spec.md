## MODIFIED Requirements
### Requirement: Contextual suggestions
The system SHALL provide contextual workflow suggestions based on current project state and recent interaction patterns.

#### Scenario: Review-remediation loop detected
- **WHEN** the system has evidence from recent local traces that the current repository is in a repeated review/remediation loop
- **AND** the loop includes repeated continuation or fix-up prompts without artifact capture
- **THEN** `wai status` suggests capturing findings with `wai add plan` or `wai add research`
- **AND** suggests using an available pipeline when one matches the work type

#### Scenario: Research-heavy work with available pipeline
- **WHEN** recent local trace evidence shows repeated research, review, and synthesis behavior
- **AND** an installed pipeline advertises itself as suitable for that kind of work
- **THEN** `wai status` suggests the matching pipeline start command
- **AND** explains briefly why the pipeline fits the observed workflow
