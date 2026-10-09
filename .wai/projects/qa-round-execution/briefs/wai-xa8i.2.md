# BRIEF: wai-xa8i.2 — sync: include resume-the-orchestration breadcrumb in sync output

## Why
Delegate to a subagent because this is a small, well-bounded render change
with existing test scaffolding — canon Invariant 1. A fresh session
implements the minimal breadcrumb without inheriting the lead's
orchestration history. (Orchestrator loop for umbrella wai-xa8i / openspec
change `resume-pipeline-adoption`; child 2 of 3.)

## Ticket
- ID: wai-xa8i.2 (beads, child of wai-xa8i)
- Type: code (Rust render + test), TDD red→green→refactor
- Spec: openspec/changes/resume-pipeline-adoption/specs/pipeline-resume/spec.md
  ("Sync includes orchestration breadcrumb" — scenario: Sync surfaces
  breadcrumb)

## Problem
The prime side of the umbrella shipped in wai-xa8i.1 (`→ Resume the
orchestration: wai pipeline next` on active mid-flight runs). `wai sync`
output still has no breadcrumb, so resume detection paths that see sync
output (e.g. session close flows) never surface the in-flight orchestration.

## Desired outcome (smallest correct version)
1. **RED (failing test first)** in `tests/sync_test.rs`: workspace fixture
   with an ACTIVE MID-FLIGHT pipeline run + a working projections config,
   then `wai sync` (real sync, not dry-run/status) stdout contains the
   canonical phrase **`resume the orchestration`** (case-insensitive).
   Fixture needs the equivalents of prime_test's `write_pipeline` +
   `write_active_run` helpers (they are test-local to prime_test.rs —
   copy the minimal versions; do NOT refactor shared helpers across test
   files in this ticket). Current code does NOT emit the phrase → test
   must fail.
2. **GREEN (minimal)**: in `src/commands/sync.rs`, in the successful real-sync
   path (right after `log::success("Agent configs synced")`, ~line 229),
   render one line when a pipeline run is active mid-flight:
   `→ Resume the orchestration: wai pipeline next`.
   Reuse the existing engine helpers — `pipeline_current_status` and
   `run_is_incomplete` are `pub` in `super::prime` (src/commands/prime.rs:548,
   prime.rs:749). A run is "active mid-flight" when
   `pipeline_current_status` returns Some(status) with `status.active == true`
   AND `status.step.is_some()` (the complete-run branch mirrors prime's
   rendering and is OUT OF SCOPE). Honor `quiet` like the surrounding code.
3. Existing sync tests (`sync_projects_inline_source_to_target_file` etc.)
   must stay green — additive change only.

## Out of scope
- Dry-run and status_only output paths; `--from-main` path; the
  complete-run ("wai close") branch; prime changes (wai-xa8i.1 done,
  wai-xa8i.3 separate); JSON payload changes; pipeline engine logic;
  refactoring fixture helpers into shared test utils.

## Exact files
- `src/commands/sync.rs` (breadcrumb render in the real-sync success path)
- `tests/sync_test.rs` (one new test + minimal fixture helpers)

## Exact commands
- `cargo test --test sync_test`
- `cargo test --test prime_test` (must stay green)
- `cargo test` (full suite green before commit; known flake
  gather_git_file_context / wai-of8s passes in isolation — re-run that
  single test if it flakes rather than debugging it)
- `cargo fmt --check`

## Red/green/refactor sequence
1. RED: `sync_active_run_includes_resume_the_orchestration` fails on
   current code.
2. GREEN: breadcrumb line in the real-sync success path; tests pass; full
   suite green.
3. REFACTOR: none expected; keep the change to ≤ ~12 lines.

## Follow-up threshold
If the breadcrumb requires touching anything beyond the two files above, or
the sync success path is unreachable in the test fixture, STOP and file
follow-ups under umbrella `wai-xa8i` instead of expanding scope.

## Completion criteria (runnable; exit 0 = done)
- `cargo test` full suite green (modulo wai-of8s flake in isolation)
- `cargo fmt --check`
- beads: `wai-xa8i.2` ready for orchestrator verify+ship (do NOT close it)

## Spawn model
- Model: same family (single-model environment; lead-side verification).
- Context ceiling: **≤ 50% of the actual model window at spawn**; fresh
  session. Fixed token counts are invalid; ceiling is a percentage of
  the live window.