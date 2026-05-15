# Change: Add decision artifact freshness tracking

## Why

Wai artifacts capture the *why* behind decisions, but today they have no way to
signal when the code they describe has changed. A design artifact written six
months ago may still be referenced by `wai status` even after its tracked files
have been refactored beyond recognition. The gap makes `wai why` progressively
less trustworthy as the codebase evolves.

Adjacent patterns already exist in the codebase:

- `doctor` compares generated vs. actual content using SHA-256 sidecars.
- `pipeline` writes per-artifact `.lock` sidecars with hashed file snapshots.
- `status` surfaces actionable next-step signals to agents and humans.

This change wires those patterns together to close the feedback loop: a decision
artifact can declare which repo paths it describes; wai detects when those paths
change and surfaces the artifact as stale; humans or agents re-evaluate it and
refresh the sidecar.

The goal is the smallest design that proves the loop end to end. Semantic
correctness analysis, git-hook automation, and decision-point rollups are
explicitly deferred to later phases (see `docs/feedback-loop-design.md`).

## What Changes

- Extend artifact frontmatter with optional `tracks` (path/glob list) and
  `decision_point` (slug) fields.
- Persist per-artifact freshness sidecars (`<artifact>.fresh.lock`) using the
  mtime + SHA-256 pattern already used by pipeline locks.
- Add a `wai artifacts stale` command that emits machine-readable stale-artifact
  reports (JSON and human-readable).
- Surface stale-artifact warnings in `wai status` as actionable next-step
  suggestions.
- Extend `wai add` to accept `--tracks <path>[,<path>...]` when creating
  research/design/plan artifacts.

## Impact

- Affected specs: `cli-core` (new subcommand), `research-management` (frontmatter
  extension), `context-suggestions` (status surfacing), new `decision-freshness`
  capability spec.
- Affected code: `src/commands/add.rs`, new `src/commands/artifacts.rs`,
  `src/commands/status.rs`, new `src/freshness.rs` sidecar helpers.
- No breaking changes to existing frontmatter; `tracks` is optional and ignored
  when absent.
