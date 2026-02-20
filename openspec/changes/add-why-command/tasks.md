# Implementation Tasks

**CRITICAL: Use TDD (Test-Driven Development) and Tidy First principles for all development tasks below.**

**Note**: When creating beads tickets from these tasks, add "CRITICAL: Use TDD and Tidy First" at the bottom of each development ticket.

## MVP - Phase 1: Foundation
- [ ] 1.1 Add `Why` command variant to `src/cli.rs`
- [ ] 1.2 Create `src/llm.rs` module with LLM abstraction trait
- [ ] 1.3 Add `WhyConfig` to `src/config.rs` for LLM backend configuration
- [ ] 1.4 Add `[why]` section to default config in init command
- [ ] 1.5 Add error codes to `src/error.rs` (wai::llm::*)

## MVP - Phase 2: Context Gathering
- [ ] 2.1 Implement context gatherer in `src/commands/why.rs`
- [ ] 2.2 Read all artifacts from `.wai/` (research, design, plan, handoff)
- [ ] 2.3 Implement robust file path detection (exists, starts with ./, src/, contains / without spaces)
- [ ] 2.4 Gather git blame/log with graceful failure handling (check git available, file tracked)
- [ ] 2.5 Collect project metadata (current phase, recent commits)
- [ ] 2.6 Implement empty artifact warning (zero artifacts → show message)
- [ ] 2.7 Implement context size management (>100K tokens → truncate to 50 most recent)

## MVP - Phase 3: LLM Integration
- [ ] 3.1 Implement Claude API client (reqwest-based, optional)
- [ ] 3.2 Support API key from config `api_key` field OR `ANTHROPIC_API_KEY` env var
- [ ] 3.3 Implement Ollama local model client (process spawn, optional)
- [ ] 3.4 Implement backend selection priority (explicit config > auto-detect)
- [ ] 3.5 Auto-detect available LLM backend (Claude API responds, ollama model available)
- [ ] 3.6 Build prompt template with gathered context and artifact escaping
- [ ] 3.7 Support model aliases ("haiku" → "claude-haiku-3-5")

## MVP - Phase 4: Output Formatting
- [ ] 4.1 Parse LLM markdown response with graceful handling of malformed output
- [ ] 4.2 Pretty-print with colors and icons (owo_colors) with terminal fallbacks
- [ ] 4.3 Show artifact references as `file:line` format (clickable in some terminals)
- [ ] 4.4 Add `--json` flag support for machine-readable output
- [ ] 4.5 Parse relevance indicators from LLM (High/Medium/Low) for JSON output

## MVP - Phase 5: Error Handling & Fallback
- [ ] 5.1 Graceful degradation to `wai search` if no LLM available
- [ ] 5.2 Handle LLM API errors with miette diagnostics (invalid_api_key, rate_limit, network_error, model_not_found)
- [ ] 5.3 Detect Ollama model not downloaded and suggest `ollama pull`
- [ ] 5.4 Handle rate limits with clear message (wait 60s or use Ollama)
- [ ] 5.5 Add `--no-llm` flag to force fallback for testing

## Production Ready - Phase 6: Configuration
- [ ] 6.1 Support explicit config priority over auto-detection
- [ ] 6.2 Support `llm = "claude"` with `model = "haiku"` or `model = "sonnet"`
- [ ] 6.3 Support `llm = "ollama"` with `model = "llama3.1:8b"` or custom
- [ ] 6.4 Support `fallback = "search"` or `fallback = "error"`
- [ ] 6.5 Implement privacy notice tracking (`privacy_notice_shown = true` in config)
- [ ] 6.6 Display one-time privacy notice for external APIs

## Production Ready - Phase 7: Testing
- [ ] 7.1 Unit tests for context gathering logic (empty artifacts, size limits)
- [ ] 7.2 Unit tests for file path detection (spaces, dots, slashes)
- [ ] 7.3 Unit tests for prompt building (escaping, injection prevention)
- [ ] 7.4 Integration test with mock LLM (returns fixed response)
- [ ] 7.5 Integration test for fallback behavior (no LLM)
- [ ] 7.6 Integration test for git failure handling (non-git repo)
- [ ] 7.7 Manual test with real Claude API
- [ ] 7.8 Manual test with local Ollama

## Production Ready - Phase 8: Documentation
- [ ] 8.1 Update `CLAUDE.md` with `wai why` usage examples (all query types)
- [ ] 8.2 Add help text to command (`--help`) with example queries
- [ ] 8.3 Document configuration options in help output
- [ ] 8.4 Document error codes in help text
- [ ] 8.5 Add example queries to onboarding flow (if applicable)
- [ ] 8.6 Add suggestion in `wai doctor` to add wai badge to README if missing (check for wai badge patterns)
- [ ] 8.7 Include badge recommendation footer in `wai why` output when README has no badge (suggest markdown badge code)

## Polish - Phase 9: Advanced Features
- [ ] 9.1 Integrate `wai why` with global verbosity levels (`-v`/`-vv`/`-vvv`) for prompt, cost, and timing output
- [ ] 9.2 Add streaming response support (display as tokens arrive) - optional
- [ ] 9.3 Add usage stats tracking (opt-in telemetry) - optional
- [ ] 9.4 Consider caching recent queries (15min TTL) - only if latency becomes issue
