# Tasks: Add --fix flag to wai doctor

## Status: COMPLETED

All tasks have been implemented and committed.

## Task Breakdown

### 1. Infrastructure

#### 1.1 Extract sync functions to sync_core module ✓
**Status**: Completed (commit: d5350d7)

Extract `execute_symlink`, `execute_inline`, and `execute_reference` from `src/commands/sync.rs` into `src/sync_core.rs` with `pub(crate)` visibility.

**Files**:
- NEW: `src/sync_core.rs`
- MODIFIED: `src/commands/sync.rs`, `src/main.rs`

**Beads**: Closes wai-bnk

---

### 2. CLI and Data Structures

#### 2.1 Add --fix flag to Doctor CLI ✓
**Status**: Completed (commit: 49fca6d)

Add `--fix` flag to `Doctor` command variant in `src/cli.rs` and update dispatch in `src/commands/mod.rs`.

**Files**:
- MODIFIED: `src/cli.rs`, `src/commands/mod.rs`, `src/commands/doctor.rs`

**Beads**: Closes wai-hcy

#### 2.2 Add fix_fn field to CheckResult ✓
**Status**: Completed (commit: 49fca6d)

Add `fix_fn: Option<Box<dyn FnOnce(&Path) -> Result<()>>>` field to `CheckResult`, annotated with `#[serde(skip)]`. Update all CheckResult construction sites.

**Files**:
- MODIFIED: `src/commands/doctor.rs` (37+ CheckResult constructions)

**Beads**: Closes wai-v3e

---

### 3. Fix Application Logic

#### 3.1 Implement doctor fix application logic ✓
**Status**: Completed (commit: 8c8a751)

Implement core fix application flow in `apply_fixes()`:
- Check safe mode (refuse if --safe)
- Filter to fixable checks
- Confirm with user (unless --yes, --no-input, or --json)
- Apply each fix_fn
- Output results (JSON or human-readable)
- Exit with appropriate code

**Files**:
- MODIFIED: `src/commands/doctor.rs`

**Beads**: Closes wai-hzt

---

### 4. Individual Fix Implementations

#### 4.1 Implement auto-fix for missing directories ✓
**Status**: Completed (commit: 8c8a751)

Attach fix_fn to `check_directories` that creates missing PARA directories.

**Files**:
- MODIFIED: `src/commands/doctor.rs:304-320`

**Beads**: Closes wai-3ug

#### 4.2 Implement auto-fix for stale projections ✓
**Status**: Completed (commit: 8c8a751)

Attach fix_fn to projection check results (symlink, inline, reference strategies) that call appropriate `sync_core` functions.

**Files**:
- MODIFIED: `src/commands/doctor.rs` (check_projection, check_*_strategy functions)

**Beads**: Closes wai-beo

#### 4.3 Implement auto-fix for missing .state files ✓
**Status**: Completed (commit: 8c8a751)

Attach fix_fn to `check_project_state` Warn case (missing .state) that writes `ProjectState::default()`.

**Files**:
- MODIFIED: `src/commands/doctor.rs:882-891`

**Beads**: Closes wai-n1g

#### 4.4 Implement auto-fix for AGENTS.md managed block ✓
**Status**: Completed (commit: 8c8a751)

Attach fix_fn to `check_agent_instructions` that calls `inject_managed_block()` with detected plugins.

**Files**:
- MODIFIED: `src/commands/doctor.rs:1033-1050`

**Beads**: Closes wai-bva

---

### 5. Testing

#### 5.1 Add integration tests for doctor --fix ✓
**Status**: Completed (commit: 5f8ae38)

Add integration tests:
- `doctor_fix_repairs_missing_directories`
- `doctor_fix_skips_confirmation_with_yes`
- `doctor_fix_no_fixable_issues`
- `doctor_fix_repairs_agents_md_block`
- `doctor_fix_skips_corrupted_state`
- `doctor_fix_blocked_by_safe_mode`

**Files**:
- MODIFIED: `tests/integration.rs`

**Beads**: Closes wai-33w

---

## Commits

1. `d5350d7` - refactor(sync): extract sync functions to sync_core module
2. `49fca6d` - feat(doctor): add --fix flag and fix_fn field to CheckResult
3. `8c8a751` - feat(doctor): implement fix application logic
4. `6d0c6d0` - feat(doctor): implement auto-fix for all fixable checks
5. `5f8ae38` - test(doctor): add integration tests for --fix flag

## Dependencies

- Task 1.1 blocks 4.2 (need sync_core for projection fixes)
- Tasks 2.1 and 2.2 block 3.1 (need CLI flag and data structure)
- Task 3.1 blocks all 4.x tasks (need fix application flow)
- All 4.x tasks block 5.1 (need implementations before testing)
