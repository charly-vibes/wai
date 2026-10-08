---
tags: [pipeline-run:epic-orchestrator-2026-10-08-wai-lqdr, pipeline-step:claim]
---

ORIENT: selected wai-lqdr (bundle tests-pass oracle on epic-orchestrator init) — the template's verify/ship steps declare tests-pass oracle gates but init does not scaffold the oracle script, so fresh scaffolds gate-block; tdd-ro5 ships its oracle via suite_pipeline_init_tdd_ro5_ships_release_oracle_script pattern to mirror
