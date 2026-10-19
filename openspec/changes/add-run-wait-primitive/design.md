# Design: add-run-wait-primitive

## Delivery vehicle: wai subcommand (revised per review CORR-001)

The primitive is `wai pipeline wait <pr> [--timeout <minutes>]`, not a
wai-repo justfile recipe. Rationale (verified against the friction data):

- The polling happens in **project repos** — dulce-de-leche, tambor, testaruda —
  each with its own justfile. A wai-repo recipe is unavailable exactly where the
  waste lives; no propagation mechanism (ddl sweep) was specified.
- wai is installed across the ecosystem; a subcommand is available in every
  spawned worktree and orchestrator session without per-repo setup.
- Rust-testable (exit-code mapping, summary-line formatting, timeout budget)
  without depending on a live PR — consistent with the sibling
  `add-spawn-preflight` design decision (subcommand over recipe).

## Blocking implementation choice

1. **`gh pr checks <pr> --watch`** (chosen) — gh's own watch mode is
   terminal-state aware and exits nonzero on failure. The polling is the
   tool's job, not a loop we maintain. wai wraps it with a timeout budget and
   the single summary line.
2. **`gh run watch <run-id>`** — per-run, not per-PR; wrong granularity.
3. **Hand-rolled `while sleep` loop** — rejected: exactly the anti-pattern
   this change removes.

## Timeout budget and the post-timeout summary (review EDGE-001)

Default budget 30m, `--timeout <minutes>` to override. A wedged CI run must
cost minutes, not hours — mirroring canon invariant 8's spawn-budget reasoning.

`timeout`-wrapping `gh pr checks --watch` kills gh before it prints state, so
the wrapper **re-queries `gh pr checks` once** after the budget expires to
capture the last observed state, then emits the summary line
(`<pr> timeout <elapsed> <state>`) and exits nonzero. The scenario contract
("elapsed budget and last observed state in its single summary line") is
satisfied by this re-query, not by the killed process.

## Zero-checks PRs (review EDGE-002)

A PR with zero checks is treated as **terminal-pass**: one summary line noting
zero checks, exit 0. Rationale: the orchestrator's question is "is the PR
green"; a PR with nothing to run is not wedged. Repos without CI therefore
never block on the primitive.

## Worktree waits (out of scope)

The scan also shows worktree log-tail polling loops (`sleep 300; for w in a b
c; tail waveN.log`). These wait on *files*, not PRs. Scoped out: the primitive
covers PR checks; worktree completion already has pokeable log paths (canon
invariant 7) and a spawn record — if the follow-up scan still shows file-wait
loops, a `wai pipeline wait --log <path>` variant can be added under the same
requirement later.

## Sequencing (review EXCL-001)

`add-spawn-preflight` also touches `src/commands/pipeline/orchestration.rs`
and `src/templates/epic-orchestrator.toml`. Implement this change **after**
preflight lands (or rebase carefully); template edits to the spawn step should
be merged in one pass to avoid two template-scan test rounds.
