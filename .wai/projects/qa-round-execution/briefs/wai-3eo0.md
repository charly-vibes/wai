# BRIEF: wai-3eo0 — Orchestrator brief format (Why / completion criteria / spawn model)

## Why
Delegate this ticket to a subagent because it is a self-contained,
test-first template edit — exactly the shape the orchestrator pattern is
for (canon Invariant 1: the lead orchestrates, subagents implement). The
work is bounded to two files with an existing test suite to extend, so a
fresh session with a clean context executes it faster and without
dragging the lead's session history along.

## Ticket
- ID: wai-3eo0 (beads; epic-orchestrator co-modification thread)
- Type: code (template + test), TDD red→green→refactor

## Desired outcome
`src/templates/epic-orchestrator.toml` brief step encodes three mandatory
brief sections, and the spawn step refuses to spawn without a committed
brief:

1. `## Why` — delegation rationale (why this goes to a subagent).
2. `## Completion criteria` — runnable commands; exit 0 = done.
3. `## Spawn model` — model + context ceiling expressed as a **% of the
   actual model window computed at spawn**; fixed token counts are
   flagged invalid.

Brief is committed at `.wai/projects/<project>/briefs/<ticket>.md`
before spawn; the spawn step refuses otherwise.

## Out of scope
- Verify/verifier step changes (wai-979g owns the verify step block).
- State-durability step blocks (wai-wvjz, already landed — do not touch
  the claim/spawn refusal/state-checkpoint lines beyond the brief-related
  addition).
- Docs, strict validation, integration test (wai-hr0w).
- Shared template convention extraction (wai-qjcz).
- Any Rust engine changes — template prompts only.

## Exact files
- `src/templates/epic-orchestrator.toml` — brief step prompt + spawn step
  refusal rule only.
- `tests/suite_pipeline_init_epic_orchestrator_template.rs` — new RED
  test asserting the encoding (append; do not modify existing tests).

## Exact commands
- Red/green: `cargo test --test suite_pipeline_init_epic_orchestrator_template`
- Regression: `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script`
- Validate scaffold: `wai pipeline validate epic-orchestrator` (exit 0)
- Tidy gate: `cargo fmt --check`

## Red/green/refactor sequence
1. RED: add `pipeline_init_epic_orchestrator_encodes_brief_format` test
   asserting the scaffolded TOML contains `## Why`, `## Completion
   criteria`, `## Spawn model`, the `%` context-ceiling wording, the
   fixed-token-count invalidation rule, and the spawn refusal for an
   uncommitted brief. Confirm it fails for the missing behavior.
2. GREEN: edit the template's brief + spawn step prompts minimally until
   the test passes and validate exits 0.
3. REFACTOR: only reflow/wording tidy; no behavior expansion.

## Follow-up threshold
Anything needing Rust engine changes, or any scope beyond the two files
above → stop and file a follow-up child under epic `wai-fvhv` instead of
expanding.

## Completion criteria (runnable; exit 0 = done)
- `cargo test --test suite_pipeline_init_epic_orchestrator_template`
- `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script`
- `wai pipeline validate epic-orchestrator`
- `cargo fmt --check`

## Spawn model
- Model: same family as the orchestrator session (default pi model).
- Context ceiling: **≤ 50% of the actual model window at spawn** — the
  brief is ~1.5k tokens and the change is small; a fresh session starts
  near 0% so this ceiling is comfortably met. Fixed token counts are
  invalid; this ceiling is stated as a percentage of the live window.