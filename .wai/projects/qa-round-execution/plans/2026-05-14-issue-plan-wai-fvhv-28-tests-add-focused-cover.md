---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-05-13-work-one-ready-child-issue-from-epic-wai-fvhv, pipeline-step:plan]
---

ISSUE PLAN: wai-fvhv.28 — Tests: add focused coverage for wai doctor. Outcome: tests/doctor_test.rs exists with 6 passing tests covering healthy workspace, broken conditions (missing dir, invalid config, corrupted state), and fix paths (--yes repair, --safe refusal). Out of scope: migrating all doctor tests from integration.rs. Files touched: tests/doctor_test.rs (new). Tests: cargo test --test doctor_test. Type: code. Red/green/refactor: file created green from scratch (no migration needed). No follow-up issues discovered.
