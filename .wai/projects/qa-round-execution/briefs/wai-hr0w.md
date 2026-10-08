# BRIEF: wai-hr0w — Epic-orchestrator: strict validation, integration test, docs

## Why
Delegate to a subagent because the work is three independent, well-bounded
additions (integration test, docs section, distribution note) that require
no design judgment beyond following existing patterns — canon Invariant 1.
The lead keeps orchestration and verification; a fresh session writes the
test and docs without pulling the lead's context.

## Ticket
- ID: wai-hr0w (beads; epic-orchestrator co-modification thread)
- Type: code + docs, TDD where applicable

## Desired outcome
1. **Integration test**: extend `tests/integration.rs` (or a new
   `tests/suite_*.rs` file, matching current suite conventions — check
   recent `tests/suite_*` files first and follow them) that scaffolds the
   template in a temp repo via `wai pipeline init epic-orchestrator` and
   walks a simulated ticket through claim→brief→state asserting the
   state-file shape (`.wai/projects/<project>/runs/<ticket>.state` with
   ticket id, branch, brief path, step history entries `step id + sha`).
2. **Docs**: document the epic-orchestrator pipeline in
   `docs/src/concepts/pipelines.md` alongside the existing tdd-ro5
   section (see line ~243 for the established pattern). Cover: usage
   (`wai pipeline start epic-orchestrator --topic=<ticket>`), the loop
   steps (claim → gates → brief → spawn → verify → ship → STOP), the
   per-ticket state file (path, shape, retry/drift semantics), refusal
   conditions (spawn refuses without state file or committed brief;
   retry-entry drift check), and degradation behavior (single-model
   setups use lead-side checks — non-blocking; contradiction blocks
   ship).
3. **Distribution note**: a note linking provenance — the evidence base
   and proposal live at `~/.wai/projects/orchestrator-tooling/` — place
   it where the docs pattern suggests (e.g. a provenance line in the
   docs section and/or the openspec change README/proposal).
4. **openspec**: `openspec validate add-epic-orchestrator-template
   --strict` stays clean after any change-dir updates.

## Out of scope
- Template step-prompt changes (`src/templates/epic-orchestrator.toml`
  is frozen for this ticket — wai-qjcz owns further template edits).
- `src/` Rust changes.
- Pushing or closing the beads ticket.

## Exact files
- `tests/integration.rs` or new `tests/suite_pipeline_epic_orchestrator_*.rs`
- `docs/src/concepts/pipelines.md`
- `openspec/changes/add-epic-orchestrator-template/` (only if the
  distribution note or task list requires it)

## Exact commands
- `cargo test --test suite_pipeline_init_epic_orchestrator_template` (must stay green)
- New integration test: `cargo test --test <new-suite-name>`
- `openspec validate add-epic-orchestrator-template --strict`
- `cargo fmt --check`

## Red/green/refactor sequence
1. RED: write the integration test first; confirm it fails because the
   asserted behavior/shape is not yet exercised (it should mostly pass
   against the landed template — if it passes immediately, the test is
   still valid as a regression ratchet; note this in the report).
2. GREEN: docs + distribution note; all commands exit 0.
3. REFACTOR: wording tidy only.

## Follow-up threshold
Anything requiring template prompt changes or src/ Rust changes → stop
and file a follow-up child under epic `wai-fvhv`.

## Completion criteria (runnable; exit 0 = done)
- `cargo test` (full suite)
- `openspec validate add-epic-orchestrator-template --strict`
- `cargo fmt --check`

## Spawn model
- Model: same family as the orchestrator (default pi model); single-model
  environment — verifier will use the documented lead-side degradation.
- Context ceiling: **≤ 50% of the actual model window at spawn** — docs +
  one test file, brief ~2k tokens, fresh session. Fixed token counts are
  invalid; ceiling is a percentage of the live window.