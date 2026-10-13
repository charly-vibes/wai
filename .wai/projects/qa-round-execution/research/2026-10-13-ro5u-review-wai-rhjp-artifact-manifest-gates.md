---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-13-wai-rhjp, pipeline-step:fix-review-findings]
---

# RO5U review: wai-rhjp (artifact-manifest gates)

5-pass review of the gate-block diff (src/templates/epic-orchestrator.toml, src/templates/tdd-ro5.toml, tests/suite_pipeline_template_blocks.rs).

## Critical

1. **tdd-ro5 ship-close structural gate is unsatisfiable for a prompt-literal agent.**
   - Location: src/templates/tdd-ro5.toml, ship-close step, new `[steps.gate.structural] min_artifacts = 1`.
   - Evidence: the ship-close prompt contains no "Record" line — it says to run `wai close`. src/commands/close.rs:14-80 confirms `wai close` creates an untagged handoff markdown, NOT a step-tagged artifact (structural gate counts artifacts with `pipeline-step:ship-close` frontmatter tags, per src/commands/pipeline/gates.rs find_step_artifacts). Every future tdd-ro5 run would dead-end at completion.
   - Fix: add a Record ship line (`wai add research "SHIP: ..."`) to the ship-close prompt, mirroring epic-orchestrator ship. Regenerating the rendered baseline becomes REQUIRED after this prompt change.

## High

(none)

## Medium

1. Test token normalization uses trim_matches('m') — fragile if a step id ever starts/ends with 'm'. No such id today; documented in test comment. Deferred with reason: no current id affected; proper fix is improving strip_ansi (shared helper, out of boundary).

## Low

1. step_starts Vec rebuilt per template — fine at this scale.
2. epic-orchestrator ship gate types=["research"] matches its existing SHIP record line — consistent, no action.

Verification evidence: full diff read; `wai pipeline gates tdd-ro5` output inspected; src/commands/close.rs read to confirm handoff is not a tagged artifact.


## Verification evidence (commands run)

- `cargo test --test suite_pipeline_template_blocks` → 4 passed, 0 failed
- `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script --test suite_pipeline_gates_tdd_ro5_shows_approval_on_ship_close` (+ 5 other pipeline suites) → all passed
- `UPDATE_BASELINE=1 cargo test --bin wai builtin_templates_render` → fixture diff = expected 2-line ship-close prompt addition only
- `cargo test --bin wai builtin_templates_render` (ratchet, no update) → passed
- `cargo clippy --all-targets` → clean
- `cargo fmt --check` → clean
- `wai pipeline gates tdd-ro5` → Structural now declared on green, refactor, ship-close; epic-orchestrator ship verified in suite output
