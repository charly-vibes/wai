## MODIFIED Requirements

### Requirement: Welcome Screen

When wai is run without arguments and no project is detected, it SHALL show a welcoming entry point.

#### Scenario: No project detected

- **WHEN** user runs `wai` with no arguments
- **AND** no `.wai/` directory exists in current or parent directories
- **THEN** the system shows "wai - Workflow manager for AI-driven development"
- **AND** shows context: "No project detected in current directory"
- **AND** suggests: `wai init`, `wai new project`, `wai --help`
- **AND** includes a note that detailed help is available via `wai --help`

### Requirement: Project Context Welcome

When wai is run without arguments inside a project, it SHALL show project-relevant suggestions.

#### Scenario: Inside project

- **WHEN** user runs `wai` with no arguments
- **AND** a `.wai/` directory exists
- **THEN** the system suggests: `wai status`, `wai phase`, `wai new project`
- **AND** includes a note that detailed help is available via `wai --help`
