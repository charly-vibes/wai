---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-gvqj, pipeline-step:red]
---

RED: command=cargo test --test suite_pipeline_preflight expected failure=3 failed — 'unrecognized subcommand preflight' (subcommand missing = the behavior under test, not setup breakage); template test fails on missing 'wai pipeline preflight' requirement in spawn step. Fixtures: stub-bin scripts fake lefthook/bd/testaruda; PATH restricted to stub bin dir.
