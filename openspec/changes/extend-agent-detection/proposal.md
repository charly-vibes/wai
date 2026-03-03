# Change: Extend agent detection to WAI_AGENT and CURSOR_AGENT

## Why

`in_agent_session()` currently detects only Claude Code (`CLAUDECODE`). Two gaps
exist:

1. **Other agents go undetected.** Cursor sets `CURSOR_AGENT=1`, and no universal
   override exists for agents without their own env var (Windsurf, Goose, future
   tools). Inside these agents, `wai why` and `wai reflect` spawn an unnecessary
   LLM subprocess instead of delegating to the parent agent.

2. **`wai reflect` agent-mode is implemented but unspecced.** The code already
   returns `AGENT_SENTINEL` and prints `[AGENT CONTEXT]…[/AGENT CONTEXT]` — the
   same mechanism as `wai why` — but the `project-reflection` spec has no
   "Agent Backend" requirement documenting this behaviour.

This change formalises the three-env detection contract and adds the missing
spec for `wai reflect` agent mode.

## What Changes

- **MODIFIED**: `in_agent_session()` checks three env vars in priority order:
  1. `WAI_AGENT` non-empty → agent mode (universal override for any agent)
  2. `CLAUDECODE` non-empty → agent mode (Claude Code)
  3. `CURSOR_AGENT` non-empty → agent mode (Cursor IDE)
- **MODIFIED**: `reasoning-oracle` spec — detection scenarios reference the
  three-env contract instead of `CLAUDECODE`-only.
- **ADDED**: `project-reflection` spec — "Agent Backend" requirement documenting
  `wai reflect` agent mode (context block output + `--inject-content` pattern).

## What Does NOT Change

- Agent-mode output format is unchanged (`[AGENT CONTEXT]…[/AGENT CONTEXT]`).
- Non-agent invocations are unaffected; the priority-ordered check is additive.
- `wai why` behaviour is unchanged: detection logic update is internal to
  `in_agent_session()`, which `wai why` already calls.
- Explicit `[why] llm = "agent"` config continues to work as before.

## Impact

- Affected specs: `reasoning-oracle` (modified), `project-reflection` (modified)
- Affected code: `src/llm.rs` (`in_agent_session()`), inline tests
- All callers of `in_agent_session()` — `AgentBackend::is_available()`,
  `detect_backend()`, `why.rs` hint logic — automatically inherit the new
  detection without further changes.
