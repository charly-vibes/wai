## ADDED Requirements

### Requirement: Built-in Pipeline Templates

The CLI SHALL ship named built-in pipeline templates that can be scaffolded into a
workspace without authoring from scratch. The built-in set MUST include `tdd-ro5`,
`scientific-research`, and `epic-orchestrator`.

#### Scenario: Scaffold a built-in template

- **WHEN** user runs `wai pipeline init epic-orchestrator`
- **THEN** the system writes `.wai/resources/pipelines/epic-orchestrator.toml` from the
  built-in template
- **AND** the scaffolded file declares the orchestrator loop steps: claim, gates, brief,
  spawn, verify, ship, STOP

#### Scenario: Built-in templates are listed

- **WHEN** the CLI displays available pipeline templates (help text or `wai pipeline list`)
- **THEN** `epic-orchestrator` is offered alongside `tdd-ro5` and `scientific-research`

#### Scenario: Orchestrator template encodes the canon loop

- **WHEN** the epic-orchestrator template is read after scaffolding
- **THEN** its header cites the orchestrator-subagents canon pattern and its tool-agnostic
  invariants
- **AND** the loop terminates with an explicit STOP step at the ticket boundary (one
  ticket per advance)
