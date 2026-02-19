## 1. OpenSpec Reader Module

- [x] 1.1 Create `src/openspec.rs` with `OpenSpecStatus` and `ChangeStatus` structs
- [x] 1.2 Implement `read_status(project_root: &Path) -> Option<OpenSpecStatus>` to scan `openspec/specs/` for spec names
- [x] 1.3 Implement scanning `openspec/changes/` for active changes (excluding `archive/`)
- [x] 1.4 Implement `tasks.md` parser counting `- [ ]` vs `- [x]` lines, with per-section breakdown

## 2. Wire Verbosity into Status Command

- [x] 2.1 Update `status::run()` signature to accept `verbose: u8`
- [x] 2.2 Update dispatch in `src/commands/mod.rs` to pass `cli.verbose` to `status::run()`
- [x] 2.3 Register `mod openspec` in `src/main.rs`

## 3. Status Output

- [x] 3.1 Add OpenSpec section to `status::run()` default output: summary counts + active changes with completion ratios
- [x] 3.2 Add verbose output path (`-v`): list all spec names and per-section task breakdown
- [x] 3.3 Ensure graceful skip when `openspec/` directory doesn't exist

## 4. Testing

- [x] 4.1 Add unit tests for `tasks.md` parser (empty, all checked, mixed, multi-section)
- [x] 4.2 Add integration test: `wai status` with `openspec/` present shows spec counts
- [x] 4.3 Add integration test: `wai status` without `openspec/` omits section gracefully
- [x] 4.4 Add integration test: `wai status -v` shows detailed breakdown
