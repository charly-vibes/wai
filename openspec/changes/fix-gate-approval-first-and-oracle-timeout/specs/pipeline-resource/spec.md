## MODIFIED Requirements

### Requirement: Pipeline Gate Protocol

The CLI SHALL evaluate pipeline gates in strict order when `wai pipeline next`
is called on a step with gates configured. A gate is a validation check on a
pipeline step. Gates are organized into four tiers: structural (artifact
existence), procedural (review coverage), oracle (domain-specific checks), and
approval (human checkpoint). The approval tier SHALL be evaluated before the
oracle tier, so an unsatisfied human checkpoint blocks advancement without
spending oracle wall-time. The first failing tier SHALL block advancement with
a descriptive reason. Steps without gates SHALL advance freely (existing
behavior).

#### Scenario: Structural gate blocks on missing artifacts

- **WHEN** step `generate` declares `[steps.gate.structural]` with `min_artifacts = 1, types = ["research"]`
- **AND** user runs `wai pipeline next`
- **AND** no research artifacts exist tagged with this step and run
- **THEN** advancement is blocked with message: "Step 'generate' requires at least 1 research artifact(s). Found 0."

#### Scenario: Procedural gate blocks on missing review

- **WHEN** step declares `[steps.gate.procedural]` with `require_review = true`
- **AND** a research artifact exists for this step
- **AND** no review artifact references that research artifact
- **THEN** advancement is blocked with message: "Artifact '2026-04-02-findings.md' has no review."

#### Scenario: Procedural gate scopes reviewable types

- **WHEN** step declares `[steps.gate.procedural]` with `require_review = true` and `review_types = ["research", "plan"]`
- **AND** a research artifact and a design artifact exist for this step
- **THEN** the procedural gate requires a review for the research artifact
- **AND** the design artifact is not required to have a review (not in `review_types`)

#### Scenario: Procedural gate defaults exclude reviews

- **WHEN** step declares `[steps.gate.procedural]` with `require_review = true` and no `review_types`
- **THEN** the procedural gate requires reviews for all artifact types EXCEPT review artifacts
- **AND** review artifacts are never required to have reviews of their own

#### Scenario: Procedural gate blocks on severity threshold

- **WHEN** step declares `[steps.gate.procedural]` with `max_critical = 0`
- **AND** a review artifact exists with `severity: {critical: 2}`
- **THEN** advancement is blocked with message: "Review of 'findings.md' has 2 critical findings (max: 0)."

#### Scenario: Approval gate evaluated before oracles

- **WHEN** step declares `[steps.gate.approval]` with `required = true`
- **AND** the step has oracle gates configured
- **AND** the step has not been approved via `wai pipeline approve`
- **THEN** the oracles are not executed
- **AND** advancement is blocked with the approval message, without spending
  oracle wall-time

#### Scenario: Oracle gate blocks on script failure

- **WHEN** step declares an oracle named `dimensional-analysis`
- **AND** the oracle script exits with code 1 and stderr "Dimensional mismatch in eq. 14"
- **THEN** advancement is blocked with message: "Oracle 'dimensional-analysis' failed for 'findings.md': Dimensional mismatch in eq. 14"

#### Scenario: Oracle timeout

- **WHEN** an oracle declares `timeout = 30`
- **AND** the oracle script does not exit within 30 seconds
- **THEN** the oracle process is killed and reaped
- **AND** it is treated as a gate failure with message: "Oracle 'name' timed out after 30s (killed)"
- **AND** any remaining oracles for the step still run, and advancement stays
  blocked (the failure blocks `wai pipeline next`; it does not abort the CLI)

#### Scenario: Oracle without explicit timeout uses enforced default

- **WHEN** an oracle does not declare `timeout`
- **AND** the oracle script does not exit within 30 seconds
- **THEN** the oracle process is killed and the step fails with the timeout
  message (the implicit 30s default is enforced, not ignored)

#### Scenario: Oracle with zero timeout waits indefinitely

- **WHEN** an oracle declares `timeout = 0`
- **THEN** no deadline is enforced and the CLI waits until the oracle exits
  (existing behavior preserved)

#### Scenario: Approval gate blocks until approved

- **WHEN** step declares `[steps.gate.approval]` with `required = true`
- **AND** the structural, procedural, and coverage gates pass
- **AND** the step has not been approved via `wai pipeline approve`
- **THEN** advancement is blocked with message: "This step requires human approval. Run 'wai pipeline approve' when ready."

#### Scenario: Approval gate passes after approval

- **WHEN** `wai pipeline approve` has been run for the current step
- **AND** all other gates pass
- **THEN** `wai pipeline next` advances to the next step

#### Scenario: Approval invalidated by new artifacts

- **WHEN** `wai pipeline approve` has been run for the current step
- **AND** new artifacts are created for this step after the approval timestamp
- **THEN** the approval is invalidated
- **AND** `wai pipeline next` blocks with message: "Approval invalidated — new artifacts created after approval. Run 'wai pipeline approve' again."

#### Scenario: No gates configured

- **WHEN** a step has no gate configuration in TOML
- **THEN** `wai pipeline next` advances immediately (existing behavior)
