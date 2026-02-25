# Change: Fix sync projection strategies and add dry-run

## Why

`wai sync` with `strategy: symlink` silently creates an empty directory at the
target path instead of an actual symlink, making file projections unusable. Missing
parent directories cause additional silent failures with no diagnostic output.

## What Changes

- Fix `execute_symlink` in `sync_core.rs` for file→file projections: currently creates
  a directory at the target path instead of a symlink (directory→directory mirroring
  already works and is preserved)
- Auto-create missing parent directories for all projection targets (`mkdir -p` behaviour)
- Add `--dry-run` flag to `wai sync` that previews operations without making any changes
- Add `strategy: copy` as a fallback for environments where symlinks are unavailable

## Impact

- Affected specs: agent-config-sync
- Affected code: `src/sync_core.rs`, `src/commands/sync.rs`, `src/cli.rs`
- No breaking changes to `.projections.yml` format; `copy` is additive
