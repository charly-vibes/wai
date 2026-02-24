# Design: add-resume-loop

## Problem

`wai prime` needs to know whether the current session is a mid-task resume
(agent cleared context mid-task and is picking up) vs. a fresh start. Without
this signal, prime renders a generic one-line handoff snippet — not enough
information for an agent to resume work without opening the handoff manually.

## Signal Design

### Chosen approach: `.pending-resume` file

A plain text file written by `wai close` to `.wai/projects/<project>/` containing
the relative path to the newly created handoff. `wai prime` reads this file on
startup; if the referenced handoff is dated today it enters RESUMING mode.

**Why a file:**
- Consistent with the existing `.state` file in the same directory
- Readable and manually editable by humans
- Committed to git as part of `.wai/` artifacts (same as handoffs, `.state`)
- Works offline, no external process or DB required
- Survives process restarts, machine reboots, and IDE restarts

**Alternatives rejected:**

| Alternative | Rejected because |
|-------------|-----------------|
| Modify `.state` YAML | Mixes ephemeral signal with durable phase state; breaks state machine semantics |
| Entry in beads DB | Beads is optional and external; wai cannot depend on it |
| Environment variable | Not persistent across shell sessions or machine restarts |
| Separate sessions DB | Over-engineered for a single pointer to a single file |
| Timestamp-only approach | Doesn't identify which handoff to read; prime would have to infer |

### Decay mechanism

The signal is only acted on when the referenced handoff is dated **today**. This
means:

- **Same-day resume** (the primary use case): agent closes, clears, resumes → RESUMING shown
- **Cross-day**: yesterday's signal is ignored → normal `• Handoff:` line shown
- **No manual cleanup required**: the signal silently decays by calendar date

The decay is intentional. Agents often start new work on a new day; forcing them
to dismiss a stale RESUMING prompt would be noise. Human users opening a workspace
days later see normal prime output.

### Signal is not consumed by prime

`wai prime` reads `.pending-resume` but does NOT delete it. Calling prime multiple
times in a session continues to show RESUMING until `wai close` is called again
(which overwrites the signal with the new handoff path). This is the desired
behavior: the agent can re-orient as many times as needed within a session.

## Rendering the RESUMING block

The `## Next Steps` section of the handoff is rendered by prime as follows:

- The section heading `## Next Steps` is stripped of its `##` marker and rendered
  with a trailing colon, indented two spaces: `  Next Steps:`
- Each content line is indented four spaces from the left margin: `    <line>`
- Lines starting with `<!--` (HTML template comments) are skipped
- Blank lines within the section are skipped
- If the section is absent or yields no renderable lines, only the header line
  (`⚡ RESUMING: <date> — '<snippet>'`) is shown, with no indented block

## `.pending-resume` and git

The file lives under `.wai/` which is tracked in git in this project. It will
appear in `wai close`'s uncommitted-files output and should be committed along
with other `.wai/` changes (handoff file, `.state`). This is intentional: the
signal is a workspace artifact, not a temp file.

The stale-decay mechanism means committing `.pending-resume` causes no harm:
the next day's prime session ignores it.

## managed_block.rs

The WAI:START template in `src/managed_block.rs` is updated to add an
"Autonomous Loop" subsection. This is not spec-driven (no capability spec
covers CLAUDE.md template content) — it is implementation guidance baked into
the tool itself. The content appears unconditionally in the template regardless
of which plugins are detected, because the loop pattern applies to all wai users.

## Autonomous Loop Content

The following content is added to `wai_block_content()` in `src/managed_block.rs`,
appended to the "## Ending a Session" section after the existing checklist.
This is the authoritative source; `tasks.md` references this section.

```markdown
### Autonomous Loop

One task per session. The resume loop:

1. `wai prime` — orient (shows ⚡ RESUMING if mid-task)
2. Work on the single task
3. `wai close` — capture state (run this before every `/clear`)
4. `git add <files> && git commit`
5. `/clear` — fresh context

→ Next session: `wai prime` shows RESUMING with exact next steps.

When context reaches ~40%: run `wai close`, then `/clear`.
Do NOT skip `wai close` — it enables resume detection.
```
