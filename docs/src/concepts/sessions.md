# Sessions

Context is expensive. Every time an AI agent starts a new session, it begins with a blank slate — no memory of what was tried, what failed, or what decision was made and why. Humans face the same problem at smaller scale: you close your laptop Friday, and by Monday the thread of reasoning has frayed.

Wai's session lifecycle exists to solve this: capture context at the end of a session, and restore it at the beginning of the next one.

## The Prime → Work → Close Loop

A wai session follows a three-step loop:

```
┌─────────┐      ┌──────┐      ┌───────┐
│  prime   │ ───▶ │ work │ ───▶ │ close │
└─────────┘      └──────┘      └───────┘
     ▲                              │
     └──────────────────────────────┘
```

1. **`wai prime`** — Orient yourself. Prime reads the latest handoff, shows active projects and their phases, surfaces plugin context (git status, open issues), and suggests next steps. If a previous session was interrupted, it detects this automatically.

2. **Work** — Do your actual work: add artifacts, advance phases, write code. Wai stays out of the way.

3. **`wai close`** — Capture state. Close generates a handoff document summarizing the session, writes a `.pending-resume` signal, and shows what to do before ending (close issues, commit, push).

## Resume Detection

When you run `wai prime` after a `wai close`, wai checks for a `.pending-resume` file. If one exists and is less than 12 hours old, prime enters **⚡ RESUMING** mode:

- It reads the linked handoff document
- Extracts the `## Next Steps` section
- Displays them prominently so you (or an agent) can pick up exactly where you left off

This is what makes session continuity work across agent boundaries — the next agent doesn't need to rediscover context; it's handed a precise resumption point.

## What Goes Into a Handoff

A handoff document is a Markdown file stored in `.wai/projects/<name>/handoffs/`. It contains:

- **Session summary** — What was accomplished
- **Current state** — Active phase, open issues, uncommitted changes
- **Plugin context** — Git status, beads issue counts, and other plugin-contributed data (gathered via the `on_handoff_generate` hook)
- **Next steps** — Concrete actions for the next session
- **Decisions made** — Key choices and their rationale

## `wai close` vs `wai handoff create`

Both generate handoff documents, but they serve different purposes:

| Command | When to use |
|---|---|
| `wai close` | End of a session. Generates a handoff *and* writes the `.pending-resume` signal so the next `wai prime` can detect it. |
| `wai handoff create` | Mid-session or explicit handoff to another person/agent. Generates the document without the resume signal. |

Use `wai close` when you're done working. Use `wai handoff create` when you need to hand context to someone else but aren't stopping yet.

## The Autonomous Loop

For AI agents running in autonomous mode, the session loop becomes a tight cycle:

1. `wai prime` — orient (detects ⚡ RESUMING if mid-task)
2. Work on one task
3. `wai close` — capture state
4. `git commit` and push
5. Clear context (e.g., `/clear`)

The next session starts with `wai prime` again, which shows RESUMING with exact next steps. This pattern keeps agents productive across context window boundaries without losing the thread.
