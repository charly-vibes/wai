# Change: Harden orchestrator spawn step — wall-clock budget, resume exception, .beads prohibition

## Why

October 2026 pi-session analysis (386 sessions) showed the orchestrator pattern's
biggest waste source: 178 blocking `pi -p` orchestration calls totaling ~48h, of which
18 errored — several wedged the orchestrator until 3600–7200s bash timeouts (~10h upper
bound lost) because nothing sized a wall-clock budget to the ticket. The same data shows
recovery-from-scratch re-running 30–60 min of finished work instead of resuming the
existing named session, and 13 explicit `git add .beads` hard errors.

The canon (`~/.wai/resources/patterns/orchestrator-subagents.md`) now states these as
invariants 6 (resume exception), 8 (never blind-block, wall-clock budget), and 9
(never `git add .beads`). Invariant 10 ("Prose does not enforce") requires the
load-bearing parts to be encoded in the tooling — this template.

## What Changes

- **spawn step** — require the spawn command be wrapped in a wall-clock budget
  (`timeout <budget>`) sized to the ticket, stated in the brief; add the invariant-6
  exception: an interrupted-but-healthy spawn (timeout, orchestrator crash, zero/partial
  progress) resumes via `pi -c -p -n "<name>-resume"` against the existing named
  session — re-prompting the full brief from scratch is a violation.
- **verify step** — refuse verification of any spawn whose log path or session record
  cannot be located (a spawn that left no record is a failed spawn, not a pass).
- **ship step** — explicitly forbid `git add .beads` / committing the beads DB.
- Template header cites canon invariants 6, 8, 9 in addition to the existing set.

## Impact

- `src/templates/epic-orchestrator.toml` — spawn/verify/ship step prompts.
- Template content tests in `tests/` (assert budget wording, resume exception, `.beads`
  prohibition present in the scaffolded template).
- No CLI behavior change; prompt-level enforcement only (matching invariant 10's
  "tooling refuses bad spawns" trajectory — a `wai spawn` wrapper remains tracked in
  beads as wai-tdnv).
