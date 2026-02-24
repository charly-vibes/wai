# Design: Agent-Aware Backend for `wai why`

## Context

`wai why` is frequently invoked from within Claude Code agent sessions — either by the agent itself as a tool call, or by a human user typing into a Claude Code terminal. In both cases there is already an active LLM with full context. The current backends (API, Ollama, claude-cli) all treat the LLM as an external service to call, ignoring the one that's already present.

## Core Insight: The Delegation Model

The agent backend inverts the normal flow:

| Normal backends         | Agent backend                              |
|-------------------------|--------------------------------------------|
| wai calls LLM           | wai prepares context, parent agent answers |
| Response returned to wai | Response is the agent's next message       |
| wai formats output      | Agent formats output naturally             |
| Requires external service| Requires nothing — agent is already there |

**Why this is the right model inside Claude Code**:
- The agent can see all tool output — the prepared context reaches it automatically via the Bash tool result
- The agent synthesizes the answer in its next response, which is where the user expects it
- No credentials, no subprocess, no network calls from wai
- Works identically whether the caller is a human (typing in the terminal) or an AI agent (calling Bash)

## Decision 1: Detection via `CLAUDECODE` Environment Variable

**What**: Use `CLAUDECODE` env var (non-empty) as the signal that wai is running inside an active Claude Code session.

**Why**:
- Claude Code sets `CLAUDECODE` in child process environments
- It's the same signal the `claude -p` nesting guard uses (`CLAUDECODE` non-empty = nested)
- No other detection method is reliable (no socket, no PID file, no IPC)

**Constraint**: `CLAUDECODE=""` (empty string) is used by the `claude-cli` backend to bypass the nesting guard when spawning `claude -p`. The agent backend must only activate when `CLAUDECODE` is non-empty (the natural session signal), not when it has been explicitly cleared.

**Implementation**:
```rust
fn in_agent_session() -> bool {
    std::env::var("CLAUDECODE").map(|v| !v.is_empty()).unwrap_or(false)
}
```

## Decision 2: Auto-Detection Priority Split by Context

**What**: Split the detection order based on whether wai is inside or outside a Claude Code session.

**Inside** (`CLAUDECODE` non-empty):
1. `ANTHROPIC_API_KEY` → Claude API (user may have both)
2. Agent mode (CLAUDECODE detected) ← new
3. Ollama (local fallback, no nesting issue)
4. Search fallback

**Outside** (`CLAUDECODE` not set or empty):
1. `ANTHROPIC_API_KEY` → Claude API
2. Claude CLI (`claude` binary, no nesting concern)
3. Ollama
4. Search fallback

**Why split**: `claude-cli` should not be auto-selected inside Claude Code (CLAUDECODE bypass is fragile), and agent mode is meaningless outside a Claude Code session (nothing reads the output).

## Decision 3: Output Format

**What**: In agent mode, `wai why` prints a clearly delimited block to stdout containing the full prepared context (the same prompt it would send to an LLM), preceded by a brief status line.

**Format**:
```
  ◆ wai why — agent mode
  ○ Context prepared: 4 artifacts, git history for src/config.rs

[AGENT CONTEXT]
You are an oracle helping understand why code and decisions exist as they do.

# User Question
why was TOML chosen?

# Available Artifacts
...
[/AGENT CONTEXT]
```

**Why delimiters**: The `[AGENT CONTEXT]` / `[/AGENT CONTEXT]` tags allow the parent agent to unambiguously locate the context block in its Bash tool output, even if other text is present. They also make it clear to human readers what the block is.

**Why the same prompt as other backends**: Consistency. The agent backend does not need a simplified format — the parent agent can handle the full structured prompt, and using the same format means the gathering and formatting logic is shared.

## Decision 4: `complete()` Contract for Agent Mode

**What**: `AgentBackend::complete()` prints the context block to stdout and returns `Ok(AGENT_SENTINEL)`. The `run()` function in `why.rs` detects the sentinel and skips normal response formatting. No new trait method is added.

**Why keeping `complete()` in the loop**: Existing error handling and backend-selection code in `run()` stays unchanged. Only the output formatting branch diverges on the sentinel value.

**Alternative considered**: Have `run()` check an `is_agent_mode()` trait method before calling `complete()` and skip the call entirely. Rejected: separating "write context to stdout" from `complete()` would require a new method on the trait, complicating the abstraction with no benefit.

## Decision 5: claude-cli Constraint (Inside vs Outside)

**What**: `claude-cli` backend is valid outside Claude Code sessions but should NOT be auto-selected inside them. Users who explicitly set `[why] llm = "claude-cli"` inside a Claude Code session will get a warning but proceed (explicit overrides auto-detection).

**Why**: Inside a Claude Code session, spawning `claude -p` with `CLAUDECODE=""` is a workaround that depends on the internal nesting check implementation. It's already working today but is inherently fragile. The agent backend is the correct solution for that context.

## Decision 6: Privacy Notice

**What**: Agent mode triggers the same one-time privacy notice as the Claude API backend.

**Why**: Artifacts are still sent to Anthropic — they just flow through the parent Claude Code session rather than a direct API call. The data handling is equivalent from a user privacy perspective.

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| CLAUDECODE removed or renamed in future Claude Code | Low | It's the documented nesting-guard variable; unlikely to change |
| Agent ignores the context block and doesn't answer | Low | The context is clearly formatted; agents naturally respond to Bash output |
| Human user confused by context dump in terminal | Medium | Clear header explains what happened; answer follows in the conversation |
| Context too large for agent's context window | Low | Same truncation logic applies as for other backends |
| Explicit `llm = "agent"` set outside a Claude Code session | Low | System warns and exits 0; user sees the context dump but gets no synthesized answer |
