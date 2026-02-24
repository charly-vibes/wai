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
- Calls `plugin::run_hooks(project_root, "on_status")` and `openspec::read_status(project_root)` — the same hooks used by `wai status` — to collect per-plugin one-line summaries; no new plugin plumbing required
- Invokes `bd ready --json` and uses the first result's `id` as the suggested next action
- Formats everything into a single oriented view

The phase is shown inline in the Project bullet as `[phase]`. There is no separate Phase
bullet. Expected output:

```
◆ wai prime — 2026-02-24
• Project: why-command [review]
• Handoff: 2026-02-23 — 'Completed Phase 9.1 verbosity levels...'
• Beads:   3 open issues (2 ready)
• Spec:    add-why-command: 49/54 (91%)
→ Suggested next: bd show wai-abc
```

Plugin summary lines (Beads, Spec, etc.) are produced by the plugin status hooks; their
format is determined by each plugin. The handoff line is omitted when no handoff exists.
The suggested-next line is omitted when beads is not detected or there are no ready issues.

## Impact

- Affected specs: `cli-core`
- Affected code: `src/cli.rs`, `src/commands/prime.rs` (new), `src/commands/mod.rs`
- No breaking changes; `wai status` is unchanged
