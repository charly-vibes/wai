# BRIEF: wai-lqdr — Bundle tests-pass oracle with epic-orchestrator init

## Why
Delegate to a subagent because this is a small, mechanical feature fix
with an established in-repo pattern to mirror (tdd-ro5's oracle bundling)
— canon Invariant 1. The lead has a fresh example of the failure
documented in docs; an independent session implements the mirror without
re-deriving the pattern.

## Ticket
- ID: wai-lqdr (beads)
- Type: code (Rust + template asset + test), TDD red→green

## Problem
The epic-orchestrator template's verify and ship steps declare
`[[steps.gate.oracles]] name = "tests-pass"` gates, but
`wai pipeline init epic-orchestrator` does not scaffold
`.wai/resources/oracles/tests-pass.sh`. Fresh scaffolds block at
verify/ship until the user hand-writes the oracle.

## Pattern to mirror (proven)
- `src/commands/pipeline/setup.rs:145` — `bundled_oracles_for(name)`
  currently maps only `"tdd-ro5"` → `release-docs-fresh.sh` via
  `include_str!("../../templates/oracles/release-docs-fresh.sh")`.
- Test: `tests/suite_pipeline_init_tdd_ro5_ships_release_oracle_script.rs`.

## Desired outcome
1. RED: new test (mirror the tdd-ro5 suite's structure) asserting
   `wai pipeline init epic-orchestrator` scaffolds an executable
   `.wai/resources/oracles/tests-pass.sh`. Fails first.
2. GREEN:
   - Add `src/templates/oracles/tests-pass.sh` — a generic,
     repo-agnostic evidence gate (base it on the deployed working
     script at `.wai/resources/oracles/tests-pass.sh` in this repo:
     passes when the artifact contains explicit verification language
     like `command=`, `commands run`, `verified:`, `tests pass`,
     `validation run`, `non-code`, `no code changes`; fails with a
     stderr message otherwise). It must not reference repo-specific
     paths.
   - Extend `bundled_oracles_for` with
     `"epic-orchestrator" => vec![("tests-pass.sh", include_str!(...))]`.
3. Both suites stay green; docs claim in
   `docs/src/concepts/pipelines.md` ("tests-pass oracle requirement" in
   the epic-orchestrator section) is now true out of the box — update
   the docs line only if it explicitly says the user must hand-write
   it.

## Out of scope
- Template TOML changes; oracle gate semantics changes; tdd-ro5 behavior.

## Exact files (likely)
- `src/templates/oracles/tests-pass.sh` (new)
- `src/commands/pipeline/setup.rs`
- `tests/suite_pipeline_epic_orchestrator_ships_tests_pass_oracle.rs`
  (new, mirroring the tdd-ro5 oracle suite) — or extend an existing
  epic-orchestrator suite if that fits conventions better
- `docs/src/concepts/pipelines.md` (only if needed)

## Exact commands
- `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script`
- `cargo test` (full suite green)
- `cargo fmt --check`

## Follow-up threshold
If the oracle gate semantics need redesign → stop and file a follow-up
under epic `wai-fvhv`.

## Completion criteria (runnable; exit 0 = done)
- `cargo test` full suite green including the new test
- `cargo fmt --check`

## Spawn model
- Model: same family (single-model environment; lead-side verification).
- Context ceiling: **≤ 50% of the actual model window at spawn**; small
  mirrored fix, fresh session. Fixed token counts are invalid; ceiling
  is a percentage of the live window.