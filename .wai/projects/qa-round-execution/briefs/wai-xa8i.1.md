# BRIEF: wai-xa8i.1 — prime: surface resume-the-orchestration when a pipeline run is active

## Why
Delegate to a subagent because this is a small, well-bounded render change
with existing test scaffolding — canon Invariant 1. A fresh session
implements the minimal directive without inheriting the lead's orchestration
history. (Orchestrator loop for umbrella wai-xa8i / openspec change
`resume-pipeline-adoption`.)

## Ticket
- ID: wai-xa8i.1 (beads, child of wai-xa8i)
- Type: code (Rust render + test), TDD red→green→refactor
- Spec: openspec/changes/resume-pipeline-adoption/specs/pipeline-resume/spec.md
  ("Prime surfaces resume-the-orchestration" — scenario: Active pipeline run)

## Problem
Session eval (Oct 7–8) showed agents were hand-nudged to resume
orchestration even though prime renders a `PIPELINE RUN ADOPT/RESUME` line
(wai-vx02.1). The line is a status readout, not a directive — nothing in the
output says "continue the orchestration without asking".

## Desired outcome (smallest correct version)
1. **RED (failing test first)** in `tests/prime_test.rs`: with an active
   pipeline run (existing helpers `write_pipeline` + `write_active_run`),
   `wai prime` stdout contains the canonical directive phrase
   **`resume the orchestration`** (case-insensitive match on the phrase
   `resume the orchestration` is acceptable; assert against the exact
   rendered line chosen in GREEN). Current code does NOT emit this phrase →
   test must fail.
2. **GREEN (minimal)**: in `render_active_pipeline_run`
   (`src/commands/prime.rs:~742`), add one line, e.g.
   `→ Resume the orchestration: wai pipeline next` (keep the existing
   ADOPT/RESUME block intact). Only the active/mid-flight branch (`Some(step)`)
   needs it — the complete-run branch is out of scope.
3. Existing tests (`prime_adopts_active_run*`) must stay green — additive
   change only.

## Out of scope
- The complete-run ("wai close") branch; `wai sync` breadcrumb (ticket
  wai-xa8i.2); epics-flow precedence (ticket wai-xa8i.3); JSON payload
  changes; gate semantics; any pipeline engine logic.

## Exact files
- `src/commands/prime.rs` (render_active_pipeline_run only)
- `tests/prime_test.rs` (one new test, next to prime_adopts_active_run tests)

## Exact commands
- `cargo test --test prime_test`
- `cargo test` (full suite green before commit)
- `cargo fmt --check`

## Red/green/refactor sequence
1. RED: new test `prime_active_run_directives_resume_the_orchestration`
   fails on current code.
2. GREEN: add the directive line; test passes; full suite green.
3. REFACTOR: none expected; keep the change to ≤ ~5 lines.

## Follow-up threshold
If the phrase placement conflicts with existing tests or requires touching
anything beyond the two files above, STOP and file follow-ups under umbrella
`wai-xa8i` instead of expanding scope.

## Completion criteria (runnable; exit 0 = done)
- `cargo test` full suite green including the new RED-derived test
- `cargo fmt --check`
- beads: `wai-xa8i.1` closed (or ready for orchestrator verify+ship)

## Spawn model
- Model: same family (single-model environment; lead-side verification).
- Context ceiling: **≤ 50% of the actual model window at spawn**; fresh
  session. Fixed token counts are invalid; ceiling is a percentage of
  the live window.