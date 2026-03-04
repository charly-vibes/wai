## Context

`wai way` and `wai doctor` both run checks and produce fix recommendations, but
they serve different concerns: `way` answers "is this a well-run repo?" while
`doctor` answers "is the wai workspace correctly configured?". They are
genuinely distinct but visually similar — both print icon-prefixed lines, both
offer suggestions. Users are uncertain which to run; the cross-reference in
73286cd mitigated but did not eliminate this.

The design decision and option analysis are captured in full in:
`.wai/projects/friction-analysis/designs/2026-03-04-design-unifying-wai-way-and-wai-doctor.md`

This document records the technical decisions for implementation.

## Goals / Non-Goals

- Goals:
  - Single unified entry point (`wai check`) that runs both
  - Graceful degradation when `.wai/` does not exist
  - No breaking changes to `wai way` or `wai doctor`
  - Internal refactor reduces duplication without changing visible behavior
- Non-Goals:
  - Exposing fix mode through `wai check` (fix operations stay on individual commands)
  - Merging `way` and `doctor` into a single command (Option C — rejected)
  - Changing output format of `wai way` or `wai doctor` in isolation

## Decisions

### Decision: Shared `CheckResult` type

Both `way.rs` and `doctor.rs` define their own `CheckResult` and `Status`
structs. The types are nearly identical; the difference is `way` uses
`Pass/Info` while `doctor` uses `Pass/Warn/Fail`.

**Decision**: Define a single `pub struct CheckResult` and `pub enum Status`
with the superset of variants: `Pass`, `Info`, `Warn`, `Fail`. Place it in
`src/checks.rs` (new module) and import it in both `way.rs` and `doctor.rs`.

**Alternatives considered**:
- Keep local types, re-map at the boundary in `check.rs`: more indirection,
  more types to maintain. Rejected.
- Use `doctor`'s type and map `Info` → `Warn` for `way`: semantically wrong —
  `way` items are never failures. Rejected.

### Decision: `run_checks()` visibility and signature

Both `way::run_checks()` and `doctor::run_checks()` are `pub` so `check.rs`
can call them. They take `&Path` (repo root / project root respectively).

`doctor::run_checks()` returns an empty Vec if no `.wai/` workspace is found
(rather than returning an error). This lets `check.rs` distinguish "no workspace"
from "workspace has failures" cleanly: an empty Vec with a sentinel or a
separate bool signal from a `Result<Option<Vec<CheckResult>>, _>` return type.

**Decision**: `doctor::run_checks()` returns `Result<Option<Vec<CheckResult>>>`:
- `Ok(None)` — no workspace found; `check.rs` prints the skip message
- `Ok(Some(checks))` — workspace found, checks collected
- `Err(...)` — unexpected error (config parse failure, I/O error)

`way::run_checks()` returns `Vec<CheckResult>` (never fails, same as current behavior).

### Decision: `check.rs` section headers

Section headers use a box-drawing style consistent with the design artifact:

```
┌─ Repo hygiene (wai way) ──────────────────────────────────
```

This is a human-mode-only concern. JSON mode emits no headers.

### Decision: Moving beads/openspec checks from `way` to `doctor`

Currently `way` includes `check_beads()` and `check_openspec()`. These check
whether the tools are present in the repo — conceptually, they belong to
"is the wai ecosystem configured?" rather than "is this a well-run repo?".

After the move:
- `wai way` no longer mentions beads or openspec
- `wai doctor` includes them as workspace ecosystem checks (with `Info` or `Warn`
  status when not present, never `Fail`)

This is a minor visible behavior change: users running `wai way` will no longer
see beads/openspec checks. Users running `wai check` or `wai doctor` will.
Document in the proposal's "What Changes" section.

### Decision: Combined exit code

`wai check` exits 1 if any check across either section has status `Fail`.
`Info` and `Warn` do not trigger a non-zero exit (consistent with `wai doctor`
which only fails on `Fail`, not `Warn`). `wai way` already always exits 0 and
that contract is preserved for its own `run()`.

### Decision: `--way-only` / `--doctor-only` as bool flags (not a subcommand)

These are simple filter flags, not separate commands. They match the pattern
of `--json`, `--fix`, `--status` in other commands. Using them as a subcommand
would require `wai check way` vs `wai check doctor` which adds a grammar level.

The two flags are mutually exclusive (clap `conflicts_with`).

### Decision: JSON output shape

```json
{
  "way": {
    "checks": [...],
    "summary": { "pass": N, "recommendations": N }
  },
  "doctor": {
    "checks": [...],
    "summary": { "pass": N, "warn": N, "fail": N }
  }
}
```

When `--way-only` is set, `"doctor"` key is absent. When `--doctor-only` is
set, `"way"` key is absent. When doctor section is skipped (no workspace),
`"doctor"` key is `null`.

## Risks / Trade-offs

- **Output length**: `wai check` runs all checks from both commands. On a
  healthy repo this is ~20 lines plus headers. Acceptable.
- **Divergence risk**: if `way` or `doctor` are later extended with new checks,
  `check` automatically picks them up via `run_checks()`. No maintenance burden.
- **beads/openspec move**: users who rely on `wai way` to detect beads/openspec
  presence lose that signal. The change is low-impact (these are `Info`-level
  items, never actionable failures) and is offset by `wai check` providing a
  complete view.

## Migration Plan

No migration required. `wai way` and `wai doctor` behavior is preserved.
`wai check` is purely additive. The `wai init` next-steps update is forward-only
(new workspaces get the updated template; existing CLAUDE.md files are not
retroactively patched).

## Open Questions

- Should `wai check --fix` be added in a follow-up? (Deferred — fix semantics
  differ between `way` and `doctor`; tackle separately.)
- Should `wai check` become the default when `wai` is run with no arguments?
  (Deferred — out of scope for this change.)
