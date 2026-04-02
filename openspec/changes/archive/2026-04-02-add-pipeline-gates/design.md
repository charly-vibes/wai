## Context
Wai pipelines are linear, sequential step definitions. Steps are navigation hints
that reference skills. The pipeline system currently has no enforcement — agents
can advance freely regardless of what work was done. This change adds a gate
protocol that blocks advancement until validation criteria are met.

The design was informed by:
- The "vibe physics" experiment's failure modes (sycophancy, data smoothing,
  context drift)
- The Ralph Loop pattern's insight that verification should check disk state,
  not agent self-assessment
- The principle that pipelines should stay linear and simple — parallelism
  belongs in beads, domain logic belongs in oracles

## Goals / Non-Goals
- Goals:
  - Block pipeline advancement when work is incomplete or unvalidated
  - Support domain-specific machine-verifiable checks via oracles
  - Force human checkpoints where machine verification is insufficient
  - Make installed pipelines and their gate requirements discoverable to agents
  - Validate pipeline definitions at author time, start time, and doctor time
- Non-Goals:
  - Loops, branching, or conditional steps in pipelines
  - Parallel step execution (parallelism is within steps via beads)
  - Guaranteeing the LLM followed instructions rigorously (gates are structural
    enforcement, not behavioral enforcement)
  - Replacing human judgment for semantic correctness

## Decisions

### 4-tier gate protocol
Gates evaluate in strict order: structural → procedural → oracle → approval.
Each tier runs only if all previous tiers passed. First failure blocks advancement.

- **Structural**: count artifacts tagged with the current step and run. Catches
  skipped steps.
- **Procedural**: verify a review artifact exists for each generated artifact and
  meets severity thresholds. Catches skipped validation.
- **Oracle**: run user-defined scripts against artifacts. Catches domain-specific
  errors.
- **Approval**: require explicit `wai pipeline approve` before advancing. Catches
  everything else via human judgment.

Alternative considered: single gate type (shell command). Rejected because it
forces every pipeline author to reinvent structural and procedural checks.
The tiered approach provides built-in coverage with oracles as the extension point.

### Review as a first-class artifact type
New `wai add review` command that creates artifacts with structured frontmatter:
`reviews` (target artifact filename), `verdict` (pass/fail/needs-work),
`severity` (counts per level), `produced_by` (skill name, optional).

Alternative considered: tag conventions on research artifacts. Rejected because
tags are unstructured and can't be validated programmatically. A dedicated type
enables the procedural gate to inspect verdict and severity fields reliably.

### Oracle script resolution
Oracles declared by `name` in TOML resolve to `.wai/resources/oracles/<name>`
with extension probing (.sh, .py, no extension). The `command` field overrides
resolution for scripts that live elsewhere.

Contract: exit 0 = pass, non-zero = fail, stderr = reason. Default timeout
30s, configurable per oracle. Two invocation scopes: per-artifact (default,
`<oracle> <artifact-path>`) and cross-artifact (`scope = "all"`,
`<oracle> <path1> <path2> ...`).

### Pipeline metadata for discoverability
`[pipeline.metadata]` section in TOML with `when` (trigger description) and
`skills` (required skills). The managed block generator reads this to produce
the "Available Pipelines" table. Missing metadata triggers a warning in
`wai pipeline validate` and `wai doctor`.

### Managed block staleness detection
`wai doctor` generates what the block should contain and diffs against actual
CLAUDE.md / AGENTS.md content. Warns if stale, suggests `wai init --update`.

## Migration Plan
N/A — non-breaking change. Existing pipelines without gates continue to work
as before. Gates are opt-in per step.

## Risks / Trade-offs
- **Risk**: Overly strict gates frustrate users → gates are opt-in per step,
  pipelines without gates work as before
- **Risk**: Oracles that hang → configurable timeout with 30s default
- **Risk**: Review artifacts add noise → only created when procedural gates
  are configured
- **Trade-off**: Structural gates check artifact existence, not quality. Accepted
  because quality verification belongs in oracles (machine) or approval (human).

## Resolved Questions
- **Should `wai pipeline approve` support a message/reason field?**
  Not in v1. The approval timestamp + artifact count snapshot is sufficient.
  A message field can be added later if users request it.
- **Should oracle results be persisted as artifacts for auditability?**
  Not in v1. Oracle stdout/stderr is displayed to the user but not stored.
  The review artifact (from the procedural gate) already provides the audit
  trail. Oracle persistence can be added later as an opt-in.
- **Should `wai pipeline check` output be structured (JSON)?**
  Not in v1. Human-readable output with pass/fail/blocked indicators and a
  summary line is sufficient. A `--json` flag can be added later for agent
  consumption if needed.
