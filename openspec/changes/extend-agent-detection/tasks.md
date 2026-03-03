## 1. Extend `in_agent_session()` (`src/llm.rs`)

- [ ] 1.1 Update `in_agent_session()` to check env vars in priority order:
  `WAI_AGENT`, then `CLAUDECODE`, then `CURSOR_AGENT` — return `true` if any
  is non-empty
- [ ] 1.2 Update the doc-comment on `in_agent_session()` to list all three vars
- [ ] 1.3 Update inline tests: add cases for `WAI_AGENT=1` and `CURSOR_AGENT=1`
  triggering agent mode; verify `CLAUDECODE` still works; verify all unset →
  non-agent
- [ ] 1.4 Update `detect_backend()` doc-comment and `src/cli.rs` help text to
  list all three detection env vars

## 2. Tests

- [ ] 2.1 Unit test: `in_agent_session()` returns `true` when only `WAI_AGENT=1`
- [ ] 2.2 Unit test: `in_agent_session()` returns `true` when only `CURSOR_AGENT=1`
- [ ] 2.3 Unit test: `in_agent_session()` returns `false` when all three are unset
- [ ] 2.4 Unit test: `in_agent_session()` returns `false` when all three are empty
  string
- [ ] 2.5 Unit test: `in_agent_session()` returns `true` when `CLAUDECODE=1`
  (existing behaviour unchanged)
