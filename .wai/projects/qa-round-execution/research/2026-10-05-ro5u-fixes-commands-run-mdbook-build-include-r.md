---
tags: [pipeline-run:tdd-ro5-2026-10-05-wai-vx02-4-approval-release-oracle-gates, pipeline-step:fix-review]
---

RO5U-FIXES: commands run = mdbook build (include renders, 1 match of gate-tier table in docs/_book/concepts/pipelines.html; pre-existing <name>/<step-id> warnings in specs/pipeline-resource.md unchanged), cargo test --test integration -- release_oracle_ pipeline_init_tdd_ro5 pipeline_gates_tdd_ro5 (6 passed / 0 failed). M1 fixed: src/templates/oracles/release-docs-fresh.sh heading-extraction pipeline guarded with || true so the 'no release heading' stderr branch is reachable under set -euo pipefail; mirrored to .wai/resources/oracles/release-docs-fresh.sh. M2 fixed: mdbook include path corrected to {{#include ../snippets/gate-tiers.md}} (relative to including file). L1 fixed: unused ARTIFACT variable removed from oracle, contract documented in comment. Critical/high findings: none. Medium/low remaining: none. Tests re-run after fixes: green.
