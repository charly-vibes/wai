## ADDED Requirements

### Requirement: Agent Backend for wai reflect

When running inside an agent session, `wai reflect` SHALL output the prepared
reflection context block to stdout — using the same `[AGENT CONTEXT]` /
`[/AGENT CONTEXT]` delimiters as `wai why` — rather than calling an external
LLM. The invoking agent synthesizes the REFLECT content and feeds it back via
`--inject-content`.

#### Scenario: Agent mode activated in wai reflect

- **WHEN** user (or agent) runs `wai reflect`
- **AND** `in_agent_session()` returns `true`
- **AND** no API key is configured
- **THEN** `wai reflect` selects the Agent backend via `detect_backend()`
- **AND** outputs the prepared reflection context block delimited by
  `[AGENT CONTEXT]` / `[/AGENT CONTEXT]` markers
- **AND** prints a status line: `◆ Agent mode — context sent to agent.`
- **AND** prints a follow-up instruction:
  `Once the agent provides the REFLECT content, run: wai reflect --inject-content '<content>'`
- **AND** exits 0 without writing any resource file

#### Scenario: Agent provides reflect content via --inject-content

- **WHEN** the invoking agent synthesizes the REFLECT content from the context block
- **AND** runs `wai reflect --inject-content '<synthesized content>'`
- **THEN** `wai reflect` skips the LLM call and uses the injected content directly
- **AND** writes the content to `.wai/resources/reflections/<date>-<project>.md`
- **AND** completes normally (updates metadata, prints resource file path)

#### Scenario: Detection uses same three-env contract

- **WHEN** determining whether agent mode should activate for `wai reflect`
- **THEN** the system uses `in_agent_session()` identically to `wai why`:
  `WAI_AGENT`, then `CLAUDECODE`, then `CURSOR_AGENT`
- **AND** the agent backend is selected by `detect_backend()` using the shared
  `[why]` LLM config section

#### Scenario: Non-agent invocation unchanged

- **WHEN** `in_agent_session()` returns `false`
- **THEN** `wai reflect` behaves exactly as before: calls the configured LLM
  backend and writes the result directly to the resource file
- **AND** no `[AGENT CONTEXT]` block is emitted
