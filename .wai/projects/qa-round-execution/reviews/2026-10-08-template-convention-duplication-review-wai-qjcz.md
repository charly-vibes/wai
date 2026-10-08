tags: [pipeline-run:epic-autonomy-tdd-ro5, pipeline-step:review, ticket:wai-qjcz, kind:review]

# Review: duplicated conventions across pipeline templates (wai-qjcz)

**Scope:** `src/templates/epic-orchestrator.toml`, `src/templates/tdd-ro5.toml`,
`src/templates/scientific-research.toml`, plus the template-loading engine in
`src/commands/pipeline/` (read only).
**Ticket:** wai-qjcz (tidy, review-first; code change only if extraction is
possible with zero behavior change).
**Verdict: NO code changes.** Duplication is real, but the TOML template
format has no include/shared-block mechanism; extraction requires an engine
preprocessing feature, which is behavior-adding — not a tidy.

## Format capability check (evidence)

The pipeline template format supports exactly one templating feature: the
`{topic}` placeholder, substituted at render time.

- `src/commands/pipeline/mod.rs:309-310` — `render_prompt()` is a single
  `prompt.replace("{topic}", topic)`. No other placeholder, variable, or
  directive is processed.
- `src/commands/pipeline/definition.rs:32-55` — `load_pipeline_toml()` reads
  the file and passes the raw string straight to `toml::from_str`. There is
  no preprocessing pass, no include/extends/shared-block resolution, and no
  variable expansion before deserialization.
- `src/commands/pipeline/setup.rs:66` + `setup.rs:162-164` — built-in
  templates are `include_str!`'d verbatim and written to
  `.wai/resources/pipelines/` unmodified (`fs::write(&file_path, template)`).
  The copy step is also inert — no opportunity to resolve shared blocks at
  deploy time.

Conclusion: any cross-file sharing of prompt fragments would require a new
engine feature (e.g. a preprocessing pass resolving includes or shared
blocks at load/deploy time). Per ticket instructions: no code changes.

## Per-convention verdicts

| # | Convention block | Where duplicated | Duplication? | Verdict |
|---|------------------|------------------|--------------|---------|
| C1 | `Advance: \`wai pipeline next\`` trail line | 7× epic-orchestrator, 8× tdd-ro5, 7× scientific-research (every step) | Yes, real | **Cannot extract zero-change** — needs engine preprocessing |
| C2 | `Record X: \`wai add <type> "..."\`` line | 7× / 9× / 7× (every step) | Yes, real | **Cannot extract zero-change** — needs engine preprocessing |
| C3 | `State checkpoint: append the step id + sha (current HEAD) to the state file...` | 8× epic-orchestrator only (7 advancing + 1 complete) | Yes, real (intra-template) | **Cannot extract zero-change** — needs engine preprocessing |
| C4 | Orientation command list (`wai prime`, `wai status`, `bd ready`, `openspec list`, `wai search "<topic>"`) | epic-orchestrator:33-37 (`claim`), tdd-ro5:26-30 (`orient`) | Yes, verbatim | **Cannot extract zero-change** — needs engine preprocessing |
| C5 | Push-refusal boilerplate | epic-orchestrator:204 (`push only when project policy...`), tdd-ro5:221 (`Do not push unless...`) | Yes, but **wording differs** | **Cannot extract zero-change** — unifying wording changes prompt text (behavior change) |
| C6 | `[[steps.gate.oracles]] name="tests-pass" timeout=300` block | Repeated within and across epic-orchestrator + tdd-ro5 (description strings differ slightly) | Yes, real | **Cannot extract zero-change** — needs engine preprocessing; description strings differ |
| C7 | Structural gate blocks `min_artifacts = 1` (+ `types = ["research"]`) | 6× epic-orchestrator, 5× tdd-ro5 | Yes, real | **Cannot extract zero-change** — needs engine preprocessing |
| C8 | `[pipeline.metadata] skills = ["tdd", "rule-of-5-universal", "commit"]` | epic-orchestrator:25, tdd-ro5:18 (identical) | Yes, verbatim | **Cannot extract zero-change** — needs engine preprocessing |

Notes:

- C3 is the heaviest *intra*-template duplication (epic-orchestrator) but
  appears in no other template — cross-template sharing wouldn't even help.
- C5 is the only case where even a hypothetical include mechanism would not
  achieve zero behavior change: the two templates intentionally (or
  incidentally) use different phrasing, so unifying them edits prompts.
- Even if an include mechanism existed, extracting per-step lines (C1–C3)
  would likely push shared fragments into step-local context where wording
  is intentionally step-specific — extraction value is moderate at best.

## Ratchet proof

All three suites run unchanged and green (zero-behavior-change proof for the
no-op outcome):

- `cargo test --test suite_pipeline_init_epic_orchestrator_template` — 8 passed, 0 failed
- `cargo test --test suite_pipeline_epic_orchestrator_ticket_state` — 3 passed, 0 failed
- `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script` — 34 passed, 0 failed

## Changes made

None. No template or `src/` file was edited. `cargo fmt --check` not needed
(no edits), though the tree is unaffected either way.

## Follow-up for the orchestrator

If shared-template support is ever wanted, it is an engine feature ticket:
template preprocessing at load/deploy time (e.g. an `#include` directive
resolved in `load_pipeline_toml` (definition.rs:32) or at `setup.rs:66`
deploy time), with its own tests and an openspec proposal (breaking-ish
change to the template format contract). Until then, the duplication is
accepted as the cost of the current self-contained template format.
