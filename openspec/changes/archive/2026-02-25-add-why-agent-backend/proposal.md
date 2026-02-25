# Change: Agent-Aware Backend for `wai why`

## Why

`wai why` currently supports two LLM backends: the Claude API (requires an API key) and Ollama (requires a local install). In practice, the most common context for running `wai why` is *inside a Claude Code session* — where an LLM is already present and active. Trying to reach a separate LLM from that context is redundant and fragile.

Two gaps exist:

1. **`claude-cli` backend is unspecced.** A `claude -p` subprocess backend was implemented as a workaround for the no-API-key case, but it has a fundamental problem: `claude -p` detects the nested session via `CLAUDECODE` and refuses to launch. The workaround (`CLAUDECODE=""`) is fragile and may break with future Claude Code releases.

2. **No agent-aware mode.** When wai runs inside a Claude Code agent session, the right answer is not to spawn another LLM process — it's to delegate the reasoning back to the invoking agent. The agent already has context, memory, and reasoning capability. The optimal path is: wai prepares the context, outputs it to stdout, and the parent agent synthesizes the answer in its next response. No subprocess, no API key, no nesting hacks.

## What Changes

- **NEW backend**: `agent` — detects `CLAUDECODE` env var (non-empty = Claude Code session), gathers context, writes the prepared prompt to stdout, and exits. The invoking agent reads the context from its Bash tool output and synthesizes the answer in its own response.

- **RETROACTIVE spec**: `claude-cli` backend — the `claude -p` subprocess backend is now formally specified, with its constraint (not for use when already inside a Claude Code session) documented.

- **UPDATED detection order**: the auto-detection logic now distinguishes between "inside a Claude Code session" and "outside one":
  - **Inside** (`CLAUDECODE` non-empty): API key → Agent → Ollama
  - **Outside**: API key → Claude CLI → Ollama

- **MODIFIED**: `reasoning-oracle` spec to cover the new backend and updated priority logic.

## Impact

- **Affected specs**: `reasoning-oracle` (MODIFIED + ADDED)
- **Affected code**: `src/llm.rs` (new `AgentBackend`), `src/commands/why.rs` (agent-mode output path), `tests/integration/`
- **Depends on**: `add-why-command` — the `reasoning-oracle` spec this change modifies is introduced by `add-why-command` and must be deployed before this change is archived
- **Breaking changes**: None — `claude-cli` continues to work outside Claude Code sessions; agent mode is auto-selected inside them
- **User impact**:
  - Claude Code users get `wai why` for free with zero configuration
  - No API key or subprocess required when inside an agent session
  - Human users see the gathered context in the terminal; their Claude Code session answers in the conversation
  - Agent callers get the context in Bash tool output and answer inline
