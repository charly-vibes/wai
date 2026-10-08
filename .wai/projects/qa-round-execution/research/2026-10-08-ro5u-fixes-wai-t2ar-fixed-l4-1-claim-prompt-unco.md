---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-t2ar, pipeline-step:fix-review-findings]
---

RO5U-FIXES: wai-t2ar; fixed=L4.1 claim prompt unconditional 'bd ready' now guarded 'if beads is present' plus 'openspec list if openspec is present' in src/templates/epic-orchestrator.toml (template tests re-run 5/5 green); deferred=L4.2 help-example comment alignment (cosmetic, no ticket — one-line comment). Medium/critical: none. VERIFIED: commands run — cargo test --test suite_pipeline_init_epic_orchestrator_template (5 passed, 0 failed) after the fix; cargo test full suite green at GREEN step (1271 passed).
