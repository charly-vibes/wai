# Change: Add artifact integrity and audit gates to pipelines

## Why

Pipeline artifacts (research, plans, designs) produced during runs have no
integrity guarantees. An agent can silently rewrite earlier step outputs
without detection — undermining auditability and making scientific-research
workflows unreliable. Inspired by recursive-mode.dev's artifact locking and
audit gate system, this change adds SHA-256 locking and formal coverage/approval
gates to pipeline step outputs.

See: `.wai/projects/scientific-research-workflows/research/2026-04-14-*.md`

## What Changes

- **Artifact locking**: `wai pipeline lock` computes a SHA-256 hash of the
  current step's artifacts, writes a `.lock` sidecar, and marks the step as
  locked in run state. Locked artifacts cannot be edited — only corrected via
  addenda.
- **Artifact verification**: `wai pipeline verify` recomputes hashes and
  reports any integrity violations. Also runs during `wai doctor`.
- **Addenda**: when a later step discovers gaps in a locked step's output,
  the agent creates an addendum artifact (tagged with the original step)
  rather than editing the locked file. Downstream steps must reference
  addenda as inputs.
- **Coverage gate**: new gate tier (slots into existing `StepGate` between
  procedural and oracle) that requires the agent to produce a coverage
  manifest demonstrating all inputs (prior step artifacts + addenda) were
  addressed before advancement.
- **Gate enforcement on `wai pipeline next`**: if coverage gate is
  configured and not satisfied, advancement is blocked. Works alongside
  existing structural, procedural, oracle, and approval gates.

## Impact

- Affected specs: `pipeline-resource`
- Affected code: `src/commands/pipeline.rs`, pipeline run state YAML,
  `src/commands/doctor.rs` (verify integration)
- Non-breaking: all new features are opt-in per pipeline definition
