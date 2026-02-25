## 1. Fix symlink strategy

- [ ] 1.1 Rewrite `execute_symlink` in `src/sync_core.rs` to create a real filesystem
        symlink at `target` pointing to `source`; detect file vs. directory source
- [ ] 1.2 Add `mkdir -p` (via `create_dir_all`) for the target's parent directory
        before creating the symlink
- [ ] 1.3 Add unit tests for symlink strategy: file-to-file, directory-to-directory,
        missing parent, pre-existing target

## 2. Add parent-directory auto-creation for all strategies

- [ ] 2.1 Extract `ensure_parent_dirs(target)` helper in `src/sync_core.rs`
- [ ] 2.2 Call it at the start of `execute_inline` and `execute_reference`
        (already needed; symlink benefits from same helper)
- [ ] 2.3 Add tests confirming no error when parent directory is absent

## 3. Add copy strategy

- [ ] 3.1 Implement `execute_copy` in `src/sync_core.rs` (copies source file to target,
        overwrites if present, creates parent dirs)
- [ ] 3.2 Wire `"copy"` in the strategy dispatch in `src/commands/sync.rs`
- [ ] 3.3 Add unit tests for copy strategy

## 4. Add --dry-run flag

- [ ] 4.1 Add `--dry-run` bool flag to `Commands::Sync` in `src/cli.rs`
- [ ] 4.2 Pass flag through to `src/commands/sync.rs`; skip `execute_*` calls when
        `dry_run = true`, printing the operations that would occur instead
- [ ] 4.3 Add integration test: `wai sync --dry-run` must not create any files

## 5. Documentation

- [ ] 5.1 Update `wai sync --help` to document `--dry-run` and `--status` flags
- [ ] 5.2 Document `strategy: copy` option in `wai sync --help` or inline help text
