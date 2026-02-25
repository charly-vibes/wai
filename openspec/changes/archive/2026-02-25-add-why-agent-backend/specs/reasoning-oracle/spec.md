## MODIFIED Requirements

### Requirement: LLM Backend Selection

The system SHALL support multiple LLM backends via configuration and context-aware auto-detection.

#### Scenario: Auto-detect backend inside a Claude Code session

- **WHEN** user runs `wai why` with no explicit `[why] llm` config
- **AND** the `CLAUDECODE` environment variable is set and non-empty (indicating an active Claude Code session)
- **THEN** the system checks available backends in this priority order:
  1. `ANTHROPIC_API_KEY` env var set → Claude API
  2. `CLAUDECODE` non-empty → Agent mode
  3. `ollama` binary + model available → Ollama
- **AND** uses the first available backend
- **AND** falls back to `wai search` with a warning if none available

**Note**: Agent mode is preferred over Ollama inside a Claude Code session because it requires no external service and uses the already-present LLM without subprocess overhead.

#### Scenario: Auto-detect backend outside a Claude Code session

- **WHEN** user runs `wai why` with no explicit `[why] llm` config
- **AND** the `CLAUDECODE` environment variable is unset or empty
- **THEN** the system checks available backends in this priority order:
  1. `ANTHROPIC_API_KEY` env var set → Claude API
  2. `claude` binary on PATH → Claude CLI
  3. `ollama` binary + model available → Ollama
- **AND** uses the first available backend
- **AND** falls back to `wai search` with a warning if none available

#### Scenario: Explicit Claude API configuration

- **WHEN** `.wai/config.toml` contains `[why] llm = "claude"`
- **THEN** the system uses the Claude API directly
- **AND** reads the API key from `api_key` config field OR `ANTHROPIC_API_KEY` env var
- **AND** fails with diagnostic code `wai::llm::no_api_key` if neither is set

#### Scenario: Explicit Claude CLI configuration

- **WHEN** `.wai/config.toml` contains `[why] llm = "claude-cli"`
- **THEN** the system delegates to the `claude` binary in print mode (`claude -p`)
- **AND** passes the prompt via stdin to handle large artifact contexts
- **AND** fails with a diagnostic if the `claude` binary is not found on PATH
- **AND** warns (but proceeds) if run inside a Claude Code session, since the agent backend is more reliable in that context

#### Scenario: Explicit Ollama configuration

- **WHEN** `.wai/config.toml` contains `[why] llm = "ollama"`
- **THEN** the system uses local Ollama with the configured model
- **AND** fails with installation instructions if the `ollama` binary is not found

#### Scenario: Explicit agent configuration

- **WHEN** `.wai/config.toml` contains `[why] llm = "agent"`
- **THEN** the system uses agent mode regardless of the `CLAUDECODE` environment variable
- **AND** outputs the prepared context block to stdout for the invoking agent to process

#### Scenario: Force fallback

- **WHEN** user runs `wai why <query> --no-llm`
- **THEN** the system skips LLM entirely and uses `wai search` directly
- **AND** displays results in search format

## ADDED Requirements

### Requirement: Agent Backend

When running inside a Claude Code session, the system SHALL delegate reasoning to the invoking agent by outputting the prepared context to stdout rather than calling an external LLM.

#### Scenario: Agent mode activated automatically

- **WHEN** `CLAUDECODE` is set and non-empty
- **AND** no API key is configured
- **THEN** the system prints a brief status line indicating agent mode is active
- **AND** outputs the full prepared context block, delimited by `[AGENT CONTEXT]` / `[/AGENT CONTEXT]` markers
- **AND** exits cleanly — the invoking agent synthesizes the answer in its next response

#### Scenario: Context block format

- **WHEN** outputting context in agent mode
- **THEN** the block uses the same prompt structure as other backends (role definition, user question, artifacts, git context, task instructions)
- **AND** the block is delimited with `[AGENT CONTEXT]` and `[/AGENT CONTEXT]` tags
- **AND** a status line precedes the block: `○ Context prepared: N artifacts[, git history for <file>]` (artifact count MAY be omitted if not directly available from the prompt string)

#### Scenario: Privacy notice precedes context block

- **WHEN** agent mode is used for the first time (privacy notice required)
- **THEN** the privacy notice is printed BEFORE the `[AGENT CONTEXT]` block, never interleaved within it

#### Scenario: Human user in a Claude Code terminal

- **WHEN** a human user runs `wai why "question"` in a Claude Code terminal
- **AND** agent mode is activated
- **THEN** the user sees the status line and context block in their terminal
- **AND** the Claude Code conversation's next response includes the synthesized answer
- **AND** the experience is coherent: the context appears in the terminal, the answer appears in the conversation

#### Scenario: AI agent invoking wai why

- **WHEN** an AI agent (e.g., Claude Code) calls `wai why "question"` via the Bash tool
- **AND** agent mode is activated
- **THEN** the context block appears in the Bash tool result
- **AND** the agent uses the context to synthesize the answer in its next response
- **AND** no additional tool calls or round-trips are required

#### Scenario: Explicit agent mode outside a Claude Code session

- **WHEN** `.wai/config.toml` contains `[why] llm = "agent"`
- **AND** `CLAUDECODE` is unset or empty (not inside a Claude Code session)
- **THEN** the system warns: "agent mode requires an active Claude Code session; no synthesized answer will be produced"
- **AND** still outputs the context block (allowing manual inspection)
- **AND** exits 0 — this is a misconfiguration warning, not a fatal error

#### Scenario: Privacy notice for agent mode

- **WHEN** agent mode is used for the first time
- **AND** the shared `privacy_notice_shown` flag is not set to `true` in `.wai/config.toml`
- **THEN** the system shows the standard one-time privacy notice (before the context block)
- **AND** records acknowledgment in `.wai/config.toml`

**Note**: Agent mode shares the `privacy_notice_shown` flag with other external backends (Claude API). Users who have already acknowledged the notice for Claude API will not see it again for agent mode.

**Rationale**: Artifacts sent via agent mode still flow to Anthropic through the parent Claude Code session. The privacy implications are equivalent to the Claude API backend.

### Requirement: Claude CLI Backend

The system SHALL support delegating `wai why` queries to the `claude` binary in print mode, for users who have Claude Code installed but no direct API key configured.

#### Scenario: Claude CLI backend behaviour

- **WHEN** the Claude CLI backend is selected (auto or explicit)
- **THEN** the system spawns `claude -p` with the prompt passed via stdin
- **AND** captures stdout as the LLM response
- **AND** processes and displays the response identically to the Claude API backend

#### Scenario: Claude CLI is not used inside Claude Code sessions (auto-detect)

- **WHEN** auto-detection runs
- **AND** `CLAUDECODE` is non-empty (inside a Claude Code session)
- **THEN** the Claude CLI backend is NOT selected (agent mode is preferred)

**Rationale**: Inside Claude Code, spawning `claude -p` requires bypassing the nested-session guard (`CLAUDECODE=""`). This is a fragile workaround; agent mode is the correct mechanism for that context.

#### Scenario: No LLM available — remediation guidance mentions agent mode

- **WHEN** no LLM backend is available and the system falls back to `wai search`
- **AND** `CLAUDECODE` is set and non-empty
- **THEN** the fallback warning mentions that running inside a Claude Code session should enable agent mode automatically
- **AND** suggests checking `[why] llm = "agent"` as an explicit override if needed
