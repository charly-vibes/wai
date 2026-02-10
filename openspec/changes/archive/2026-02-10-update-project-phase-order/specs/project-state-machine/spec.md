## MODIFIED Requirements
### Requirement: Phase Definitions

Projects SHALL progress through defined phases that represent workflow stages.

#### Scenario: Default phase

- **WHEN** a new project is created
- **THEN** it starts in the "research" phase

#### Scenario: Phase list

- **WHEN** querying available phases
- **THEN** the system returns: research, design, plan, implement, review, archive
