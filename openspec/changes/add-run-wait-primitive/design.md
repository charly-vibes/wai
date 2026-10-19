# Design: add-run-wait-primitive

## Blocking implementation choice

The primitive must block without a hand-written poll loop. Options considered:

1. **`gh pr checks <pr> --watch`** — gh's own watch mode; terminal-state aware;
   exits nonzero on failure. Chosen as the core: the polling is the tool's job,
   not a loop we maintain. Wrapper adds: `--interval` default, timeout budget,
   and the single summary line (`<pr> <state> <n> checks`).
2. **`gh run watch <run-id>`** — per-run, not per-PR; wrong granularity for the
   orchestrator's typical "is the PR green" question.
3. **A hand-rolled `while sleep` loop** — rejected: it is exactly the
   anti-pattern this change removes (the anti-goal is "no internal sleep-polling
   loops *behind the primitive*"; gh's watch mode is an upstream implementation,
   not our loop).

## Worktree waits

The scan also shows worktree log-tail polling loops (`sleep 300; for w in a b c;
tail waveN.log`). These wait on *files*, not PRs. Scoped out: the recipe covers
PR checks; worktree completion already has pokeable log paths (canon invariant 7)
and a spawn record — if the follow-up scan still shows file-wait loops, a
`just wt-wait` variant can be added under the same requirement later.

## Timeout budget

The recipe accepts an optional timeout (default 30m) so a wedged CI run costs
minutes, not hours — mirroring canon invariant 8's spawn-budget reasoning.
