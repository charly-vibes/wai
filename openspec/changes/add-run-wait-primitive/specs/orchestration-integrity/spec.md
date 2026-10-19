## ADDED Requirements

### Requirement: Orchestrator Wait Primitive

Waiting on PR or CI status inside an orchestrator run MUST go through the
canonical blocking wait primitive (`just ci-wait <pr>`, optional timeout
argument, default 30m). The epic-orchestrator template's spawn step MUST name
the primitive as the required way to wait on status and MUST name free-form
`sleep <N>` polling loops as a violation of the step. The primitive MUST emit
exactly one summary line (PR, terminal state, check count) and MUST exit
nonzero when checks fail. Gate evaluation semantics are unchanged by waiting.

#### Scenario: Agent waits on a PR through the primitive

- **WHEN** an orchestrator run needs CI confirmation for a shipped PR
- **AND** the agent invokes `just ci-wait <pr>`
- **THEN** the command blocks until checks reach a terminal state
- **AND** exactly one summary line naming the PR, state, and check count is emitted
- **AND** the exit code is 0 on success and nonzero on check failure

#### Scenario: Free-form sleep polling is a named violation

- **WHEN** the epic-orchestrator template's spawn step is read
- **THEN** waiting on status via repeated `sleep <N>` poll commands is stated
  as a violation of the step
- **AND** the wait primitive is stated as the required path

#### Scenario: Wedged CI costs a bounded budget

- **WHEN** `just ci-wait <pr>` is invoked with a timeout argument
- **AND** the checks do not reach a terminal state within the timeout
- **THEN** the command exits nonzero with the elapsed budget and the last
  observed state in its single summary line
