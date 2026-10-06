---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-05-wai-vx02-3-epic-run-tree, pipeline-step:red-or-analysis]
---

RED: wai-vx02.3 epic run tree; command=cargo test --test integration epic_; expected failure=5/5 fail: no --epic flag on start (parent run never created/no ready-children skip), child start does not append run id to parent child_runs, next advances parent while child-run mid-flight (should refuse), current --json renders no epic tree; artifacts=design 2026-10-05-wai-vx02-3-epic-run-tree-design.md (test shape defined first)
