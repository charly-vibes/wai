# Change: Add LLM-Powered Reasoning Oracle (`wai why`)

## Why

Developers using wai accumulate rich context (research, designs, plans) but struggle to rediscover *why* decisions were made. The existing `wai search` command provides keyword matching, but LLMs are far better at:
- Understanding semantic similarity ("config" ↔ "configuration")
- Ranking by relevance (design docs > research notes)
- Building reasoning chains (research → design → plan)
- Synthesizing coherent narratives

Rather than reimplementing ML/ranking algorithms poorly, delegate to LLMs and make wai a **context orchestrator** that gathers artifacts, formats prompts, and presents synthesized answers.

## What Changes

- **NEW command**: `wai why <query>` - LLM-powered oracle for discovering reasoning
- **NEW capability**: `reasoning-oracle` - context gathering, prompt building, LLM integration
- **NEW config section**: `[why]` in `.wai/config.toml` for LLM backend selection
- **MODIFIED**: `cli-core` spec to add `why` command

The implementation is deliberately simple (~400 LOC):
1. Gather context (artifacts + git info) - no complex indexing
2. Format prompt with context
3. Call LLM (Claude, Gemini, OpenAI, or local Ollama)
4. Display synthesized answer

Graceful degradation: if no LLM available, falls back to `wai search` with a warning.

**Badge recommendation**: `wai why` and `wai doctor` will suggest adding a wai badge to the README when one isn't present, helping projects showcase their reasoning workflow.

**Development approach**: All development tasks follow TDD (Test-Driven Development) and Tidy First principles to ensure quality and maintainability.

## Impact

- **Affected specs**:
  - `cli-core` (MODIFIED - add `why` command to command structure)
  - `reasoning-oracle` (ADDED - new capability)

- **Affected code**:
  - `src/cli.rs` - add `Why` command variant
  - `src/commands/why.rs` - new command implementation
  - `src/llm.rs` - new module for LLM integration
  - `src/config.rs` - add `WhyConfig` struct
  - `src/error.rs` - add LLM-specific error codes (wai::llm::*)

- **Affected docs**:
  - `CLAUDE.md` - add `wai why` usage examples

- **Dependencies**:
  - Optional: LLM API clients (reqwest for HTTP, or local model via process spawn)
  - No new required dependencies (degrades gracefully)

- **Cost**: ~$0.003 per query with Claude Haiku (dirt cheap)

- **User impact**:
  - Positive: Much better than keyword search for understanding decisions
  - Learning: New command to discover, but natural language makes it intuitive
  - Performance: ~500ms latency (dominated by LLM), acceptable for reasoning queries
