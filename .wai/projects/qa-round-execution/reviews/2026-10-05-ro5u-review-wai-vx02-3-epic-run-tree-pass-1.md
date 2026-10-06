---
reviews: ro5u-wai-vx02-3-pass1.md
verdict: pass
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-05-wai-vx02-3-epic-run-tree, pipeline-step:fix-review-findings]
---

# RO5U Review: wai-vx02.3 epic run tree (pass 1)

Artifact under review: design `2026-10-05-wai-vx02-3-epic-run-tree-design.md` +
implementation in `src/cli/subcommands.rs`, `src/commands/pipeline/mod.rs`,
`src/commands/pipeline/orchestration.rs`, `src/json.rs`, `tests/integration.rs`.

## Findings

### Critical
- none.

### High
- none.

### Medium
1. **Idempotency check ordering** — `src/commands/pipeline/orchestration.rs`
   `cmd_start_epic`: `discover_ready_children` runs before
   `find_epic_parent_run`, so re-starting an epic whose parent is mid-flight
   but whose children are all claimed (nothing "ready") prints "no ready
   children — no parent run created" instead of "Epic parent run already
   active". Evidence: call order at the top of `cmd_start_epic`. Concrete fix:
   check `find_epic_parent_run` before discovery and return the reuse message.
2. **Parent reuse ignores pipeline name** — `find_epic_parent_run` matches any
   run state with `epic == <id>` regardless of which pipeline the parent run
   belongs to; starting pipeline B with `--epic=<id>` would reuse a parent
   created under pipeline A. Evidence: the predicate compares only
   `run.epic.as_deref() == Some(epic_id)`. Concrete fix: also require
   `run.pipeline == name`.

### Low
3. `record_child_run_on_epic_parent` scans every run yml on every plain start
   — O(n) file reads per start; acceptable at expected scale, no change needed.
4. Child-run guidance prints `wai pipeline start <pipeline> --topic=<child>`,
   which moves `.last-run` off the parent; parent resume relies on
   `WAI_PIPELINE_RUN` or the printed export line. Acceptable, documented
   behavior.

## Verdict

Pass. Both Medium findings are real but do not break the ticket's acceptance
behavior; they are edge-UX and cross-pipeline edge cases. Recorded for a
follow-up ticket rather than expanding this ticket's scope.

