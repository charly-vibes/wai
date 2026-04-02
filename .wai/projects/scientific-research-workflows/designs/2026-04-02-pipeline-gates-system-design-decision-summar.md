Pipeline Gates System — Design Decision

## Summary
Extend wai's pipeline system with a 4-tier gate protocol that enforces validation
at step boundaries, plus supporting commands for discoverability and debugging.

## Core Design: 4-Tier Gate Protocol

Gates evaluate in order when `wai pipeline next` is called. First failure blocks advancement.

### Tier 1: Structural Gates
- Purpose: verify a step produced expected outputs
- Mechanism: count artifacts tagged `pipeline-step:<step-id>` + `pipeline-run:<run-id>`
- Prerequisite: step-level artifact tagging (currently only run-level exists)
- TOML:
  [steps.gate.structural]
  min_artifacts = 1
  types = ["research"]

### Tier 2: Procedural Gates
- Purpose: verify validation process was followed
- Mechanism: new `review` artifact type with `reviews:` relationship field and `verdict`/`severity` frontmatter
- Gate checks: review exists for each artifact, verdict passes, severity within thresholds
- TOML:
  [steps.gate.procedural]
  require_review = true
  review_verdict = "pass"
  max_critical = 0
  max_high = 0

### Tier 3a: Oracle Gates
- Purpose: domain-specific machine-verifiable checks
- Mechanism: user-written scripts in .wai/resources/oracles/, exit 0 = pass
- Name-based resolution: name -> .wai/resources/oracles/<name>[.sh|.py]
- Explicit path override via `command` field
- Timeout: default 30s, configurable per oracle
- TOML:
  [[steps.gate.oracles]]
  name = "dimensional-analysis"
  description = "Verify dimensional consistency"
  timeout = 300

### Tier 3b: Approval Gates
- Purpose: forced human checkpoint ("hammock time")
- Mechanism: `wai pipeline approve` sets timestamp in run state YAML
- Per-step, not per-artifact
- TOML:
  [steps.gate.approval]
  required = true
  message = "Review all accrued artifacts before advancing"

## Pipeline Enforcement Model
- Pipelines stay strictly linear — no loops, no branching
- Parallelism lives within steps, managed by beads issue dependencies
- Gates enforce phase ordering and validation completion
- LLM compliance is structural (must produce artifacts) not behavioral (can't verify rigor)
- Skill attestation via `produced_by` frontmatter raises the bar for fake reviews

## Oracle Script Location
- Default: .wai/resources/oracles/<name>[.sh|.py]
- Escape hatch: explicit `command` path in TOML
- Contract: receives artifact path as arg, exit 0 = pass, stderr = failure reason
- Scaffolded by `wai pipeline init`

## New Commands
- `wai pipeline show <name>` — detailed view with gates per step
- `wai pipeline gates [--step=<id>]` — gate requirements for current/specific step
- `wai pipeline check [--oracle=<name>]` — dry-run gates without advancing
- `wai pipeline approve` — human approval for current step
- `wai pipeline validate [name]` — validate pipeline TOML + gate config

## Validation (wai pipeline validate + wai doctor)
- Runs on: standalone, during wai doctor, before wai pipeline start
- Checks: TOML syntax, required fields, unique step IDs, metadata presence,
  skill references, oracle command existence, gate config validity
- Managed block staleness: doctor compares generated vs actual WAI:START block,
  warns if outdated (pipeline added/removed but block not refreshed)

## Managed Block Integration
- Pipeline TOML gets [pipeline.metadata] with `when` and `skills` fields
- wai_block_content() reads installed pipelines, generates 'Available Pipelines' table
- wai init --update refreshes the block
- Both CLAUDE.md and AGENTS.md get identical blocks

## Key Design Decisions
1. Pipeline stays linear — complexity belongs in beads (parallelism) and oracles (domain logic)
2. Gates are declarative TOML, not imperative scripts (except oracles)
3. Oracle name resolution with explicit path escape hatch
4. Review is a first-class artifact type, not a tag convention
5. Approval is per-step, not per-artifact
6. Validation runs at multiple points (author time, start time, advance time)
7. Managed block auto-generates from pipeline metadata for discoverability
