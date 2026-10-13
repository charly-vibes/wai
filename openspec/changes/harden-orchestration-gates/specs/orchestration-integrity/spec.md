# orchestration-integrity Specification

## ADDED Requirements

### Requirement: Step Artifact Manifest Gates

Orchestrator pipeline templates MUST declare structural gates on
artifact-producing steps (at minimum: implement, ship, and any step whose
output feeds a later gate), requiring the step-tagged artifact types the
step is responsible for (this includes epic-orchestrator and
epic-autonomy-tdd-ro5). Advancement past such a step MUST block while the
declared artifacts are missing.

#### Scenario: Manifest step blocks without artifacts

- **WHEN** the `implement` step of an orchestrator run declares a structural
  gate requiring `min_artifacts = 1, types = ["design"]`
- **AND** `wai pipeline next` is called
- **AND** no design artifact is tagged with this step and run
- **THEN** advancement is blocked with a message naming the missing artifact type

#### Scenario: Manifest step passes with artifacts

- **WHEN** the declared artifacts exist and are tagged with the active step
- **THEN** the structural gate passes and advancement proceeds to procedural
  and oracle tiers as configured

#### Scenario: RED violation class is closed

- **WHEN** a run reaches the step following `implement` without any GREEN
  evidence artifact (design or review) tagged for the implement step
- **THEN** `wai pipeline next` blocks advancement and the run cannot reach
  `refactor-or-tidy` or later steps until the artifact exists

### Requirement: Gate Bypass Telemetry

A gate bypass via `--no-verify` during an active pipeline run MUST be
recordable in the run-state ledger (timestamp, command, disclosed reason)
via a `wai pipeline bypass` record command — hooks cannot self-record a
bypass because `--no-verify` skips them — and agent-facing guidance MUST
instruct recording at bypass time. `wai pipeline status` MUST surface the
bypass count and the most recent reason. Equivalent disclosed-bypass paths
are treated the same way.

#### Scenario: Bypass during active run recorded

- **WHEN** an agent runs `wai pipeline bypass <reason>` during an active
  pipeline run
- **THEN** the run-state ledger contains the entry
- **AND** `wai pipeline status` shows the bypass count and last disclosed reason

#### Scenario: Bypass without active run ignored

- **WHEN** a bypassed commit occurs with no active pipeline run
- **THEN** no run-state entry is written and no error is raised

### Requirement: Verifier Evidence Contract

A verify subagent session MUST produce recorded evidence before its
CONFIRM verdict is valid: the review artifact cites the artifacts it read
and includes a fresh test-run output. A verify session lacking the required
evidence MUST record REVIEW_REQUIRED instead of CONFIRM. A verify performed
by the same model as the implement step (single-model degradation) MUST
record REVIEW_REQUIRED, never CONFIRM.

#### Scenario: Verify with evidence yields CONFIRM

- **WHEN** the verify review artifact cites ≥1 read artifact path and includes
  fresh test-run output
- **THEN** the verdict CONFIRM is valid

#### Scenario: Verify without test-run output is not CONFIRM

- **WHEN** the review artifact lacks fresh test-run output
- **THEN** the recorded verdict is REVIEW_REQUIRED

#### Scenario: Single-model degradation never CONFIRM

- **WHEN** the verify step runs on the same model as the implement step
- **THEN** the recorded verdict is REVIEW_REQUIRED with a degradation note

### Requirement: Cross-Agent Collision Surfacing

`wai prime` (and `wai pipeline next` when a run is active) MUST warn when
workspace state indicates a parallel agent has parked work: a git stash
entry whose message matches agent-provenance patterns (agent name, "parked",
"owns"), or an agent lock-file under `.wai/` that is present and unexpired.
The warning MUST be advisory and MUST NOT hard-fail.

#### Scenario: Parked stash from parallel agent surfaced

- **WHEN** a stash entry message names a parallel agent and a ticket
  (e.g., "evallerina-00n WIP (parked — parallel agent owns wai)")
- **THEN** `wai prime` output includes a collision warning naming the agent
  and the referenced ticket

#### Scenario: No collision state stays silent

- **WHEN** no stash entry matches provenance patterns and no lock-file exists
- **THEN** prime and pipeline output are unchanged (silent-when-green)
