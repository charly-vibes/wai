## ADDED Requirements

### Requirement: Artifact Locking

The CLI SHALL support locking pipeline step artifacts with SHA-256 hashes to
prevent post-hoc modification. Locking SHALL be opt-in per step via the
`lock = true` field in pipeline TOML. Lock metadata SHALL be stored in a
`.lock` sidecar file (TOML format) alongside each locked artifact.

#### Scenario: Lock current step artifacts

- **WHEN** a pipeline run is active
- **AND** user runs `wai pipeline lock`
- **THEN** the system computes SHA-256 hashes (LF-normalized) of all artifacts
  tagged with the current step
- **AND** writes a `.lock` sidecar file for each artifact (named
  `<artifact>.<run-id>.lock`) containing: `artifact`, `locked_at`,
  `lock_hash`, `pipeline_run`, `pipeline_step`
- **AND** marks the step as locked in run state

#### Scenario: Auto-lock on advancement

- **WHEN** user runs `wai pipeline next`
- **AND** the current step has `lock = true` in its TOML definition
- **THEN** the system locks all current step artifacts before advancing
- **AND** proceeds with normal gate checks and advancement

#### Scenario: Lock with zero artifacts

- **WHEN** a pipeline run is active
- **AND** user runs `wai pipeline lock` or `wai pipeline next` with `lock = true`
- **AND** no artifacts are tagged with the current step
- **THEN** the system errors: "Cannot lock step '<step-id>' with no artifacts."
- **AND** does NOT advance

#### Scenario: Detect tampered locked artifact

- **WHEN** a locked artifact's content has been modified since locking
- **AND** user runs `wai pipeline verify` or `wai doctor`
- **THEN** the system reports the integrity violation with file path and
  expected vs actual hash
- **AND** suggests creating an addendum instead of editing the locked artifact

#### Scenario: Lock without active run

- **WHEN** no pipeline run is active
- **AND** user runs `wai pipeline lock`
- **THEN** the system errors: "No active pipeline run."

### Requirement: Artifact Verification

The CLI SHALL support verifying the integrity of locked pipeline artifacts by
recomputing SHA-256 hashes and comparing against stored lock metadata.

#### Scenario: Verify intact artifacts

- **WHEN** user runs `wai pipeline verify`
- **AND** all locked artifacts match their stored hashes (LF-normalized)
- **THEN** the system reports: "All N locked artifacts verified."

#### Scenario: Detect tampered artifact via verify

- **WHEN** user runs `wai pipeline verify`
- **AND** a locked artifact's LF-normalized content differs from its stored hash
- **THEN** the system reports the mismatch with file path and expected vs actual hash
- **AND** exits with a non-zero status code

#### Scenario: Verify during doctor

- **WHEN** user runs `wai doctor`
- **AND** pipeline lock files exist in the workspace
- **THEN** the doctor check includes artifact integrity verification
  (LF-normalized hash comparison)
- **AND** reports any mismatches as warnings

### Requirement: Pipeline Addenda

The CLI SHALL support addenda as the correction mechanism for locked pipeline
artifacts. An addendum is a regular wai artifact tagged with
`pipeline-addendum:<step-id>` and containing a `corrects` metadata reference
to the original artifact.

#### Scenario: Create addendum for locked step

- **WHEN** user runs `wai add research --corrects="<artifact-path>" "<content>"`
- **AND** the referenced artifact belongs to a locked pipeline step
- **THEN** the system tags the new artifact with `pipeline-addendum:<step-id>`
- **AND** records the `corrects` path in the artifact's frontmatter
- **AND** the addendum appears in `wai pipeline status` under the corrected step

#### Scenario: Addendum for unlocked step warns

- **WHEN** user runs `wai add research --corrects="<artifact-path>" "<content>"`
- **AND** the referenced artifact belongs to a step that is NOT locked
- **THEN** the system warns: "Step '<step-id>' is not locked — consider editing
  the original artifact directly."
- **AND** still creates the addendum if the user proceeds

#### Scenario: Addenda visible in status

- **WHEN** user runs `wai pipeline status`
- **AND** a step has associated addenda
- **THEN** the status output shows the addenda count and paths alongside the
  step's original artifacts

### Requirement: Coverage Gate

Pipeline steps SHALL support an optional coverage gate (`[steps.gate.coverage]`
with `require_input_manifest = true`) that requires the agent to produce a
coverage manifest before the step can advance. The coverage gate is a new tier
in `StepGate`, evaluated after procedural and before oracle gates.

A coverage manifest is a wai artifact of type `review` tagged with
`coverage-manifest:<step-id>`, listing each input artifact path (prior step
outputs + any addenda) with a one-line disposition: `addressed`, `deferred`,
or `N/A`.

#### Scenario: Coverage gate blocks advancement

- **WHEN** user runs `wai pipeline next`
- **AND** the current step has `require_input_manifest = true` in its
  `[steps.gate.coverage]` configuration
- **AND** no artifact tagged `coverage-manifest:<step-id>` exists for the
  current step
- **THEN** the system blocks advancement
- **AND** reports: "Coverage gate not satisfied. Create a coverage manifest
  (type: review, tag: coverage-manifest:<step-id>) listing all inputs
  addressed."

#### Scenario: Coverage gate passes

- **WHEN** user runs `wai pipeline next`
- **AND** the current step has `require_input_manifest = true`
- **AND** a `review` artifact tagged `coverage-manifest:<step-id>` exists
- **THEN** the coverage gate passes and advancement proceeds (subject to
  remaining gate tiers: oracle, approval)

## MODIFIED Requirements

### Requirement: Pipeline Run Lifecycle

The CLI SHALL support running a pipeline and tracking per-step completion.

#### Scenario: Start a pipeline run

- **WHEN** user runs `wai pipeline start issue-pipeline --topic="ant-forager"`
- **THEN** the system validates the pipeline definition (blocking on errors)
- **AND** generates a run ID (e.g., `issue-pipeline-2026-02-25-ant-forager`)
- **AND** stores initial run state in `.wai/pipeline-runs/<run-id>.yml`
- **AND** outputs the run ID and a hint to invoke the first step

#### Scenario: Step artifact tagged with run and step ID

- **WHEN** a skill runs in the context of a pipeline run
- **AND** the environment variable `WAI_PIPELINE_RUN=<run-id>` is set or `.wai/.pipeline-run` exists
- **AND** the skill calls any `wai add <type>` command (research, plan, design, review, or handoff)
- **THEN** the artifact is automatically tagged with `pipeline-run:<run-id>` and `pipeline-step:<step-id>`

#### Scenario: Advance pipeline to next step

- **WHEN** user runs `wai pipeline next`
- **AND** all configured gates for the current step pass (or no gates configured)
- **AND** if `lock = true`, all current step artifacts are successfully locked
- **THEN** the system marks the current step complete
- **AND** outputs the next step prompt (or completion message if last step)

#### Scenario: Advance blocked by gate failure

- **WHEN** user runs `wai pipeline next`
- **AND** a gate check fails (including coverage or approval gates)
- **THEN** the system does NOT advance
- **AND** outputs the failure reason and suggests corrective action

#### Scenario: Advance past last step rejected

- **WHEN** user runs `wai pipeline next`
- **AND** all steps are already marked complete
- **THEN** the system errors with a message indicating the pipeline run is finished
- **AND** suggests starting a new run with `wai pipeline start`

#### Scenario: Advance with unknown run ID

- **WHEN** user runs `wai pipeline next`
- **AND** no active run exists
- **THEN** the system errors with a clear message and lists available pipelines
