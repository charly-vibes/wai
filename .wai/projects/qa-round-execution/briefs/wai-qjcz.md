# BRIEF: wai-qjcz — Tidy: extract shared template conventions (review-first)

## Why
Delegate to a subagent because this is a bounded, read-mostly review of
three template files with a mechanical conclusion space (extract / don't
extract) — canon Invariant 1. An independent session avoids the lead's
attachment to the wording it wrote in wai-wvjz/3eo0/979g.

## Ticket
- ID: wai-qjcz (beads; epic-orchestrator thread, final child)
- Type: review/decision work first; code change ONLY if the review
  concludes extraction is possible with zero behavior change.

## Desired outcome
A written review of `src/templates/epic-orchestrator.toml`,
`src/templates/tdd-ro5.toml`, `src/templates/scientific-research.toml`
covering:
1. Which convention blocks are duplicated across templates (e.g.
   "Advance: `wai pipeline next`" trail lines, Record/artifact lines,
   refusal boilerplate, per-step checkpoint lines).
2. For each: can it be extracted with **zero behavior change** given the
   template format? Check whether the pipeline template format supports
   includes/shared blocks/variables beyond `{topic}` (read
   `src/commands/pipeline/` template-loading code to confirm).
3. Verdict per the ticket: "extract shared helpers or constants if
   duplication is real — zero behavior change".

**Expected conclusion (hypothesis to confirm or refute):** the duplication
is real but the TOML template format has no include/shared-block
mechanism, so extraction requires an engine feature (template
preprocessing) — which is a behavior-adding feature, not a tidy. If so:
do NOT change the templates; instead write the finding into the review
artifact and leave extraction to the lead (a follow-up ticket will be
filed by the orchestrator). If the format DOES support shared blocks,
perform the minimal extraction, run the gates, and commit.

## Out of scope
- Any behavior change to step prompts' semantics.
- Any `src/` changes unless the format already supports extraction.
- Beads ticket close, push.

## Exact files (review)
- `src/templates/epic-orchestrator.toml`
- `src/templates/tdd-ro5.toml`
- `src/templates/scientific-research.toml`
- `src/commands/pipeline/` (template loading/format capabilities — read only)

## Exact commands
- `cargo test --test suite_pipeline_init_epic_orchestrator_template`
- `cargo test --test suite_pipeline_epic_orchestrator_ticket_state`
- `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script`
- `cargo fmt --check` (if any file is edited)

## Red/green/refactor sequence
Review-first: no RED applies unless extraction proves possible; if code
changes are made, existing suites are the ratchet (they must stay green
unchanged — that IS the zero-behavior-change proof).

## Follow-up threshold
If extraction requires engine changes → no code change, record the
finding; the orchestrator files the follow-up.

## Completion criteria (runnable; exit 0 = done)
- All three suites above exit 0 (unchanged, whether or not templates were edited).
- Review artifact exists with per-convention verdicts.

## Spawn model
- Model: same family (single-model environment; lead-side verification applies).
- Context ceiling: **≤ 50% of the actual model window at spawn** — read-mostly
  review, brief ~1.8k tokens, fresh session. Fixed token counts are invalid;
  ceiling is a percentage of the live window.