# BRIEF: wai-vsn6 — Pipeline templates: shared-block/include mechanism

## Why
Delegate to a subagent because this is a well-bounded engine feature
with a crisp constraint (zero rendered-output change) and an existing
review documenting the duplication — canon Invariant 1. A fresh session
designs the minimal directive without inheriting the lead's template
editing history.

## Ticket
- ID: wai-vsn6 (beads)
- Type: code (Rust engine + template adoption), TDD red→green→refactor

## Constraint (hard)
**Zero behavior change to rendered prompts.** After this change, every
rendered step prompt for every built-in template must be byte-identical
to what the current code renders. The mechanism only changes how source
TOMLs express repeated content, not what gets rendered.

## Problem (from wai-qjcz review)
Template format only supports `{topic}` substitution
(`src/commands/pipeline/mod.rs:309-310`, `render_prompt`); built-ins are
copied verbatim (`setup.rs:66`). Convention boilerplate (Advance trail
lines, Record lines, checkpoint lines) is duplicated 7-9× per template
across 3 templates. Full review:
`.wai/projects/qa-round-execution/reviews/2026-10-08-template-convention-duplication-review-wai-qjcz.md`

## Desired outcome (smallest correct version)
1. **Mechanism**: extend `render_prompt` (or add a resolve pass next to
   it) supporting shared-block references, e.g. `{{block:name}}` that
   expands from a `[[blocks]]` table (or a dedicated `blocks.toml`)
   inside the SAME template file. Keep it minimal: same-file blocks
   only — NO cross-file includes, NO deploy-time resolution changes,
   NO scaffold output changes (scaffolded TOML stays as-authored).
   Unknown block name → hard error at load/validate time (fail loudly,
   not silently).
2. **Ratchet tests**: (a) rendered prompts for ALL built-in templates
   are byte-identical before/after the mechanism exists (capture
   current rendered output in the test as expectations, or diff against
   a pre-change snapshot — your choice, must be deterministic);
   (b) unknown block reference fails validation with a clear error;
   (c) `wai pipeline validate` passes for all built-ins.
3. **Adoption**: adopt the mechanism for ONE convention only — the
   `Advance: \`wai pipeline next\`` trail line in
   `epic-orchestrator.toml` — as proof. Do NOT convert all templates
   and all conventions in this ticket (that is mechanical follow-up
   work, tracked separately by the orchestrator).

## Out of scope
- Cross-file includes; deploy/scaffold path changes; rendered-output
  changes; converting tdd-ro5/scientific-research or other conventions.
- Gate semantics changes.

## Exact files (likely)
- `src/commands/pipeline/mod.rs` (render_prompt + block resolution)
- `src/commands/pipeline/definition.rs` (load/validation, if blocks live in TOML)
- `src/templates/epic-orchestrator.toml` (adopt for the Advance trail)
- new test suite (e.g. `tests/suite_pipeline_template_blocks.rs`)

## Exact commands
- `cargo test` (full suite green)
- `cargo fmt --check`

## Red/green/refactor sequence
1. RED: byte-identical rendering test + unknown-block-error test.
2. GREEN: mechanism + adoption.
3. REFACTOR: none expected; keep minimal.

## Follow-up threshold
If byte-identical rendering proves impossible for the adopted convention
(e.g. block content must differ per step), reduce adoption scope rather
than relaxing the constraint; if the mechanism grows beyond same-file
blocks → stop and file follow-ups under epic `wai-fvhv`.

## Completion criteria (runnable; exit 0 = done)
- `cargo test` full suite green including new ratchet tests
- `cargo fmt --check`

## Spawn model
- Model: same family (single-model environment; lead-side verification).
- Context ceiling: **≤ 50% of the actual model window at spawn**; fresh
  session. Fixed token counts are invalid; ceiling is a percentage of
  the live window.