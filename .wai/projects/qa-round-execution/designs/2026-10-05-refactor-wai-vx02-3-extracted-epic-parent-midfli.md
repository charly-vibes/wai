---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-05-wai-vx02-3-epic-run-tree, pipeline-step:refactor-or-tidy]
---

REFACTOR: wai-vx02.3; extracted epic_parent_midflight_children() + build_epic_tree() helpers out of cmd_next/pipeline_current_status inline blocks; child_def_steps reused for both mid-flight checks; no behavior change.

## Verification (commands run)

- `cargo test --test integration epic_` → 5 passed, 0 failed
- `cargo test` → all suites pass; single failure is known timing-flake
  `plugin::tests::execute_hook_no_deadlock_on_fast_command` (wai-z25x),
  passes in isolation
- `cargo fmt --all` → clean
- `cargo clippy --all-targets` → 0 warnings
