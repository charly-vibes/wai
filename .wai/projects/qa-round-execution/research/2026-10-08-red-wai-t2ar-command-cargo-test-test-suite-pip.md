---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-t2ar, pipeline-step:red-or-analysis]
---

RED: wai-t2ar; command=cargo test --test suite_pipeline_init_epic_orchestrator_template; expected failure=3/5 fail for missing behavior: pipeline_init_epic_orchestrator_declares_orchestrator_loop_steps + cites_canon_pattern fail because init falls back to generic 'step-one/step-two' template (no built-in epic-orchestrator), pipeline_help_lists_epic_orchestrator_builtin fails because BUILTIN_PIPELINE_TEMPLATES_HELP lacks it; the 2 passing tests (file exists, valid TOML) assert fallback-template properties. RED evidence artifact: .wai/projects/qa-round-execution/research/
