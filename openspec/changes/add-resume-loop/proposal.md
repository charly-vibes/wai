# Change: Add resume loop — `.pending-resume` signal and `wai prime` resume detection

## Why

Autonomous agents that clear context mid-task have no structured way to resume where
they left off. `wai close` already creates a handoff, and `wai prime` already surfaces
the latest handoff — but `prime` treats all sessions identically: it shows a date and
an 80-character snippet regardless of whether the previous session ended cleanly or
was interrupted mid-task.

The missing piece: a signal that tells `wai prime` "this session was interrupted —
here are the exact next steps" so an agent can resume immediately after `/clear`
without manually opening the handoff file.

Observed workflow failure mode: agent hits ~40% context, runs `wai close`, runs
`/clear`, new session starts, runs `wai prime` — sees "Handoff: 2026-02-24 —
'Implementing phase transition logic...'" — but has no idea what the actual next
steps are without opening the handoff manually. The orientation command fails its
core job in the one scenario where it matters most.

## What Changes

- **`wai close`** writes a `.pending-resume` file to the project directory after
  every successful handoff creation. The file contains the relative path to the
  new handoff (e.g. `handoffs/2026-02-24-session-end.md`). It is overwritten on
  every subsequent `wai close` call, so it always points to the most recent session
  end.

- **`wai prime`** checks for `.pending-resume` before rendering:
  - If the file exists and the referenced handoff is dated **today**: renders a
    `⚡ RESUMING` block showing the handoff's `## Next Steps` section immediately
    before the regular plugin status lines.
  - If the handoff is dated before today: ignores the signal and renders the normal
    `• Handoff:` line. This prevents confusing human users who start a session days
    after their last `wai close`.
  - If the file is absent or the referenced handoff is missing: renders normally.
  - `wai prime` does NOT modify or delete `.pending-resume` after reading it.

- **`src/managed_block.rs`** gains an "Autonomous Loop" subsection in
  `wai_block_content()`, appended to the "## Ending a Session" section. This is
  not spec-driven — it is implementation guidance baked into the template. See
  `design.md § Autonomous Loop Content` for the verbatim text.

## Expected Output

Normal session (no `.pending-resume`, or stale signal):
```
◆ wai prime — 2026-02-24
• Project: why-command [review]
• Handoff: 2026-02-23 — 'Completed Phase 9.1 verbosity levels...'
• Beads:   3 open issues (2 ready)
• Spec:    add-why-command: 49/54 (91%)
→ Suggested next: bd show wai-abc
```

Resume session (`.pending-resume` exists, handoff dated today):
```
◆ wai prime — 2026-02-24
• Project: why-command [review]
⚡ RESUMING: 2026-02-24 — 'Implementing phase transition logic...'
  Next Steps:
    1. Finish state machine in src/state.rs
    2. Write tests for phase transitions
    3. Run openspec validate --strict
• Beads:   3 open issues (2 ready)
• Spec:    add-why-command: 49/54 (91%)
→ Suggested next: bd show wai-abc
```

Note: `  Next Steps:` is indented two spaces; each item is indented four spaces.
The `## Next Steps` heading marker is stripped and a colon is appended when rendered.

## Impact

- Affected specs: `cli-core`, `handoff-system`
- Affected code: `src/commands/close.rs`, `src/commands/prime.rs`,
  `src/managed_block.rs`
- No breaking changes; all new behavior is additive. Existing `wai close` and
  `wai prime` output is unchanged when `.pending-resume` is absent.
- The `.pending-resume` file lives in `.wai/projects/<project>/` alongside `.state`.
  It is a tracked workspace artifact and will appear in `wai close`'s uncommitted-
  files output; it should be committed with other `.wai/` changes. The stale-decay
  mechanism (same-day only) ensures committing it causes no harm.
- See `design.md` for signal design rationale and rendering rules.
