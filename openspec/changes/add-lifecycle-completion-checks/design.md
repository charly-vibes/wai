## Context

The context-suggestion engine (`src/workflows.rs`) detects `WorkflowPattern`
variants from a `ProjectContext` snapshot and emits `WorkflowDetection` structs
containing suggestions. `wai status` renders these detections.

`ProjectContext` currently carries artifact counts and phase, but **not the phase
entry timestamp**. That timestamp lives in `ProjectState.history.last().started`
(a `DateTime<Utc>`), which is already written on every phase transition
(`state.rs::transition_to`).

## Goals / Non-Goals

- Goals: surface stale and complete projects to the user via `wai status`
- Non-Goals: auto-advance phases, change `wai doctor`, touch openspec tooling

## Decisions

### Decision: Where the checks live — status, not doctor

`wai doctor` semantics = "is the workspace structurally broken?" Checks are
pass/warn/fail with optional auto-fix. A project stuck in the wrong phase is not
a *broken workspace* — the `.state` file is valid YAML and the phase transition
is legal.

`wai status` semantics = "what should I do next?" The existing context-suggestions
engine is the natural fit: it already evaluates phase + artifact counts to produce
workflow suggestions.

Adding a new class of checks to doctor would expand its scope beyond structural
health and blunt its signal-to-noise ratio.

### Decision: Extend ProjectContext rather than loading state twice

`scan_project` already loads `ProjectState` to read `current`. Extending
`ProjectContext` to include `phase_started: DateTime<Utc>` costs one field in
the struct and reads from the state that is already in memory. The alternative
(letting `detect_patterns` receive a `&ProjectState`) leaks persistence concerns
into the detection layer — keep them separate.

### Decision: Two patterns, not three

An early design included `OrphanedProject` (research + 0 artifacts + >N days) as
a third pattern. In practice, `StalePhase` already fires for this case when the
threshold is crossed (the project is in research, no transition has happened in
14 days). A distinct pattern would produce duplicate or redundant suggestions.
`NewProject` (research + 0 artifacts, recent) already handles the "fresh" case.
The two-pattern model is sufficient.

### Decision: 14-day threshold for StalePhase, exported as a constant

14 days is a reasonable sprint boundary — most active work shows phase
advancement within a sprint. The threshold is exposed as
`pub const STALE_PHASE_DAYS: i64 = 14` so integration tests can construct
timestamps relative to it without hardcoding magic numbers.

### Decision: LooksComplete requires only review phase + ≥1 handoff

The simplest heuristic with no external dependencies. Openspec change completion
percentages require shelling out to `openspec list --json` — adding a subprocess
dependency to the detection engine is disproportionate for a suggestion. A
handoff artifact signals that the user *deliberately wrapped up a session* in the
review phase, which is a reliable proxy for intentional completion.

## Risks / Trade-offs

- **False positives for StalePhase**: a project can legitimately sit in
  `implement` for >14 days (large features, part-time work). The suggestion is
  non-blocking — it nudges rather than blocks. Acceptable.
- **False positives for LooksComplete**: a project in review with a handoff may
  still have open work. Again, the suggestion is non-blocking. The user decides.
- **No suppression mechanism**: there is no way to silence a suggestion for a
  specific project. This is intentional for now — YAGNI until real demand.

## Open Questions

- None blocking. Suppression ("snooze this project") can be a follow-up if
  false positives prove noisy in practice.
