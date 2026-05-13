---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-05-13-pipeline-gates-and-approvals-tests-wai-fvhv-108, pipeline-step:plan]
---

ISSUE PLAN: wai-fvhv.108 — Pipeline gates and approvals tests

Outcome: Add integration tests for gate display, dry-run evaluation, and approval recording.

Scope:
- Test cmd_gates (definition display + live status)
- Test cmd_approve (timestamp recording, complete-run rejection)
- Test gate evaluation tiers: structural (min_artifacts), procedural (require_review, max_critical), approval (required flag)
- Test approval invalidation when artifact created after approval
- At least one blocked/failure path

Out of scope:
- Oracle script execution tests (needs mock infra, separate ticket)
- Coverage gate (require_input_manifest) — thin behavior, separate ticket
- Refactoring pipeline.rs (blocked by this ticket via wai-fvhv.49)

Files touched:
- tests/integration.rs — new test functions in pipeline section
- .wai/resources/pipelines/ — test fixture TOML if needed (inline preferred)

Red/green/refactor:
1. RED: Write failing tests for gates display, approve, structural gate, procedural gate, approval invalidation
2. GREEN: Tests should pass against existing implementation (no src changes expected)
3. TIDY: Clean up test helpers if patterns emerge

Commands: cargo test --test integration -- pipeline_gate
Follow-up threshold: any oracle/coverage test ideas become new tickets
