## ADDED Requirements

### Requirement: Orchestrator Wait Primitive

Waiting on PR or CI status inside an orchestrator run MUST go through the
canonical blocking wait primitive (`wai pipeline wait <pr>`, optional timeout
argument, default 30m). The primitive MUST emit exactly one summary line
naming the PR, terminal state, and check count, and MUST exit 0 on success
and nonzero on check failure or timeout exhaustion. On timeout exhaustion the
summary line MUST include the elapsed budget and the last observed state. A
PR with zero checks MUST be treated as terminal-pass with a summary noting
zero checks. The epic-orchestrator template's spawn step MUST name the
primitive as the required way to wait on status and MUST name free-form
`sleep <N>` polling loops as a violation of the step. Gate evaluation
semantics as defined in Pipeline Gate Protocol are unchanged by waiting.

#### Scenario: Agent waits on a PR through the primitive

- **WHEN** an orchestrator run needs CI confirmation for a shipped PR
- **AND** the agent invokes `wai pipeline wait <pr>`
- **THEN** the command blocks until checks reach a terminal state
- **AND** exactly one summary line naming the PR, state, and check count is emitted
- **AND** the exit code is 0 on success and nonzero on check failure

#### Scenario: Free-form sleep polling is a named violation

- **WHEN** the epic-orchestrator template's spawn step is read
- **THEN** waiting on status via repeated `sleep <N>` poll commands is stated
  as a violation of the step
- **AND** the wait primitive is stated as the required path

#### Scenario: Wedged CI costs a bounded budget

- **WHEN** `wai pipeline wait <pr>` is invoked with a timeout argument
- **AND** the checks do not reach a terminal state within the timeout
- **THEN** the command exits nonzero
- **AND** its single summary line includes the elapsed budget and the last
  observed state, captured by re-querying check state once after the budget
  expires

#### Scenario: PR with zero checks is terminal-pass

- **WHEN** `wai pipeline wait <pr>` is invoked on a PR with no configured checks
- **THEN** the command exits 0
- **AND** its summary line names the PR and notes zero checks
