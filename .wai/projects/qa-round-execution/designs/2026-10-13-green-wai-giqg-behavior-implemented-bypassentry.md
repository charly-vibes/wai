---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-13-wai-giqg, pipeline-step:execute]
---

GREEN: wai-giqg; behavior implemented: BypassEntry{timestamp,reason} + serde(default) bypasses ledger on PipelineRun; wai pipeline bypass <reason> (orchestration::cmd_bypass — silent no-op without active run per spec); status surfacing via bypass_count+last_bypass_reason payload fields + human rendering 'Gate bypasses: N — last: ...'; docs/src/concepts/pipelines.md Disclosing gate bypasses section. Verification evidence: cargo test --test suite_pipeline_bypass_telemetry → 2 passed; all 14 pipeline suites → 108 passed 0 failed; cargo clippy --all-targets clean; cargo fmt --check clean.

Verification evidence (commands run): cargo test --test suite_pipeline_bypass_telemetry → 2 passed 0 failed; all pipeline suites → 108 passed 0 failed; cargo clippy --all-targets clean; cargo fmt --check clean.
