---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-gvqj, pipeline-step:fix-review]
---

RO5U-FIXES: F1 fixed — cmd_preflight check 3 changed from testaruda --version to testaruda init (spec orchestration-integrity requires store initialization; init idempotent; spec permits tool-store mutation). Tests re-run: suite_pipeline_preflight 3/3, builtin_templates_render 1/1, just test exit 0. F2/F3 low, accepted.

Verification evidence (commands run):
- `cargo test --test suite_pipeline_preflight` — test result: ok. 3 passed; 0 failed
- `cargo test --bin wai builtin_templates_render` — test result: ok. 1 passed
- `just test` — 1261 results ingested, exit 0, no failures
