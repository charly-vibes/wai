# Change: Add spawn preflight for orchestrator worktree agents

Tracked as: wai-gvqj (P2). Baseline measurement: `scripts/session-friction.py` (PR #53).
Revised per Rule-of-5 review (CORR-002: preflight ordering pinned — the worktree
must exist before the preflight can run inside it).

## Why

Week 2026-10-13..19 scan: **7 opaque lefthook pre-commit failures** (`npx lefthook`
exit 2 with no output — lefthook not installed in the spawned worktree), **11 bd
failures** from wrong issue-ID prefixes, and **17 testaruda failures**. All three
are environment-state problems that exist *before* the agent does any work, but
are discovered mid-run: the agent burns 30–60 minutes before hitting them, and
the failure messages name neither the missing tool nor its fix.

Canon invariant 10 ("prose does not enforce") applies: telling agents to check
their environment in the brief does not survive a fast spawn. The check must be
a command that fails fast with an actionable message.

## What Changes

- **`wai pipeline preflight`** — a subcommand run inside the newly created
  worktree, immediately after its creation and before the subagent launches,
  that verifies: (1) lefthook executable is present, (2) `bd list` exits 0,
  (3) testaruda local store is initialized (or auto-initializes). The spawn
  sequence is: create worktree → preflight → launch agent. Each failure
  exits nonzero with a message naming the tool and its fix (install/init paths).
- **epic-orchestrator template** — the spawn step MUST require the preflight to
  pass before spawning; a failed preflight aborts the run at spawn, not mid-flight.
- **No auto-install** — the preflight fails fast and tells the agent how to fix
  it; it never installs tools itself and makes no network calls.

## Impact

- **Affected specs:** `orchestration-integrity` (ADDED requirement: Spawn
  Preflight)
- **Affected code:** `src/commands/pipeline/orchestration.rs` (new subcommand),
  `src/templates/epic-orchestrator.toml` (spawn step prompt), `tests/`
  (spawn-preflight regression test)
- **Follow-up measurement:** next weekly scan must show the
  lefthook-exit-2-no-output class at 0 occurrences vs baseline 7 (ticket Must gate).
