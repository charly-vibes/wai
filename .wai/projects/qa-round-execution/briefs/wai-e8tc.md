# BRIEF: wai-e8tc — Fix: `wai add` tags pipeline artifacts with the previous step

## Why
Delegate to a subagent because this is a well-scoped bug fix requiring
empirical reproduction and a focused code change — canon Invariant 1. The
lead observed the symptom four times across orchestrator runs; an
independent session reproduces it cleanly with a fixture and root-causes
without inheriting the lead's assumptions.

## Ticket
- ID: wai-e8tc (beads, BUG)
- Type: code (Rust), TDD red→green→refactor

## Symptom (observed 4×, reproducible)
After `wai pipeline next` advances the run, the first `wai add <type>`
writes frontmatter tagged with the PREVIOUS step id:
- Evidence: `.wai/projects/qa-round-execution/research/2026-10-08-gates-*hr0w*`
  tagged `pipeline-step:claim` while the run yml said `current_step: 1`.
- Consequence: the new current step's structural gate finds 0 artifacts
  and blocks `wai pipeline next` until manual retagging.
- NOTE: in at least one case the artifact may have been added BEFORE the
  advancing `wai pipeline next` (operator sequencing) — the reproduction
  MUST distinguish "add before next" (correct behavior, not a bug) from
  "add after next shows the new step prompt, artifact still tagged with
  old step" (the bug).

## Relevant code (starting points, verify yourself)
- `src/commands/add.rs` — `build_tags()`, `resolve_current_step_id()`:
  reads `.wai/pipeline-runs/<run>.yml` `current_step` and indexes
  `definition.steps[current_step]`.
- `src/commands/pipeline/orchestration.rs` — `wai pipeline next` write
  ordering: WHEN is the yml rewritten relative to printing the new step
  prompt?
- Hypotheses to test (confirm or refute empirically):
  (a) yml written late / add reads stale yml;
  (b) off-by-one or 0- vs 1-based index mismatch between yml and steps;
  (c) `.last-run` pointer stale relative to yml.

## Desired outcome
- RED: an integration/fixture test that reproduces the lag deterministically
  (temp workspace, `pipeline init` + `start` a built-in template, `next`,
  then `add`, assert the artifact tag equals the CURRENT step).
- GREEN: minimal fix so `wai add` always resolves the same current step
  that `wai pipeline current` reports.
- Regression: existing suites untouched and green.

## Out of scope
- Template file changes; gate semantics changes; retagging migration of
  historical artifacts.

## Exact files (likely)
- `src/commands/add.rs` (and/or `src/commands/pipeline/orchestration.rs`)
- new test in `tests/suite_pipeline_epic_orchestrator_ticket_state.rs`
  or a dedicated suite file

## Exact commands
- `cargo test` (full suite must stay green)
- `cargo fmt --check`

## Follow-up threshold
If the fix requires changing gate/artifact association semantics beyond
tag resolution → stop and file a follow-up under epic `wai-fvhv`.

## Completion criteria (runnable; exit 0 = done)
- `cargo test` full suite green, including the new RED→GREEN test
- `cargo fmt --check`

## Spawn model
- Model: same family (single-model environment; lead-side verification).
- Context ceiling: **≤ 50% of the actual model window at spawn**; fresh
  session, focused bug fix. Fixed token counts are invalid; ceiling is a
  percentage of the live window.