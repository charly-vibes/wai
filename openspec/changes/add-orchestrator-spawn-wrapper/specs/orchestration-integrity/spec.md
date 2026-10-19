## ADDED Requirements

### Requirement: Orchestrator Spawn Wrapper

Spawning a pi subagent inside an orchestrator run MUST go through the canonical
spawn wrapper (`wai pipeline spawn <name> <prompt>`, optional `--budget
<minutes>`, default 30). The wrapper MUST launch the spawn detached (returning
immediately), MUST stream output to a known log path under
`.wai/pipeline-runs/spawns/`, MUST enforce the wall-clock budget via the
`timeout` binary, and MUST write a checkpoint file
(`.wai/pipeline-runs/spawns/<name>.json`) recording the session name, pid, log
path, prompt, budget, and start timestamp. `wai pipeline spawn status
[name]` MUST print exactly one line per spawn answering: what is running, where
the log is, done or stuck — classified from pid liveness and a one-line log-tail
probe, never by re-deriving state from session JSONLs. `wai pipeline spawn
resume <name>` MUST verify the checkpoint exists and MUST launch the
`pi -c -p -n "<name>-resume"` invocation against the existing named session —
never re-prompting the full brief. The epic-orchestrator template's spawn step
MUST name the wrapper as the required spawn path and MUST name improvised
manual `pi -p` background incantations as a violation of the step. Gate
evaluation semantics as defined in Pipeline Gate Protocol are unchanged by
spawning.

#### Scenario: Agent spawns a subagent through the wrapper

- **WHEN** an orchestrator run needs a subagent for the spawn step
- **AND** the agent invokes `wai pipeline spawn <name> <prompt>`
- **THEN** the command returns immediately with the spawn running detached
- **AND** output streams to `.wai/pipeline-runs/spawns/<name>.log`
- **AND** a checkpoint file `.wai/pipeline-runs/spawns/<name>.json` exists
  recording session name, pid, log path, prompt path, budget, and start time

#### Scenario: One probe answers the progress question

- **WHEN** `wai pipeline spawn status` is invoked while a spawn is running
- **THEN** exactly one line is printed naming the spawn, its state
  (running/exited/budget-exhausted), and the log path
- **AND** the state is classified from pid liveness plus a one-line log-tail
  probe, without opening session JSONLs

#### Scenario: Interrupted-but-healthy spawn resumes in place

- **WHEN** `wai pipeline spawn resume <name>` is invoked for a checkpointed
  spawn whose session record exists
- **THEN** the resume launches detached as `pi -c -p -n "<name>-resume"`
  against the existing named session
- **AND** a fresh checkpoint `<name>-resume.json` is written
- **AND** the full brief is NOT re-prompted from scratch

#### Scenario: Manual background incantation is a named violation

- **WHEN** the epic-orchestrator template's spawn step is read
- **THEN** spawning via improvised manual `pi -p ... &` shell incantations is
  stated as a violation of the step
- **AND** the spawn wrapper is stated as the required path

#### Scenario: Wedged spawn costs a bounded budget

- **WHEN** `wai pipeline spawn` is invoked (with default or explicit budget)
- **AND** the spawned subagent never completes
- **THEN** the wall-clock budget is enforced by the `timeout` binary
- **AND** the spawn costs at most the budget, not the agent's bash timeout
