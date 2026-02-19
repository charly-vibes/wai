# Proposal: Add --fix flag to wai doctor

## Problem

`wai doctor` currently only diagnoses issues and suggests manual fixes. Users must copy-paste commands or run `wai init` / `wai sync` manually to fix common problems. This creates friction and slows down the workflow.

## Solution

Add a `--fix` flag to `wai doctor` that automatically fixes issues where safe to do so, following the pattern established by `beads doctor --fix`.

### Auto-fixable Issues

1. **Missing PARA directories** - Create with `fs::create_dir_all()`
2. **Stale projections** - Re-sync using extracted sync_core functions
3. **Missing .state files** - Write `ProjectState::default()` (Warn case only)
4. **Missing AGENTS.md managed block** - Inject using `inject_managed_block()`

### Not Auto-fixable

- Corrupted config.toml (needs `wai init`)
- Missing plugin tools (requires external installation)
- Corrupted .state files (data loss risk)
- Invalid YAML in plugin configs (requires judgment)

## Implementation

See `tasks.md` for detailed breakdown. Key changes:

- Add `--fix` flag to `Doctor` CLI command
- Add `fix_fn: Option<Box<dyn FnOnce(&Path) -> Result<()>>>` to `CheckResult`
- Implement `apply_fixes()` with confirmation flow
- Respect `--yes`, `--no-input`, `--json`, `--safe` flags
- Extract sync functions to `src/sync_core.rs` for reuse

## Design Decisions

- **Extract sync_core module**: Allows doctor to call sync functions without CLI guards
- **FnOnce closures**: Capture fix logic at check time, execute later
- **No dry-run flag**: The existing fix hint output already previews changes
- **Refuse in safe mode**: --fix is a write operation, incompatible with --safe
- **Exit codes**: 0 if all fixes succeeded, 1 if any failed

##Status

**IMPLEMENTED** - This is a retrospective proposal documenting already-implemented work.

## References

- Research: `.wai/projects/doctor-fix-flag/research/2026-02-19-research-wai-doctor-fix-flag-implementation.md`
- Beads pattern: `bd doctor --fix` (external reference)
