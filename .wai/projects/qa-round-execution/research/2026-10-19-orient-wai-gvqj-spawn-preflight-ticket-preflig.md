---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-gvqj, pipeline-step:orient]
---

ORIENT: wai-gvqj spawn preflight — ticket: preflight each spawned worktree at orchestrator spawn (lefthook present, bd list sane, testaruda store init) fail-fast actionable, no auto-install/no network. Openspec change add-spawn-preflight tasks.md 1.1-3.4, red-first: preflight_reports_all_failures, preflight_passes_clean_env, orchestrator_template_requires_preflight; impl wai pipeline preflight subcommand in orchestration.rs + template spawn step requires preflight. Sequenced before add-run-wait-primitive (co-modification EXCL-001). Files: src/commands/pipeline/orchestration.rs, template.
