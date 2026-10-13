---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-13-wai-rhjp, pipeline-step:fix-review-findings]
---

RO5U-FIXES: wai-rhjp; fixed=critical-1 (ship-close prompt now records SHIP artifact; baseline fixture regenerated with expected 2-line drift; suites green); deferred=medium-1 (strip_ansi trim workaround — shared helper change out of boundary, noted on ticket); low=no-action

## Verification evidence (commands run)

- `cargo test --test suite_pipeline_template_blocks` → 4 passed, 0 failed
- `cargo test --test suite_pipeline_init_tdd_ro5_ships_release_oracle_script --test suite_pipeline_gates_tdd_ro5_shows_approval_on_ship_close` (+ 5 other pipeline suites) → all passed
- `UPDATE_BASELINE=1 cargo test --bin wai builtin_templates_render` → fixture diff = expected 2-line ship-close prompt addition only
- `cargo test --bin wai builtin_templates_render` (ratchet, no update) → passed
- `cargo clippy --all-targets` → clean
- `cargo fmt --check` → clean
- `wai pipeline gates tdd-ro5` → Structural now declared on green, refactor, ship-close; epic-orchestrator ship verified in suite output
