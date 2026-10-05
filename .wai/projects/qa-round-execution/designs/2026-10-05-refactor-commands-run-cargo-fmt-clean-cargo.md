---
tags: [pipeline-run:tdd-ro5-2026-10-05-wai-vx02-4-approval-release-oracle-gates, pipeline-step:refactor]
---

REFACTOR: commands run = cargo fmt (clean), cargo clippy --all-targets (0 warnings), cargo test --test integration -- release_oracle_ pipeline_init_tdd_ro5 pipeline_gates_tdd_ro5 (6 passed / 0 failed). Tidy done: (1) setup.rs — extracted write_executable_if_absent(path, contents) helper, deduplicating the write+chmod-0o755 block previously duplicated between the example oracle scaffold and the new bundled-oracle shipping loop; (2) docs — extracted the gate-tier table (Tier 1/2/3a/3b) into docs/src/snippets/gate-tiers.md, included in concepts/pipelines.md via mdbook {{#include docs/src/snippets/gate-tiers.md}}, so wai-fvhv.105 can reuse one source (pairs-with note inside the snippet). No behavior change; no behavior expansion.
