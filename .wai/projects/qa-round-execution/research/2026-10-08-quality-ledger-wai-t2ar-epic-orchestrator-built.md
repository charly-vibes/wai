---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-t2ar, pipeline-step:quality-ledger]
---

QUALITY LEDGER: wai-t2ar — epic-orchestrator built-in pipeline template registration.

Changed:
- src/templates/epic-orchestrator.toml (new): orchestrator loop claim→gates→brief→spawn→verify→ship→stop; header cites canon orchestrator-subagents.md + Invariant 8 + invariants 1,2,3,4,5,6,7; gate syntax mirrors tdd-ro5 (structural min_artifacts, tests-pass oracles on verify/ship)
- src/config.rs: BUILTIN_PIPELINE_TEMPLATES + BUILTIN_PIPELINE_TEMPLATES_HELP now include epic-orchestrator
- src/commands/pipeline/setup.rs: get_builtin_template match arm for epic-orchestrator
- src/cli/subcommands.rs, src/cli/mod.rs, src/help.rs: help/example text listing epic-orchestrator
- tests/suite_pipeline_init_epic_orchestrator_template.rs (new): 5 tests — scaffold, loop steps, canon header, TOML validity, help listing
- openspec/changes/add-epic-orchestrator-template/: proposal.md, tasks.md (1.1–1.4), pipeline-resource delta spec

Verified:
- commands run: cargo test --test suite_pipeline_init_epic_orchestrator_template (5 passed, 0 failed); cargo test full suite (1271 passed, 0 failed); openspec validate add-epic-orchestrator-template --strict (valid); manual smoke: pipeline init/validate/gates epic-orchestrator in fresh temp workspace via target/debug/wai (all steps resolve)

Review:
- RO5U pass 1 APPROVED: 0 critical, 0 high, 0 medium, 2 low (1 fixed, 1 deferred cosmetic). Artifact: reviews/2026-10-08-ro5u-findings-pass-1-for-wai-t2ar-epic-orchestrator.md

Risks:
- known remaining risks: none known; fresh-workspace tests-pass oracle warning is parity with tdd-ro5, not new

Next:
- commit atomically on feat/epic-orchestrator-template; open PR; after merge, children wai-wvjz / wai-3eo0 / wai-979g unblock (they edit later step blocks of the same TOML)
