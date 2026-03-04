# Change: Add `wai check` unified health command

## Why

Users running `wai` in a new repo face two overlapping commands with no clear
guidance on which to reach for. `wai way` checks repo hygiene; `wai doctor`
checks workspace health. The friction-analysis project (`2026-03-04-design-unifying-wai-way-and-wai-doctor.md`) identified this as a persistent confusion
point: both commands run checks, both print fix suggestions, yet neither tells
you about the other's domain until you read the help text carefully.

The cross-reference added in commit 73286cd improved discoverability but did not
eliminate the decision cost. The fix is a single entry point that runs both.

## What Changes

- **ADDED**: `wai check` subcommand — runs `wai way` checks then `wai doctor`
  checks in sequence, under distinct section headers. The combined exit code
  is non-zero if either section has failures.
- **ADDED**: `--way-only` flag on `wai check` — runs only repo hygiene checks.
- **ADDED**: `--doctor-only` flag on `wai check` — runs only workspace health
  checks.
- **ADDED**: `--json` output for `wai check` — structured JSON combining both
  check sections under separate keys.
- **REFACTORED** (internal, non-breaking): `way.rs` and `doctor.rs` expose
  `run_checks() -> Vec<CheckResult>` functions so `check.rs` can call them
  without duplicating logic. Presentation (rendering, exit code) stays in each
  command's own `run()`.
- **REFACTORED** (internal, non-breaking): `CheckResult` is unified into a
  shared type (superset of Pass/Info/Warn/Fail) usable by both `way`, `doctor`,
  and `check`.
- **MOVED** (internal, minor behavior change): beads and openspec presence
  checks move from `wai way` to `wai doctor`. These checks test for wai
  ecosystem tooling, which is more at home in workspace health than repo hygiene.
- **UPDATED**: `wai init` next-steps text — mentions `wai check` as the
  recommended first verification command after initialization.
- **UPDATED**: `wai way` and `wai doctor` help text — cross-reference `wai check`
  as the umbrella command.

## What Does NOT Change

- `wai way` and `wai doctor` are **preserved with no breaking changes**. Their
  flags, output format, exit codes, and behavior are unchanged.
- The beads/openspec check *results* are unchanged; only which command surfaces
  them moves (from `way` to `doctor`).
- Fix mode (`wai doctor --fix`, `wai way --fix skills`) is not exposed through
  `wai check` — users run the individual commands for fix operations.

## Impact

- Affected specs: `check-command` (new capability), `cli-core` (modified — adds
  `wai check` to Command Structure requirement)
- Affected code:
  - `src/cli.rs` — add `Check` variant to `Commands` enum
  - `src/commands/check.rs` — new file
  - `src/commands/way.rs` — extract `run_checks()`, move beads/openspec checks
    to doctor
  - `src/commands/doctor.rs` — extract `run_checks()`, add beads/openspec checks
  - `src/commands/mod.rs` — dispatch for new `check` command
  - Wherever `CheckResult` / `Status` types live — unify into shared module or
    expose from each command
  - `wai init` managed block template — add `wai check` to Quick Reference
