## MODIFIED Requirements

### Requirement: LLM Backend Selection

The system SHALL support multiple LLM backends via configuration and
context-aware auto-detection.

#### Scenario: Auto-detect backend inside an agent session

- **WHEN** user runs `wai why` with no explicit `[why] llm` config
- **AND** `in_agent_session()` returns `true` — i.e. at least one of
  `WAI_AGENT`, `CLAUDECODE`, or `CURSOR_AGENT` is set and non-empty
- **THEN** the system checks available backends in this priority order:
  1. `ANTHROPIC_API_KEY` env var set → Claude API
  2. `in_agent_session()` true → Agent mode
  3. `ollama` binary + model available → Ollama
- **AND** uses the first available backend
- **AND** falls back to `wai search` with a warning if none available

**Note**: Agent mode is preferred over Ollama inside any agent session because
it requires no external service and uses the already-present LLM without
subprocess overhead.

#### Scenario: Explicit agent mode outside an agent session

- **WHEN** `.wai/config.toml` contains `[why] llm = "agent"`
- **AND** `in_agent_session()` returns `false` (no supported agent env var set)
- **THEN** the system warns: "agent mode requires an active agent session; no
  synthesized answer will be produced"
- **AND** still outputs the context block (allowing manual inspection)
- **AND** exits 0 — this is a misconfiguration warning, not a fatal error

### Requirement: Agent Backend

When running inside an agent session, the system SHALL delegate reasoning to
the invoking agent by outputting the prepared context to stdout rather than
calling an external LLM.

#### Scenario: Agent mode activated automatically

- **WHEN** `in_agent_session()` returns `true`
- **AND** no API key is configured
- **THEN** the system prints a brief status line indicating agent mode is active
- **AND** outputs the full prepared context block, delimited by
  `[AGENT CONTEXT]` / `[/AGENT CONTEXT]` markers
- **AND** exits cleanly — the invoking agent synthesizes the answer in its next
  response

#### Scenario: Detection env var priority

- **WHEN** determining whether agent mode should activate
- **THEN** the system checks env vars in this order:
  1. `WAI_AGENT` non-empty → agent mode (universal override)
  2. `CLAUDECODE` non-empty → agent mode (Claude Code)
  3. `CURSOR_AGENT` non-empty → agent mode (Cursor IDE)
- **AND** the first non-empty match activates agent mode
- **AND** all other unset env vars do not affect the result
