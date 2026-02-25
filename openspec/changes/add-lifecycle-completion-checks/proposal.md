# Change: Lifecycle Completion Checks in wai status

## Why

Projects and phases accumulate silent debt. A project can sit in the wrong phase
for weeks after its work is done, because there is no signal that anything is
off. Today's cleanup session surfaced three real failures:

- `doctor-fix-flag` was fully implemented but stuck in `research` phase — no
  tool flagged it.
- `why-command` was complete and handoffs existed, but nothing suggested
  archiving it — it sat in `review` indefinitely.
- Projects in general can go stale with no artifacts added, yet `wai status`
  offers the same "add research" nudge as a brand-new project.

These are **workflow signals**, not filesystem health failures. That distinction
matters for placement:

- `wai doctor` = structural/filesystem integrity (config valid, dirs exist,
  projections synced). It checks *is the workspace broken?*
- `wai status` = workflow guidance (phase-aware suggestions, next steps). It
  answers *what should I do now?*

Staleness and completion belong in `wai status`, specifically in the
context-suggestions engine (`src/workflows.rs`), where phase-aware patterns
already live.

## What Changes

- **NEW pattern `StalePhase`**: when a project's phase has not changed in more
  than 14 days (and the project is not archived), surface a suggestion to either
  advance or archive.
- **NEW pattern `LooksComplete`**: when a project is in `review` phase and has at
  least one handoff artifact, surface a suggestion to archive it.
- **Extended `ProjectContext`**: add `phase_started: DateTime<Utc>` (read from
  `state.history.last().started`) to give the detection logic access to the
  phase entry timestamp without additional I/O.
- **Threshold constants**: `STALE_PHASE_DAYS = 14` exported from `workflows.rs`
  so integration tests can reason about them.

## Out of Scope

- `openspec archive` validation gate (openspec is an external tool; a separate
  convention or openspec-side proposal is needed).
- Beads ↔ openspec task cross-referencing (convention, not tooling change).
- Automatic phase transitions — the spec explicitly lists this as a non-goal of
  the project state machine. Suggestions only; no auto-advancing.

## Impact

- Affected specs: `context-suggestions`
- Affected code: `src/workflows.rs`, `src/commands/status.rs` (suggestion
  rendering), `tests/` (new unit + integration tests)
- No breaking changes — new patterns are additive
