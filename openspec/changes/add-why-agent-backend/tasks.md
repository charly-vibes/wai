## Phase 1: Agent Detection

- [ ] 1.1 Add `in_agent_session() -> bool` to `llm.rs` — returns true when `CLAUDECODE` is set and non-empty
- [ ] 1.2 Unit test: `CLAUDECODE=1` → true; `CLAUDECODE=` (empty) → false; unset → false
- [ ] 1.3 Unit test: `CLAUDECODE=""` (the claude-cli bypass value) is treated as false

## Phase 2: AgentBackend Implementation

- [ ] 2.1 Add `AgentBackend` struct to `llm.rs` (no fields)
- [ ] 2.2 Implement `LlmClient` for `AgentBackend`:
  - `is_available()` → `in_agent_session()`
  - `name()` → `"Agent"`
  - `model_id()` → `"agent"`
  - `complete(prompt)` → prints `[AGENT CONTEXT]…[/AGENT CONTEXT]` block to stdout, returns `Ok(AGENT_SENTINEL)`
- [ ] 2.3 Add `AGENT_SENTINEL` constant (unique string `wai::agent-mode`) recognized by `run()` in `why.rs`
- [ ] 2.4 Unit test: `AgentBackend::complete()` writes context to stdout and returns sentinel
- [ ] 2.5 Unit test: `AgentBackend::is_available()` matches `in_agent_session()`

## Phase 3: Detection Priority Update

- [ ] 3.1 Update `detect_backend()` in `llm.rs`:
  - If `CLAUDECODE` non-empty: priority = Claude API → Agent → Ollama
  - If `CLAUDECODE` empty/unset: priority = Claude API → Claude CLI → Ollama
- [ ] 3.2 Add `Some("agent")` explicit branch to the `match cfg.llm.as_deref()` block
- [ ] 3.3 Unit test: `CLAUDECODE=1`, no API key → agent backend selected
- [ ] 3.4 Unit test: `CLAUDECODE` unset, no API key, claude binary present → claude-cli selected
- [ ] 3.5 Unit test: `CLAUDECODE=1`, API key present → Claude API selected (API key takes precedence)
- [ ] 3.6 Unit test: explicit `llm = "agent"` config → agent backend regardless of CLAUDECODE

## Phase 4: Output Path in `why.rs`

- [ ] 4.1 In `run()`: after `backend.complete(&prompt)`, detect `AGENT_SENTINEL` response
- [ ] 4.2 When sentinel detected: print status line (`○ Context sent to your agent`) and return `Ok(())`
- [ ] 4.3 Normal response path unchanged
- [ ] 4.4 Unit test: sentinel response produces clean exit with status line, no LLM output formatting

## Phase 5: Agent Mode Output Format

- [ ] 5.1 In `AgentBackend::complete()`, print:
  ```
    ◆ wai why — agent mode
    ○ Context prepared: N artifacts[, git history for <file>]

  [AGENT CONTEXT]
  <prompt>
  [/AGENT CONTEXT]
  ```
- [ ] 5.2 Status line counts artifacts from the prompt (or take `GatheredContext` as param — see note)
- [ ] 5.3 Unit test: output includes `[AGENT CONTEXT]` and `[/AGENT CONTEXT]` delimiters
- [ ] 5.4 Unit test: output includes the prompt content between delimiters

**Note**: `complete(prompt: &str)` receives the already-built prompt string. Artifact count is not directly available; either parse it from the prompt or restructure to pass `&GatheredContext` to agent mode separately. Prefer the simpler approach: derive count from the prompt or omit it from the status line.

## Phase 6: Privacy Notice

- [ ] 6.1 Add `"Agent"` to `is_external_backend()` in `why.rs` (already done — confirm)
- [ ] 6.2 Verify privacy notice is shown on first agent-mode use and suppressed thereafter
- [ ] 6.3 Integration test: agent mode with `privacy_notice_shown = false` → notice shown

## Phase 7: Fallback Message Update

- [ ] 7.1 When falling back to search inside a Claude Code session, update warning to mention agent mode
- [ ] 7.2 Unit test: fallback message inside CLAUDECODE session mentions agent mode

## Phase 8: Integration Tests

- [ ] 8.1 Integration test: `CLAUDECODE=1`, no API key → wai why prints `[AGENT CONTEXT]` block to stdout, exits 0
- [ ] 8.2 Integration test: `CLAUDECODE=` (empty), no API key, no claude binary → falls back to search (claude-cli not selected)
- [ ] 8.3 Integration test: explicit `[why] llm = "agent"` → agent mode regardless of CLAUDECODE

## Phase 9: Documentation

- [ ] 9.1 Update `wai why --help` to document `agent` as a valid `llm` config value
- [ ] 9.2 Add `claude-cli` to `wai why --help` config documentation
- [ ] 9.3 Add detection priority note to help text (inside vs outside Claude Code sessions)
