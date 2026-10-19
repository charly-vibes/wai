# Change: Add canonical run wait primitive (`wai pipeline wait`) wired into orchestrator runs

Tracked as: wai-63h9 (P1). Baseline measurement: `scripts/session-friction.py` (PR #53).
Revised per Rule-of-5 review (CORR-001: delivery vehicle must reach the repos where
polling happens — dulce-de-leche, tambor, testaruda — not just wai's own justfile).

## Why

The week 2026-10-13..19 friction scan (318 sessions, 36.9k tool calls) measured the
workflow's largest wall-clock waste: **539 `sleep`-poll commands of ≥60s, ≈3,163
minutes (~53h) of blocked polling**. Orchestrator and monitor sessions improvise
`sleep 300; date; gh pr list/checks` and worktree log-tail loops because the
epic-orchestrator template says "Poll or log rather than blind-blocking" without
providing a canonical way to wait. Each poll is a wasted tool turn plus 5 minutes
of blocked wall-clock; a single orchestrator run burns dozens of both.

The polling happens in project repos (dulce-de-leche, tambor, testaruda) that each
have their own justfiles — so the primitive must ship with **wai itself** (installed
across the ecosystem), not as a wai-repo-only recipe.

Canon invariant 10 ("prose does not enforce") says the load-bearing part must be
encoded in tooling — the template currently encodes the *prohibition* (no blind
blocking) but not the *capability* (how to wait).

## What Changes

- **`wai pipeline wait <pr>`** — a canonical blocking wait subcommand: blocks until
  the PR's checks reach a terminal state (or the status flips), emits exactly one
  summary line, exits nonzero on check failure or timeout exhaustion. No polling
  loop is written by the agent or by wai (gh's own watch mode does the waiting).
- **epic-orchestrator template** — the spawn step's "Poll or log rather than
  blind-blocking" guidance is upgraded: waiting on CI/worktree status MUST use the
  wait primitive; free-form `sleep <N>` poll loops are named as a violation.
- **No gate semantics change** — the primitive is a waiting convenience, not a gate
  tier; gate evaluation order is untouched.

## Impact

- **Affected specs:** `orchestration-integrity` (ADDED requirement: Orchestrator
  Wait Primitive)
- **Affected code:** `src/commands/pipeline/orchestration.rs` (new subcommand),
  `src/templates/epic-orchestrator.toml` (spawn step prompt), `tests/`
  (regression tests)
- **Follow-up measurement:** next weekly scan must show `sleep_poll_ge60`
  commands reduced ≥80% vs baseline 539 (ticket Must gate). Attainment depends on
  adoption beyond the orchestrator template (monitor sessions too) — the
  subcommand's availability everywhere wai is installed is what makes that possible.
