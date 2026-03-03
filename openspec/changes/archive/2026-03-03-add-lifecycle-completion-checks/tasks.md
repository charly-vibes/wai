## Phase 1: Data Model

- [x] 1.1 Add `phase_started: DateTime<Utc>` field to `ProjectContext` in
  `src/workflows.rs`
- [x] 1.2 Update `scan_project` to populate `phase_started` from
  `state.history.last().map(|e| e.started).unwrap_or(Utc::now())`
- [x] 1.3 Export `pub const STALE_PHASE_DAYS: i64 = 14` from `workflows.rs`
- [x] 1.4 Update all `ProjectContext` constructions in unit tests to include
  `phase_started` (use `Utc::now()` for non-staleness tests)

## Phase 2: Pattern Detection

- [x] 2.1 Add `StalePhase { days: i64 }` variant to `WorkflowPattern` enum
- [x] 2.2 Add `LooksComplete` variant to `WorkflowPattern` enum
- [x] 2.3 Implement `StalePhase` detection in `detect_patterns`:
  - Skip if `ctx.phase == Phase::Archive`
  - Compute `days = (Utc::now() - ctx.phase_started).num_days()`
  - If `days > STALE_PHASE_DAYS`, push `StalePhase { days }` with suggestions:
    - `wai phase next` — advance phase
    - `wai move <name> archives` — abandon / archive
- [x] 2.4 Implement `LooksComplete` detection in `detect_patterns`:
  - Condition: `ctx.phase == Phase::Review && ctx.handoff_count >= 1`
  - Push `LooksComplete` with suggestions:
    - `wai move <name> archives` — archive it
    - `wai phase next` — advance to archive phase

## Phase 3: Unit Tests

- [x] 3.1 `stale_phase_detected_after_threshold`: `phase_started = Utc::now() -
  Duration::days(STALE_PHASE_DAYS + 1)`, any non-archive phase →
  `StalePhase` present
- [x] 3.2 `stale_phase_not_detected_within_threshold`: `phase_started =
  Utc::now() - Duration::days(STALE_PHASE_DAYS - 1)` → no `StalePhase`
- [x] 3.3 `stale_phase_not_detected_for_archive`: archive phase +
  old `phase_started` → no `StalePhase`
- [x] 3.4 `looks_complete_detected_in_review_with_handoff`: `phase == Review`,
  `handoff_count = 1` → `LooksComplete` present
- [x] 3.5 `looks_complete_not_detected_without_handoff`: `phase == Review`,
  `handoff_count = 0` → no `LooksComplete`
- [x] 3.6 `looks_complete_not_detected_outside_review`: `phase == Implement`,
  `handoff_count = 2` → no `LooksComplete`
- [x] 3.7 `stale_phase_suggestions_include_archive_and_advance`: assert both
  `wai phase next` and `wai move` appear in `StalePhase` suggestions
- [x] 3.8 `looks_complete_suggestions_include_archive_and_advance`: assert both
  `wai move` and `wai phase next` appear in `LooksComplete` suggestions

## Phase 4: Integration Tests

- [x] 4.1 `status_flags_stale_project`: create a project, mutate `.state` to
  set `phase_started` >14 days in the past, run `wai status`, assert output
  contains stale-phase suggestion text
- [x] 4.2 `status_does_not_flag_recent_project`: same setup but `phase_started`
  within threshold → no stale suggestion
- [x] 4.3 `status_flags_complete_review_project`: create a project in review
  phase with a handoff file, run `wai status`, assert output contains
  completion-readiness suggestion
- [x] 4.4 `status_json_includes_stale_suggestion`: `wai status --json` with
  stale project → `suggestions` array contains stale-phase entry
- [x] 4.5 `status_json_includes_complete_suggestion`: `wai status --json` with
  looks-complete project → `suggestions` array contains completion entry
