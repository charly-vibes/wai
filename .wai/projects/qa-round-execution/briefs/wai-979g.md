# BRIEF: wai-979g — Isolation-based verify step (named read-only verifier)

## Why
Delegate to a subagent because this is a self-contained, test-first
template edit confined to two files with an existing suite to extend —
the exact shape the orchestrator pattern delegates (canon Invariant 1).
The lead's context is better spent on verification and shipping; a fresh
session executes the red→green loop without dragging session history.

## Ticket
- ID: wai-979g (beads; epic-orchestrator co-modification thread)
- Type: code (template + test), TDD red→green→refactor

## Desired outcome
The `verify` step in `src/templates/epic-orchestrator.toml` encodes an
isolation-based verification pass:

1. The verify step spawns a **read-only verifier** as a separate NAMED
   session: `subagent:<ticket>:verify` — read-only tools, cannot spawn
   further subagents, and a **different model family when configured**.
2. The verifier cross-checks the implementor's report: listed commits vs
   `git log` on the ticket's branch/worktree, and vs ticket-tracker
   (beads) state.
3. **Contradiction fails the run** — no advance to ship.
4. **Single-model degradation**: when no second model family is
   configured, the lead-side checks path applies (the existing manual
   git/beads checks) and is **non-blocking**.

## Out of scope
- Brief step block (wai-3eo0, landed — do not modify).
- State-durability step blocks (wai-wvjz, landed — do not modify).
- Docs / strict validation / integration test (wai-hr0w).
- Shared template convention extraction (wai-qjcz).
- Rust engine changes — template prompts only.

## Exact files
- `src/templates/epic-orchestrator.toml` — verify step prompt only.
- `tests/suite_pipeline_init_epic_orchestrator_template.rs` — append one
  new test; do not modify existing tests.

## Exact commands
- Red/green: `cargo test --test suite_pipeline_init_epic_orchestrator_template`
- Regression: `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script`
- Validate scaffold: `wai pipeline validate epic-orchestrator` (exit 0)
- Tidy gate: `cargo fmt --check`

## Red/green/refactor sequence
1. RED: add `pipeline_init_epic_orchestrator_encodes_isolation_verify`
   asserting the scaffolded TOML contains: `subagent:<ticket>:verify`
   named-session convention, read-only verifier tooling restriction,
   cross-check of listed commits vs git log and vs beads state,
   contradiction-blocks-ship rule, and the single-model degradation
   (lead-side checks, non-blocking). Confirm failure is the missing
   behavior.
2. GREEN: minimally edit the verify step prompt until tests pass and
   validate exits 0.
3. REFACTOR: wording tidy only.

## Follow-up threshold
Rust engine changes or scope beyond the two files → stop and file a
follow-up child under epic `wai-fvhv`.

## Completion criteria (runnable; exit 0 = done)
- `cargo test --test suite_pipeline_init_epic_orchestrator_template`
- `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script`
- `wai pipeline validate epic-orchestrator`
- `cargo fmt --check`

## Spawn model
- Model: same family as the orchestrator (default pi model) — no second
  family is configured in this environment; this is the documented
  single-model degradation case.
- Context ceiling: **≤ 50% of the actual model window at spawn** — small
  two-file change, brief ~1.5k tokens, fresh session starts near 0%.
  Fixed token counts are invalid; ceiling is a percentage of the live
  window.