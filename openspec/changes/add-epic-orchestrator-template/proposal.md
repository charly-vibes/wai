# Change: Add epic-orchestrator built-in pipeline template

## Why

The orchestrator + subagents pattern (`~/.wai/resources/patterns/orchestrator-subagents.md`)
is load-bearing for wai's own development (epic wai-vx02 ran entirely through it), but it
currently exists only as prose plus manual discipline. Invariant 8 of the pattern states:
"Prose does not enforce — wherever the pattern is load-bearing, encode the loop as a wai
pipeline + brief template + status recipe, and let the tooling refuse bad spawns." The
orchestrator loop (claim → gates → brief → spawn → verify → ship → STOP) needs to be a
registered, scaffordable pipeline so tooling can enforce the pattern instead of hoping
agents follow prose.

## What Changes

- Add `src/templates/epic-orchestrator.toml` — a built-in pipeline template encoding the
  orchestrator loop as ordered steps:
  1. **claim** — atomically claim the next ready child ticket (`bd update <id> --claim`)
  2. **gates** — run pre-work gates (`wai pipeline gates` / project quality gates)
  3. **brief** — write the subagent brief as a committed repo artifact (`.wai/projects/<name>/briefs/`)
  4. **spawn** — spawn a named, resumable subagent session streaming to a known log path
  5. **verify** — verify before trust: `git log` vs report, repo gates, ticket state
  6. **ship** — push, close the ticket, export beads
  7. **STOP** — hard stop at the ticket boundary; no chaining past it
- Register `epic-orchestrator` in `BUILTIN_PIPELINE_TEMPLATES` (`src/config.rs`) so
  `wai pipeline init epic-orchestrator` scaffolds it and `wai pipeline list` offers it.
- Header of the template cites canon: `~/.wai/resources/patterns/orchestrator-subagents.md`
  and its Invariant 8.
- Template test mirrors the tdd-ro5 template tests in `tests/` (scaffold into a temp repo,
  assert content and listing).

Scope note: this change covers template registration only (tasks 1.1–1.4). Run-state
durability (wai-wvjz), brief format (wai-3eo0), and the isolation-based verify step
(wai-979g) are separate tickets editing later step blocks of the same TOML.

## Impact

- Affected specs: `pipeline-resource`
- Affected code:
  - `src/templates/epic-orchestrator.toml` (new)
  - `src/config.rs` (`BUILTIN_PIPELINE_TEMPLATES`, `BUILTIN_PIPELINE_TEMPLATES_HELP`)
  - `src/cli/subcommands.rs`, `src/cli/mod.rs`, `src/help.rs` (template listing/help text)
  - `src/commands/pipeline/setup.rs` (built-in scaffold path — no gate change needed, list-driven)
  - `tests/` (new template test mirroring tdd-ro5 tests)
