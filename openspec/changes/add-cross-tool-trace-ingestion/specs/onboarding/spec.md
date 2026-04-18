## MODIFIED Requirements
### Requirement: Project Context Welcome
When wai is run without arguments inside a project, it SHALL show project-relevant suggestions.

#### Scenario: Inside project
- **WHEN** user runs `wai` with no arguments
- **AND** a `.wai/` directory exists
- **THEN** the system suggests: `wai status`, `wai search`, `wai prime`, `wai new project`
- **AND** includes a note that detailed help is available via `wai --help`

#### Scenario: Review-heavy workflow hint
- **WHEN** user runs `wai` with no arguments inside a project that has active pipelines installed
- **THEN** the system includes a short note suggesting `wai pipeline list` for larger guided workflows
