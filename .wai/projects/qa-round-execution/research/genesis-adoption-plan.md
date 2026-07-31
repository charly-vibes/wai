# Genesis v0.4.0 Adoption Plan

Date: 2026-07-31

## Overview

Adopt all 9 missing genesis v0.4.0 modules into wai, replacing local reimplementations with shared genesis equivalents. Already using: envelope, config, guide, managed_block, suggestions. Missing: cli, doctor, status, feedback, scaffold, discovery, fixture, aix, suite_linter.

## Current State

- `genesis-vibes = "0.4"` — bumped from 0.2, no features needed (all modules public)
- Custom DoctorCheck system at `src/commands/doctor/mod.rs` (~1,789 lines)
- Custom feedback command at `src/commands/feedback.rs` (~447 lines) — already uses genesis::feedback::scratch, context, redactor, gh
- Custom init scaffold at `src/commands/init.rs` (~265 lines) + `src/workspace.rs` (~350 lines)
- Custom status/prime commands at `src/commands/status.rs` (~584 lines) and `src/commands/prime.rs` (~621 lines)
- Inline completions + version-json in `src/main.rs` and `src/commands/mod.rs`
- Custom `src/output.rs` (~46 lines), `src/json.rs` (~251 lines)

## Phase Status

| Phase | Module | Status |
|-------|--------|--------|
| 1 | cli — completions + version-json | ✅ DONE |
| 2 | feedback — adopt genesis modules | ✅ ALREADY ADOPTED (keeps wai-specific enhancements) |
| 3 | scaffold — init scaffolding | 🟡 PLANNED |
| 4 | doctor — DoctorCheck trait | 🟡 PLANNED (split into sub-phases) |
| 5 | status — StatusContributor | 🟡 PLANNED (depends on Phase 4) |

## Desired End State

- All genesis modules imported and used where applicable
- Zero local reimplementations of genesis patterns remain
- CI passes (build, test, clippy, fmt)
- `llms.txt`/`llm.txt` updated to reflect genesis adoption

## Out of Scope (future P2 tickets)

- Discovery module adoption — `.genesis/tools.toml` manifest
- Fixture module adoption — test scratch environments
- AIX module adoption — currently just `agents_block` helper (minimal surface)
- Suite_linter module adoption — orchestrator pattern, wai doesn't need it yet

## Risks & Mitigations

| Risk | Mitigation | Resolution |
|------|------------|------------|
| Genesis::doctor API diverges from wai's needs | wai's checks are detailed — may need `LintCheck` trait adapter. Fix closures (`FnOnce`) have no genesis equivalent — keep local fix system. | Keep local fix logic; adopt DoctorCheck API for check definition + DoctorReport for output |
| FeedbackKind mismatch | wai has 5 kinds (Bug, Friction, DocsGap, AixGap, Idea); genesis has 4 (bug, feature, question, chore). wai's is more expressive. | Keep wai's FeedbackKind + FeedbackArgs; genesis FeedbackArgs not used |
| Genesis API behavioral changes in patch releases | Pin to "0.4" (semver: patch = backward-compatible fixes only) | Acceptable risk; add integration tests for doctor/status output shapes |
| Phase 4 cross-module coupling | Phase 5 (StatusContributor) depends on Phase 4 (DoctorRunner) via DoctorStatusBridge | Phase 5 must be last; plan dependency explicitly |

---

## Phase 1: cli module — completions + version-json ✅ DONE

### Changes (already applied)

File: `src/main.rs`
- Replaced inline `--version --json` pre-parse (~25 lines) with `genesis::cli::maybe_print_version_json("wai", VERSION)`
- `scratch_timestamp` / `is_leap` retained locally — genesis::feedback::scratch's `timestamp()` is `#[cfg(test)]`-only

File: `src/commands/mod.rs`
- Replaced inline `clap_complete::generate(...)` with `genesis::cli::generate_completions(&mut cmd, shell).into_diagnostic()`

### Verification
- ✅ `cargo build` passes
- ✅ `cargo test` (470 unit + 350 integration) passes
- ✅ `cargo clippy` passes

---

## Phase 2: feedback module — already using genesis modules

### Current state

wai's `src/commands/feedback.rs` already adopts these genesis modules:
- `genesis::feedback::scratch` — `read_last_error`, `write_scratch_best_effort`
- `genesis::feedback::context` — `gather_context`, `format_context_bundle`
- `genesis::feedback::redactor` — `redact`, `reduce_git_remote_url`
- `genesis::feedback::gh` — `CreateIssueOptions`, `create_issue`, `GhResult`

### Why NOT to adopt genesis::feedback::handle_feedback

| Need | wai has | genesis has |
|------|---------|-------------|
| Custom kinds | Bug, Friction, DocsGap, AixGap, Idea | bug, feature, question, chore |
| User-provided title | `--title` CLI arg | Only from scratch/stdin |
| --web mode | Prefilled GitHub URL | Not available |
| --json mode | Envelope output | Not available |
| --no-context | Omit env bundle | Not available |

Per the boundary rule: "If wai needs functionality that is NOT in genesis and NOT duplicated in another tool, keep it local." All wai-specific enhancements are legitimate.

### No changes needed
- Tests: `tests/feedback_test.rs` (8 tests)

---

## Phase 3: scaffold module — genesis Scaffold for init

### Changes

File: `src/workspace.rs` (`ensure_workspace_current`)
- Use `genesis::scaffold::Scaffold` for directory creation + .gitignore setup
- Scaffold handles: PARA dirs, agent-config subdirs, resource subdirs, .gitignore, default configs
- Keep local: PLUGINS.md (conditional), managed block injection, pipeline detection, .projections.yml

File: `src/commands/init.rs`
- Keep wai-specific: project name prompt, plugin detection, git auto-commit, JSON output, re-init logic

### Boundary
```
Scaffold handles:         ensure_workspace_current handles (rest):
- Dir creation            - PLUGINS.md (conditional on detected plugins)
- .gitignore              - Managed block injection (slim + detailed)
- Default configs         - Pipeline detection
                          - Skill detection
                          - .projections.yml
                          - Version stamp sync
```

### Tests
- `tests/init_test.rs` (3 tests)
- Manual: `wai init` in temp dir

### Success
- [ ] `cargo build` passes
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

---

## Phase 4: doctor module — adopt genesis DoctorCheck (sub-phased)

This is the largest change (~1,789 lines). **Split into 3 sub-phases** to manage risk.

### Sub-phase 4a: Adopt DoctorReport for JSON output (low-risk)

File: `src/commands/doctor/mod.rs`
- Use `genesis::doctor::DoctorReport` with `to_envelope()` for JSON output instead of custom `DoctorPayload`
- Use `genesis::doctor::CheckEntry` and `genesis::doctor::DoctorSummary` for serialization
- Keep internal `CheckResult` + `Summary` types; convert to `CheckEntry`/`DoctorSummary` at output boundary
- Keep `render_human` as-is (no genesis equivalent for cliclack rendering)

**Verify:** `wai doctor --json` produces same shape as before

### Sub-phase 4b: Convert checks to DoctorCheck trait (high-risk)

File: `src/commands/doctor/mod.rs`, `src/commands/doctor/checks_basic.rs`, `src/commands/doctor/checks_sync.rs`
- Convert each check function to a struct implementing `genesis::doctor::DoctorCheck`
- Wrap via `LintCheckAdapter` where check signature matches
- For checks with fix closures: keep local `CheckResult` with `fix_fn`, convert to `LintResult` with fix command at output
- Add unit tests for each new `DoctorCheck` impl (TDD: write test before struct)

**Risk:** Fix closures (`FnOnce`) have no genesis equivalent. Keep local fix dispatch; only adopt `DoctorCheck::run` for detection.

### Sub-phase 4c: Adopt DoctorRunner for orchestration (medium-risk)

File: `src/commands/doctor/mod.rs`
- Use `genesis::doctor::DoctorRunner` to orchestrate checks
- `DoctorRunner::run()` returns `DoctorReport` directly
- `health_summary` function still works via `DoctorReport::summary`
- Keep `apply_fixes` dispatch logic (closure-based)

### Tests
- `tests/doctor_test.rs` (9 tests) — must all pass after each sub-phase
- New: Test each `DoctorCheck` impl directly (name, description, run behavior)
- Manual: `wai doctor`, `wai doctor --fix`, `wai doctor --json`

### Verification per sub-phase
- [ ] `cargo build` passes
- [ ] `cargo test` passes (all 9 doctor tests + new unit tests)
- [ ] `cargo clippy` passes

---

## Phase 5: status module — genesis StatusContributor

**Depends on Phase 4** (uses `DoctorStatusBridge` wrapping `DoctorRunner`).

### Changes

File: `src/commands/status.rs`, `src/commands/prime.rs`
- Implement `genesis::status::StatusContributor` for wai
- Register via `StatusBuilder` for structured status output
- Use `DoctorStatusBridge` to surface doctor health automatically in status
- Keep wai-specific: project listing, suggested next steps, pipeline info, openspec section

### Tests
- `tests/status_test.rs` (7 tests)
- `tests/prime_test.rs` (8 tests)

### Success
- [ ] `cargo build` passes
- [ ] `cargo test` passes (all status + prime tests)
- [ ] `cargo clippy` passes

---

## Testing Strategy

1. **Per (sub-)phase**: `cargo build && cargo test && cargo clippy`
2. **TDD for Phase 4b**: Write tests for each `DoctorCheck` before implementing the struct
3. **At end**: Full CI pass + update `llms.txt`/`llm.txt` to reflect genesis adoption

## Test Files Reference

| Phase | Test file(s) | Test count |
|-------|-------------|------------|
| 1 (cli) | `tests/genesis_envelope_migration.rs` | 8 |
| 2 (feedback) | `tests/feedback_test.rs` | 8 |
| 3 (scaffold) | `tests/init_test.rs` | 3 |
| 4 (doctor) | `tests/doctor_test.rs` + new `DoctorCheck` unit tests | 9+ |
| 5 (status) | `tests/status_test.rs`, `tests/prime_test.rs` | 15 |

## Cross-Phase Checklist

- [ ] `llms.txt` updated to mention genesis module adoption
- [ ] `llm.txt` updated similarly
- [ ] No dead code from pre-adoption patterns remains
- [ ] Integration tests (350 tests in `tests/integration.rs`) all pass

## Rollback

Each (sub-)phase is a single commit. Revert individual commits if a phase creates issues.
Phase 4a is lowest-risk and can be landed independently. Phases 4b+4c require a coordinated rollback if issues emerge.