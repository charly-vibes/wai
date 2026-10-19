## ADDED Requirements

### Requirement: Spawn Preflight

An orchestrator run MUST verify the worktree environment at spawn time via
`wai pipeline preflight`, which checks at minimum: lefthook executable
presence, `bd list` exiting 0, and testaruda store initialization. Every
failed check MUST produce a message naming the tool and its fix; all check
failures MUST be reported in one pass. A failed preflight MUST abort the run
at spawn. The preflight MUST NOT install tools or make network calls, and
MUST NOT participate in step gate evaluation. The epic-orchestrator template's
spawn step MUST require the preflight to pass before spawning.

#### Scenario: Preflight failure aborts at spawn with an actionable message

- **WHEN** a worktree is spawned and `wai pipeline preflight` runs
- **AND** lefthook is not installed in the worktree
- **THEN** the preflight exits nonzero
- **AND** the message names lefthook and its fix (install path)
- **AND** the spawn step records the run as failed-at-spawn rather than
  proceeding to mid-run discovery

#### Scenario: All failures reported in one pass

- **WHEN** multiple environment problems exist (missing lefthook AND
  broken `bd` state)
- **THEN** one preflight run reports every failure, each naming its tool
- **AND** the agent does not need multiple runs to learn all problems

#### Scenario: Healthy environment passes without side effects

- **WHEN** lefthook is present, `bd list` exits 0, and the testaruda store
  is initialized
- **THEN** the preflight exits 0
- **AND** no files outside the tool stores are modified and no network call
  is made
