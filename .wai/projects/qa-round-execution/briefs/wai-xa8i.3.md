# BRIEF: wai-xa8i.3 — prime: pipeline-run suggestion is authoritative over epics/bd-ready guidance

## Why
Delegate to a subagent because this is a small, well-bounded precedence fix
with existing test scaffolding (fake-bd helpers) — canon Invariant 1. A
fresh session implements the minimal gating without inheriting the lead's
orchestration history. (Orchestrator loop for umbrella wai-xa8i / openspec
change `resume-pipeline-adoption`; child 3 of 3, final.)

## Ticket
- ID: wai-xa8i.3 (beads, child of wai-xa8i; blocked-by wai-xa8i.1 — shipped)
- Type: code (Rust render/payload gating + test), TDD red→green→refactor
- Spec: openspec/changes/resume-pipeline-adoption/specs/pipeline-resume/spec.md
  ("Prime suggestion is authoritative over epics flow" — scenario: Pipeline
  run wins over epics flow)

## Problem
When a pipeline run is active, prime renders the resume-the-orchestration
directive (wai-xa8i.1) but ALSO still emits the older epics-flow suggestion:
terminal `→ Suggested next: bd show <id>` (render_closing_sections,
src/commands/prime.rs:108) and JSON payload `next_steps` populated from
`suggested_next` (src/commands/prime.rs:~306). Session eval showed agents
could not tell which system was authoritative and asked instead of
continuing.

## Desired outcome (smallest correct version)
1. **RED (failing test first)** in `tests/prime_test.rs`: fixture with an
   active MID-FLIGHT pipeline run (`write_pipeline` + `write_active_run`,
   already test-local in prime_test.rs) AND a fake `bd` whose
   `bd ready --json` emits one issue (use the existing helper
   `install_fake_bd_ready_json` from `tests/common/mod.rs:615` — check its
   exact signature and how other tests prepend the fake-bin dir to PATH).
   Run `wai prime`; assert stdout does NOT contain `Suggested next:`.
   Pre-change, prime prints both the resume directive and
   `→ Suggested next: bd show <id>` → RED fails.
2. **GREEN (minimal)**: gate the epics-flow suggestion on "no active
   mid-flight pipeline run":
   - Terminal: in `render_closing_sections` (prime.rs:~108), skip the
     `suggested_next` block when
     `pipeline_current_status(project_root)` returns Some(status) with
     `status.active` AND `run_is_incomplete(&status)` (same helpers the
     sync breadcrumb in src/commands/sync.rs uses — definitions live in
     `super::pipeline`/orchestration.rs; prime already imports both at
     prime.rs:18).
   - JSON: same condition gates the `next_steps` fallback from
     `suggested_next` (prime.rs:~306) so the payload agrees with the
     terminal render.
   Keep the complete-run (no step) case OUT of scope — mirrors wai-xa8i.1.
3. Existing tests stay green, including `prime_adopts_active_run*` and any
   test asserting `Suggested next:` appears when NO run is active (grep for
   `Suggested next` in tests/prime_test.rs first; if such a test exists it
   must still pass — it has no active run, so gating must not fire).

## Out of scope
- The complete-run branch; sync changes (wai-xa8i.2 done); JSON payload
  structure beyond the next_steps gating; bd invocation changes; any
  pipeline engine logic; refactoring fixture helpers.

## Exact files
- `src/commands/prime.rs` (render_closing_sections + payload next_steps)
- `tests/prime_test.rs` (one new test using the fake-bd helper)

## Exact commands
- `cargo test --test prime_test`
- `cargo test --test sync_test` (wai-xa8i.2 must stay green)
- `cargo test` (full suite green before commit; known flake
  gather_git_file_context / wai-of8s passes in isolation — re-run that
  single test if it flakes rather than debugging it)
- `cargo fmt --check`

## Red/green/refactor sequence
1. RED: `prime_active_run_suppresses_suggested_next` fails on current code.
2. GREEN: gate both call sites; tests pass; full suite green.
3. REFACTOR: none expected; keep the change to ≤ ~15 lines.

## Follow-up threshold
If the fake-bd helper can't make `suggested_next` fire in the fixture (e.g.
PATH isolation issues), or the change grows beyond the two files above, STOP
and file follow-ups under umbrella `wai-xa8i` instead of expanding scope.

## Completion criteria (runnable; exit 0 = done)
- `cargo test` full suite green (modulo wai-of8s flake in isolation)
- `cargo fmt --check`
- beads: `wai-xa8i.3` ready for orchestrator verify+ship (do NOT close it)

## Spawn model
- Model: same family (single-model environment; lead-side verification).
- Context ceiling: **≤ 50% of the actual model window at spawn**; fresh
  session. Fixed token counts are invalid; ceiling is a percentage of
  the live window.