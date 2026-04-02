## ADDED Requirements

### Requirement: Step-Level Artifact Tagging

Artifacts created during a pipeline run SHALL be tagged with both
`pipeline-run:<run-id>` and `pipeline-step:<step-id>` where `step-id` is the
ID of the currently active step in the run state.

#### Scenario: Artifact created during active step

- **WHEN** `wai add research "findings"` is called
- **AND** a pipeline run is active with run ID `sci-2026-04-02-qcd`
- **AND** the current step is `describe`
- **THEN** the artifact frontmatter includes:
  `tags: [pipeline-run:sci-2026-04-02-qcd, pipeline-step:describe]`

#### Scenario: No active pipeline run

- **WHEN** `wai add research "findings"` is called
- **AND** no pipeline run is active
- **THEN** no pipeline tags are injected (existing behavior unchanged)

#### Scenario: Retroactive tagging not supported

- **WHEN** `wai pipeline next` advances from step `decompose` to step `generate`
- **AND** the user then creates an artifact intended for step `decompose`
- **THEN** the artifact is tagged with `pipeline-step:generate` (the current step)
- **AND** the user must use `--tags pipeline-step:decompose` to override manually

---

### Requirement: Review Artifact Type

The CLI SHALL support a `review` artifact type created via `wai add review`.
Review artifacts SHALL have structured frontmatter with `reviews` (target
artifact filename, scoped to the current project), `verdict` (pass, fail, or
needs-work), and optional `severity` (object with critical, high, medium, low
counts) and `produced_by` (skill name) fields. The `--reviews` target SHALL be
validated against existing artifacts in the current project at creation time.

#### Scenario: Create a review artifact

- **WHEN** user runs `wai add review "Accuracy: all citations verified..." --reviews 2026-04-02-nlo-structure.md`
- **AND** `2026-04-02-nlo-structure.md` exists in the current project
- **THEN** the system creates a file in `.wai/projects/<project>/reviews/`
- **AND** the frontmatter includes `reviews: 2026-04-02-nlo-structure.md`

#### Scenario: Review with structured metadata

- **WHEN** user runs `wai add review "..." --reviews artifact.md --verdict pass --produced-by ro5 --severity critical:0,high:1,medium:3,low:2`
- **THEN** the frontmatter includes:
  ```yaml
  reviews: artifact.md
  verdict: pass
  produced_by: ro5
  severity: {critical: 0, high: 1, medium: 3, low: 2}
  ```
- **AND** `produced_by` is informational only (no automatic severity parsing)
- **AND** `--severity` accepts comma-separated `level:count` pairs; omitted
  levels default to 0

#### Scenario: Review without target rejected

- **WHEN** user runs `wai add review "..."` without `--reviews`
- **THEN** the system rejects with error: "review requires --reviews <artifact-filename>"

#### Scenario: Invalid verdict rejected

- **WHEN** user runs `wai add review "..." --reviews x.md --verdict excellent`
- **THEN** the system rejects with error listing valid values: pass, fail, needs-work

#### Scenario: Review target does not exist

- **WHEN** user runs `wai add review "..." --reviews nonexistent.md`
- **AND** no artifact named `nonexistent.md` exists in the current project
- **THEN** the system rejects with error: "artifact 'nonexistent.md' not found in project '<project>'"

---

### Requirement: Pipeline Gate Protocol

The CLI SHALL evaluate pipeline gates in strict order when `wai pipeline next`
is called on a step with gates configured. A gate is a validation check on a
pipeline step. Gates are organized into four tiers: structural (artifact
existence), procedural (review coverage), oracle (domain-specific checks), and
approval (human checkpoint). The first failing tier SHALL block advancement with
a descriptive reason. Steps without gates SHALL advance freely (existing behavior).

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

#### Scenario: Oracle gate blocks on script failure

- **WHEN** step declares an oracle named `dimensional-analysis`
- **AND** the oracle script exits with code 1 and stderr "Dimensional mismatch in eq. 14"
- **THEN** advancement is blocked with message: "Oracle 'dimensional-analysis' failed for 'findings.md': Dimensional mismatch in eq. 14"

#### Scenario: Oracle timeout

- **WHEN** an oracle declares `timeout = 30`
- **AND** the oracle script does not exit within 30 seconds
- **THEN** the oracle is killed and treated as a failure with message: "Oracle 'name' timed out after 30s"

#### Scenario: Approval gate blocks until approved

- **WHEN** step declares `[steps.gate.approval]` with `required = true`
- **AND** all other gates pass
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

---

### Requirement: Oracle Script Resolution

Oracle gates SHALL resolve scripts by name from `.wai/resources/oracles/`.
Resolution SHALL probe for `<name>`, `<name>.sh`, and `<name>.py` in order,
selecting the first match. Resolved scripts MUST be executable (shebang +
executable permission) and SHALL be invoked directly, not via an interpreter
prefix. An explicit `command` field in the oracle TOML SHALL override name
resolution and is executed as-is via the shell.

#### Scenario: Name-based resolution with executable script

- **WHEN** oracle declares `name = "dimensional-analysis"` without `command`
- **AND** `.wai/resources/oracles/dimensional-analysis.py` exists and is executable
- **THEN** the system executes `.wai/resources/oracles/dimensional-analysis.py <artifact-path>`

#### Scenario: Explicit command override

- **WHEN** oracle declares `name = "custom-check"` with `command = "python research/validate.py"`
- **THEN** the system executes `python research/validate.py <artifact-path>` via the shell
- **AND** name resolution is skipped

#### Scenario: Oracle not found

- **WHEN** oracle declares `name = "nonexistent"` without `command`
- **AND** no file matching the name exists in `.wai/resources/oracles/`
- **THEN** `wai pipeline validate` reports a warning
- **AND** the oracle gate fails at runtime with: "Oracle 'nonexistent' not found in .wai/resources/oracles/"

#### Scenario: Ambiguous oracle resolution

- **WHEN** oracle declares `name = "check"` without `command`
- **AND** both `.wai/resources/oracles/check.sh` and `.wai/resources/oracles/check.py` exist
- **THEN** the system selects the first match in probe order (`check.sh`)
- **AND** `wai pipeline validate` reports a warning: "oracle 'check' has multiple matches, using 'check.sh'"

#### Scenario: Oracle not executable

- **WHEN** oracle resolves to `.wai/resources/oracles/check.py`
- **AND** the file is not executable (missing execute permission)
- **THEN** the oracle gate fails with: "Oracle 'check' is not executable. Run: chmod +x .wai/resources/oracles/check.py"

#### Scenario: Cross-artifact oracle

- **WHEN** oracle declares `scope = "all"`
- **THEN** the system invokes the oracle once, passing all matching artifact
  paths as arguments: `<oracle> <path1> <path2> ... <pathN>`
- **AND** the oracle can read all artifacts for cross-artifact consistency
  checks (e.g., notation consistency, conservation laws)

#### Scenario: Default oracle scope is per-artifact

- **WHEN** oracle does not declare `scope`
- **THEN** the system invokes the oracle once per applicable artifact:
  `<oracle> <artifact-path>`

---

### Requirement: Pipeline Show Command

The CLI SHALL support `wai pipeline show <name>` displaying the pipeline
description, metadata, steps, and per-step gate configuration.

#### Scenario: Show pipeline with gates

- **WHEN** user runs `wai pipeline show scientific-research`
- **THEN** the system displays: pipeline name, description, metadata (when, skills),
  step list with gate summary per step (e.g., "structural + procedural + oracles + approval"),
  and oracle directory path

#### Scenario: Show nonexistent pipeline

- **WHEN** user runs `wai pipeline show nonexistent`
- **THEN** the system errors with message listing available pipelines

---

### Requirement: Pipeline Gates Command

The CLI SHALL support `wai pipeline gates [pipeline-name] [--step=<id>]` to
show gate requirements and live status. With an active run (and no arguments),
it shows live status for the current step. Without an active run, the pipeline
name is required and `--step` selects the step to display.

#### Scenario: Gates with active run

- **WHEN** a pipeline run is active at step `generate-validate-accrue`
- **AND** user runs `wai pipeline gates`
- **THEN** the system displays each gate tier with pass/fail/blocked status
  and counts (e.g., "✗ min 1 research artifact — found 0")

#### Scenario: Gates without active run

- **WHEN** no pipeline run is active
- **AND** user runs `wai pipeline gates scientific-research --step=generate-validate-accrue`
- **THEN** the system displays gate definitions (not live status) for that step

#### Scenario: Gates on step without gates

- **WHEN** a pipeline run is active at a step with no gate configuration
- **AND** user runs `wai pipeline gates`
- **THEN** the system displays: "No gates configured for step '<step-id>'."

#### Scenario: Gates without active run and no pipeline name

- **WHEN** no pipeline run is active
- **AND** user runs `wai pipeline gates` without arguments
- **THEN** the system errors: "No active pipeline run. Specify a pipeline name: wai pipeline gates <name>"

---

### Requirement: Pipeline Check Command

The CLI SHALL support `wai pipeline check` to evaluate all gates for the current
step without advancing. An optional `--oracle=<name>` flag SHALL run a single
oracle against all applicable artifacts. Output SHALL include per-tier status
with pass/fail/blocked indicators and a summary result.

#### Scenario: Check all gates

- **WHEN** user runs `wai pipeline check`
- **THEN** the system evaluates structural, procedural, oracle, and approval gates
- **AND** reports pass/fail/blocked per tier with details (counts, reasons)
- **AND** reports a summary: "Result: PASS" or "Result: BLOCKED — resolve N failures"
- **AND** does NOT advance the step

#### Scenario: Check on step without gates

- **WHEN** user runs `wai pipeline check`
- **AND** the current step has no gate configuration
- **THEN** the system reports: "No gates configured for step '<step-id>'. Result: PASS"

#### Scenario: Check single oracle

- **WHEN** user runs `wai pipeline check --oracle=dimensional-analysis`
- **THEN** the system runs only that oracle against all applicable artifacts
- **AND** reports per-artifact pass/fail with stderr output on failure

---

### Requirement: Pipeline Approve Command

The CLI SHALL support `wai pipeline approve` to record human approval for the
current step. Approval SHALL be stored as a timestamp in the run state YAML.
Approval is invalidated if any artifact tagged with the current step has a
creation timestamp later than the approval timestamp.

#### Scenario: Approve current step

- **WHEN** a pipeline run is active
- **AND** user runs `wai pipeline approve`
- **THEN** the run state records an approval timestamp for the current step
- **AND** the system confirms: "Approved step 'step-id'. Run 'wai pipeline next' to advance."

#### Scenario: Approve without active run

- **WHEN** no pipeline run is active
- **AND** user runs `wai pipeline approve`
- **THEN** the system errors: "No active pipeline run."

---

### Requirement: Pipeline Validate Command

The CLI SHALL support `wai pipeline validate [name]` to check pipeline TOML
definitions for correctness. Validation SHALL also run during `wai doctor`
and before `wai pipeline start`.

#### Scenario: Validate a valid pipeline

- **WHEN** user runs `wai pipeline validate scientific-research`
- **AND** the TOML is well-formed with valid gates and existing oracles
- **THEN** the system reports success with step count and gate summary

#### Scenario: Validate catches missing metadata

- **WHEN** a pipeline lacks `[pipeline.metadata]`
- **THEN** validation reports warning: "missing [pipeline.metadata] — pipeline won't appear in managed block"

#### Scenario: Validate catches missing oracle

- **WHEN** a pipeline references oracle `check-dims` and no matching file exists
- **THEN** validation reports warning: "gate oracle 'check-dims' — command not found"

#### Scenario: Validate catches non-executable oracle

- **WHEN** a pipeline references oracle `check-dims` and the file exists but is not executable
- **THEN** validation reports warning: "gate oracle 'check-dims' — not executable"

#### Scenario: Validate catches duplicate names

- **WHEN** two TOML files declare the same `pipeline.name`
- **THEN** validation reports error: "duplicate pipeline name"

#### Scenario: Start blocked by validation errors

- **WHEN** user runs `wai pipeline start broken-pipeline --topic="test"`
- **AND** the pipeline has validation errors (not warnings)
- **THEN** the system refuses to start and shows the errors

---

### Requirement: Pipeline Metadata

Pipeline TOML definitions SHALL support an optional `[pipeline.metadata]`
section with `when` (string describing when to suggest the pipeline) and
`skills` (list of skill names the pipeline depends on).

#### Scenario: Metadata in TOML

- **WHEN** a pipeline defines:
  ```toml
  [pipeline.metadata]
  when = "Frontier-level research requiring systematic validation"
  skills = ["design-practice", "ro5"]
  ```
- **THEN** `wai pipeline show` displays the metadata
- **AND** `wai pipeline validate` checks that referenced skills are installed

#### Scenario: Missing metadata

- **WHEN** a pipeline omits `[pipeline.metadata]`
- **THEN** the pipeline functions normally but `wai pipeline validate` warns
  that it won't appear in the managed block

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
- **THEN** the system marks the current step complete
- **AND** outputs the next step prompt (or completion message if last step)

#### Scenario: Advance blocked by gate failure

- **WHEN** user runs `wai pipeline next`
- **AND** a gate check fails
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
