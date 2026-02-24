# Change: Add `wai close` session-end helper command

## Why

Session end is the most neglected step in the workflow: many threads end with "commit and push"
with no handoff, no sync, and no artifact capture. A manual checklist in CLAUDE.md exists but
isn't consistently triggered. `wai close` wraps the whole checklist into one command so the
right thing happens by default.

## What Changes

- Adds a new `wai close` top-level command
- Auto-detects the active project (single project: uses it; multiple: prompts or respects `--project`)
- Delegates handoff creation to the existing `wai handoff create` logic
- Reads `git status --porcelain` and lists uncommitted files (skipped gracefully when git unavailable)
- Prints a next-steps reminder that includes `bd sync --from-main` only when the beads plugin is detected

## Impact

- Affected specs: `cli-core`
- Affected code: `src/cli.rs`, `src/commands/close.rs` (new), `src/commands/mod.rs`
- No breaking changes; existing `wai handoff create` is unchanged
