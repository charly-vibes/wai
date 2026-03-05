# Agent Detection Mechanisms for wai Agent-Aware Mode

## Context

Research for wai-q2mf. Prerequisite for wai-mq7j (feat: agent-aware mode — suppress LLM calls when inside Claude Code).

The goal: detect reliably when wai is running inside an LLM agent so it can skip spawning a nested LLM subprocess.

---

## Existing Implementation

`src/llm.rs:212` already has `in_agent_session()`:

```rust
pub fn in_agent_session() -> bool {
    std::env::var("CLAUDECODE")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}
```

It checks `CLAUDECODE` (non-empty = in agent). Confirmed: Claude Code sets `CLAUDECODE=1` in every subprocess it spawns. The existing `ClaudeCliClient` clears it to `""` before spawning a nested `claude` process, preventing infinite nesting.

This function exists but is **not yet used to suppress LLM calls in `wai why` / `wai reflect`** — that's what wai-mq7j will implement.

---

## Environment Variables by Agent

### Claude Code (confirmed empirically)
- `CLAUDECODE=1` — primary signal, non-empty when inside agent
- `CLAUDE_CODE_ENTRYPOINT=cli` — secondary; indicates entrypoint type

### Cursor IDE
- `CURSOR_AGENT=1` — set when Cursor Agent is running terminal commands
- `CURSOR_CLI` — set in integrated terminal
- Source: Cursor community forum (add-agent-1-environment-variable-for-composer-runs)

### Windsurf (Codeium)
- No known standardized env var for agent detection found in documentation
- Uses a dedicated zsh shell for Cascade terminal commands; no detection signal published

### GitHub Copilot
- No env var currently set; open VS Code feature request (microsoft/vscode#265446) proposes `VSCODE_COPILOT_TERMINAL=1` or `COPILOT_TERMINAL=1`
- Not yet implemented as of 2026-03

### Goose (Block)
- `GOOSE_SESSION_ID` observed in env (e.g. `GOOSE_SESSION_ID=20260228_2`) — presence indicates a Goose session

---

## Process Tree Inspection (Linux)

Empirically verified by walking /proc ancestry from the tool process:

```
PID 1836566: zsh          <- tool subprocess shell
PID 1107640: claude       <- Claude Code process
PID 277313:  nu           <- user shell
PID 14197:   ptyxis-agent <- terminal emulator
```

Reading `/proc/PPID/comm` up the chain would detect `claude`, `cursor`, `windsurf` etc. by binary name.

Tradeoffs:
- Works on Linux; not portable to macOS (no /proc, would need sysctl + proc_pidpath)
- Fragile: process name can be anything; binary rename defeats it
- Adds latency (multiple /proc reads or ps calls per invocation)
- Unnecessary when env vars are available

**Decision: process inspection is a fallback of last resort.**

---

## Explicit --agent Flag

Add a `--agent` flag to wai commands, or a `WAI_AGENT=1` env var, that agents without a known env var can set explicitly.

Tradeoffs:
- Requires callers to opt in; easy to forget for flag approach
- Env var approach (`WAI_AGENT=1`) is a clean universal override
- Useful for Windsurf and future agents without their own signals

---

## Output Format / TTY Detection

Checking `isatty()` on stdout is not useful — agent terminals often are TTYs (Claude Code uses a PTY).

---

## Recommendations

### Primary: CLAUDECODE env var (already implemented)
`in_agent_session()` is the right approach. Already coded and tested.

### Extension: WAI_AGENT and CURSOR_AGENT
Extend `in_agent_session()` to also check:
1. `WAI_AGENT` non-empty → agent mode (universal override)
2. `CLAUDECODE` non-empty → agent mode (Claude Code)
3. `CURSOR_AGENT` non-empty → agent mode (Cursor)
4. Otherwise → normal mode

### Avoid: process tree inspection
Too fragile and platform-specific.

---

## Decision for wai-mq7j

Use `in_agent_session()` as-is for Claude Code, extended to check `WAI_AGENT` and `CURSOR_AGENT`.

The openspec for wai-l7dw should document these three env vars as the official detection contract.

When agent mode is detected in `wai why`/`wai reflect`: skip LLM subprocess, output raw artifact context in a format the parent agent can synthesize directly.

