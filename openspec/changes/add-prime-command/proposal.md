# Change: Add `wai prime` session orientation command

## Why

Agents and users start sessions by running several separate commands to reconstruct
context: `wai status`, `bd ready`, last handoff date, openspec progress. From user
behavior research: 233 `/clear` commands observed, most sessions open with "work" or
"what's the status?" — there is no single command that answers this. `wai prime` wraps
all orientation data into one view.

## What Changes

- Adds a new `wai prime` top-level command
- Auto-detects the active project (same logic as `wai close`)
- Reads the most recent handoff file, extracts date and a one-line summary snippet
- Calls the plugin status hook to collect per-plugin one-line summaries (same mechanism as `wai status`)
- Invokes `bd ready` to surface the highest-priority ready issue as a suggested next action
- Formats everything into a single oriented view

Expected output:

```
◆ wai prime — 2026-02-24
• Project: why-command [review]
• Handoff: 2026-02-23 — 'Completed Phase 9.1 verbosity levels...'
• Beads:   3 open issues (2 ready)
• Spec:    add-why-command: 49/54 (91%)
→ Suggested next: bd show wai-abc
```

The handoff line is omitted when no handoff exists. Plugin lines appear only for detected
plugins. The suggested-next line is omitted when beads is not detected or there are no
ready issues.

## Impact

- Affected specs: `cli-core`
- Affected code: `src/cli.rs`, `src/commands/prime.rs` (new), `src/commands/mod.rs`
- No breaking changes; `wai status` is unchanged
