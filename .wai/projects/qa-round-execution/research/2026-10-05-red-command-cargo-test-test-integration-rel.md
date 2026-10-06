---
tags: [pipeline-run:tdd-ro5-2026-10-05-wai-vx02-4-approval-release-oracle-gates, pipeline-step:red]
---

RED: command='cargo test --test integration -- release_oracle_ pipeline_init_tdd_ro5 pipeline_gates_tdd_ro5' result=5 failed / 1 passed (filtered). Failure modes as expected, not setup breakage: pipeline_init_tdd_ro5_ships_release_oracle_script fails because pipeline init does not ship release-docs-fresh.sh (NotFound); the three release_oracle_* tests fail on the same missing-script root cause (sandbox setup itself works — git commit helper succeeded, since panics happen only at script invocation); pipeline_gates_tdd_ro5_shows_approval_on_ship_close fails because gates output for shipped tdd-ro5 shows only 'Oracle: tests-pass' on ship-close with no Approval tier. Expected failures match missing behavior: (a) no approval gate in src/templates/tdd-ro5.toml ship-close, (b) no release oracle script shipping in setup.rs, (c) no oracle implementation.
