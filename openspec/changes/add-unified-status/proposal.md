# Change: Add unified implementation status to `wai status`

## Why

The `wai` tool has three tracking systems (OpenSpec specs/changes, WAI project state, Beads issues) but `wai status` only shows project phases and plugin hook output. There is no single view that surfaces what capabilities are implemented vs. what changes are proposed, or how far along proposed changes are. Users must manually inspect `openspec/` directories to understand implementation status.

## What Changes

- Add an OpenSpec section to `wai status` output showing spec counts and active change proposals with task completion progress
- Thread the existing `--verbose` global flag into the status command for progressive detail
- Default view: summary counts + active changes with completion ratios
- Verbose view (`-v`): additionally lists all spec names and per-section task breakdown for each change

## Impact

- Affected specs: cli-core (Status Command requirement)
- Affected code: `src/commands/status.rs`, `src/commands/mod.rs`, new `src/openspec.rs` module
