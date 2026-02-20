# Reasoning Oracle

## Purpose

Define the LLM-powered reasoning oracle that helps developers understand *why* code exists as it does by synthesizing context from artifacts, git history, and project metadata.

The term **"oracle"** refers to the system's ability to proactively surface relevant context and synthesize reasoning chains from multiple sources, like consulting a knowledgeable advisor who can connect disparate information and explain the evolution of decisions.

This capability builds on the existing `wai search` command (see [timeline-search](../timeline-search/spec.md)) by adding an LLM synthesis layer that transforms keyword matches into reasoned explanations.

## Problem Statement

As projects accumulate research notes, design documents, and plans, developers struggle to rediscover the reasoning behind past decisions. While `wai search` provides keyword matching, understanding *why* requires semantic synthesis: connecting related artifacts, building decision chains, and explaining trade-offs in natural language. These are tasks that LLMs excel at, making them ideal for reasoning discovery.

## Design Rationale

### LLM-First Architecture

This is a **Type 1 decision**: delegate reasoning to LLMs rather than implementing custom ranking algorithms. The design makes wai a **context orchestrator** that gathers artifacts and formats prompts, trusting the LLM to handle semantic understanding, relevance ranking, and narrative synthesis.

**Why LLM-first**:
- LLMs naturally understand semantic similarity ("config" ↔ "configuration")
- LLMs rank by relevance (design docs > research notes for architectural questions)
- LLMs build reasoning chains (research → design → plan) without explicit programming
- Implementation is 3x simpler (~400 LOC vs ~1200 LOC for custom algorithms)
- Quality improves automatically as LLMs improve

**Alternative considered**: Custom keyword extraction + Jaccard similarity + temporal scoring. Rejected due to complexity, brittleness, and inferior results.

### Multi-Backend Support

Support multiple LLM backends (Claude API, local Ollama, future: Gemini, OpenAI) via trait abstraction. Auto-detect available backend and gracefully degrade to `wai search` if no LLM is accessible.

## Scope and Requirements

This spec covers the `wai why` command, LLM integration, and configuration.

### Non-Goals

- Multi-turn conversation mode (future consideration)
- Custom ranking algorithms or ML models
- Vector embeddings or semantic search infrastructure
- Local-only operation (LLM is recommended but optional)

## ADDED Requirements

### Requirement: Why Command

The CLI SHALL provide `wai why <query>` to answer reasoning questions using LLM synthesis of gathered context.

#### Scenario: Natural language query

- **WHEN** user runs `wai why "why use TOML for config?"`
- **THEN** the system gathers all artifacts and git context
- **AND** sends a structured prompt to the configured LLM
- **AND** displays the LLM's synthesized answer with relevant artifacts, decision chains, and suggestions

#### Scenario: File path query

- **WHEN** user runs `wai why src/config.rs`
- **THEN** the system detects the file path
- **AND** gathers git blame and log for that file
- **AND** gathers all artifacts (prioritizing those mentioning the file or from similar timeframe)
- **AND** asks the LLM to explain why this file exists

#### Scenario: Output format

- **WHEN** displaying the answer
- **THEN** the output shows:
  - Direct answer to the question
  - 3-5 relevant artifacts with relevance explanation
  - Decision chain (temporal evolution of reasoning)
  - Suggestions for further exploration
- **AND** artifact references include file paths for verification

#### Scenario: JSON output

- **WHEN** user runs `wai why <query> --json`
- **THEN** the system outputs structured JSON with:
  - `query` (original question)
  - `answer` (LLM response text)
  - `artifacts` (array of artifact paths with relevance indicators parsed from LLM response: "High"/"Medium"/"Low")
  - `suggestions` (array of suggested follow-up commands)

**Note**: The system handles diverse query types including natural language questions (`"why use TOML for config?"`), file paths (`src/config.rs`), design decisions (`"why not async-std?"`), feature rationale (`"plugin system design"`), and rejection rationale (`"why was dynamic loading rejected?"`).

### Requirement: LLM Backend Selection

The system SHALL support multiple LLM backends via configuration and auto-detection.

#### Scenario: Auto-detect backend

- **WHEN** user runs `wai why` with default configuration (no explicit `[why] llm` in config)
- **THEN** the system checks for available backends in priority order:
  1. Check `ANTHROPIC_API_KEY` environment variable is set AND Claude API responds → use Claude Haiku
  2. Check `ollama` binary exists in PATH AND default model can be loaded → use Ollama with llama3.1:8b
- **AND** uses the first available backend
- **AND** falls back to `wai search` with warning if none available

**Note**: Explicit `[why] llm = "..."` config always takes precedence over auto-detection. Auto-detection only runs when no LLM is explicitly configured.

#### Scenario: Explicit Claude configuration

- **WHEN** `.wai/config.toml` contains:
  ```toml
  [why]
  llm = "claude"
  model = "haiku"  # or "sonnet" (short aliases)
  api_key = "sk-ant-..."  # optional, for convenience
  ```
- **THEN** the system uses Claude API with the specified model
- **AND** reads API key from config `api_key` field OR `ANTHROPIC_API_KEY` environment variable (env var takes precedence for security)
- **AND** fails with diagnostic code `wai::llm::no_api_key` if neither is set

**Note**: Short aliases (e.g., "haiku", "sonnet") are resolved to current model versions internally. Alias mappings may change as new models are released. See design.md for current mappings.

#### Scenario: Explicit Ollama configuration

- **WHEN** `.wai/config.toml` contains:
  ```toml
  [why]
  llm = "ollama"
  model = "llama3.1:8b"  # or any other Ollama model
  ```
- **THEN** the system uses local Ollama with the specified model
- **AND** fails with installation instructions if `ollama` binary not found

#### Scenario: Force fallback

- **WHEN** user runs `wai why <query> --no-llm`
- **THEN** the system skips LLM entirely and uses `wai search` directly
- **AND** displays results in search format

### Requirement: Graceful Degradation

The system SHALL fall back to `wai search` when LLM is unavailable or fails.

#### Scenario: No LLM available

- **WHEN** user runs `wai why` with no API key and no Ollama installed
- **THEN** the system displays a warning that no LLM is available
- **AND** displays remediation guidance (install Ollama or set API key)
- **AND** runs `wai search <query>` and displays keyword matches

#### Scenario: LLM API error with diagnostic codes

- **WHEN** LLM API returns error (rate limit, invalid key, network failure)
- **THEN** the system displays a miette diagnostic error with an appropriate code (e.g., `wai::llm::invalid_api_key`, `wai::llm::rate_limit`, `wai::llm::network_error`, `wai::llm::model_not_found`)
- **AND** includes actionable remediation in the error help text
- **AND** falls back to `wai search` with explanation

#### Scenario: Malformed LLM response

- **WHEN** LLM returns response that doesn't match expected markdown format
- **THEN** the system displays the raw response with a warning about unexpected format
- **AND** shows the response anyway (user can still read it)
- **AND** does NOT fall back to search (LLM did respond, just oddly)

### Requirement: Context Gathering

The system SHALL gather comprehensive context to send to the LLM while managing context size to stay within LLM limits.

#### Scenario: Gather artifacts

- **WHEN** preparing context for LLM
- **THEN** the system reads all artifacts from `.wai/`:
  - Research notes from all projects
  - Design documents from all projects
  - Plans from all projects
  - Handoffs from all projects
- **AND** includes artifact metadata (date, type, project)

#### Scenario: Gather git context for file queries

- **WHEN** the system detects the query as a file path
- **AND** the repository is a git repository
- **AND** the file is tracked by git
- **THEN** the system runs `git blame <file>` to get line-level authorship and commit info
- **AND** runs `git log --follow <file>` to get file history
- **AND** includes recent commits that touched the file

#### Scenario: Git unavailable or file not tracked

- **WHEN** query looks like a file path but:
  - Repository is not a git repository, OR
  - Git command fails, OR
  - File is not tracked by git
- **THEN** the system skips git context gathering
- **AND** displays an informational message suggesting git initialization or file tracking for better context
- **AND** proceeds with artifacts-only context

#### Scenario: Gather project metadata

- **WHEN** preparing context
- **THEN** the system includes:
  - Current project phase
  - Recent commits (last 10)
  - Active beads issues (if plugin detected)
  - Active openspec changes (if plugin detected)

#### Scenario: Plugin-contributed context

- **WHEN** preparing context and detected plugins expose relevant data
- **THEN** the system MAY incorporate context from detected plugins via the plugin interface (e.g., beads issues, openspec changes)
- **AND** plugin context is included alongside artifacts and git context in the LLM prompt

#### Scenario: No artifacts available

- **WHEN** preparing context and `.wai/` contains zero artifacts
- **THEN** the system warns that no artifacts were found and suggests using `wai add`
- **AND** still proceeds with LLM call using only git context and project metadata
- **AND** the prompt instructs the LLM to note that no project artifacts were available

#### Scenario: Context size management for large projects

- **WHEN** total artifact content exceeds the configured model's context budget
- **THEN** the system selects a subset of artifacts, prioritizing relevance to the query and recency
- **AND** informs the user that context was truncated and suggests `wai search` for broader coverage
- **AND** includes a truncation note in the prompt for LLM awareness

### Requirement: Prompt Construction

The system SHALL build structured prompts that guide the LLM to produce consistent, useful answers.

#### Scenario: Basic prompt structure

- **WHEN** building a prompt
- **THEN** the prompt includes:
  - Role definition ("You are an oracle...")
  - User question
  - Project context (phase, commits)
  - Artifacts with metadata (in escaped code blocks to prevent markdown injection)
  - Git context (if file query)
  - Task instructions (identify relevant artifacts, synthesize narrative, suggest next steps)
  - Output format instructions (markdown sections)
- **AND** all artifact content is escaped or quoted to prevent:
  - Markdown injection (triple backticks, special characters)
  - Prompt injection (instructions embedded in artifacts)
  - Format breaking (malformed markdown)

#### Scenario: Verbosity levels

- **WHEN** user runs `wai why <query>` with global verbosity flags (`-v`, `-vv`, `-vvv`)
- **THEN** the system provides progressively more detail:
  - `-v`: backend selected, artifact count, estimated cost
  - `-vv`: full prompt sent to the LLM
  - `-vvv`: raw LLM response, timing breakdown, token counts

### Requirement: Cost Awareness

The system SHALL default to cost-effective LLM models and provide usage transparency.

#### Scenario: Default to cost-effective models

- **WHEN** using Claude without explicit model configuration
- **THEN** the system defaults to the most cost-effective available model
- **AND** allows users to configure a more capable model via the `model` config field

**Note**: The system SHALL warn when estimated query cost exceeds a configurable threshold (default $0.05) due to large context or expensive model selection.

#### Scenario: Display cost estimate

- **WHEN** user runs `wai why <query>` with verbosity level 1 or higher (`-v`)
- **THEN** the output includes estimated cost for the query
- **AND** shows token counts (input and output) at verbosity level 3 (`-vvv`)

### Requirement: Privacy Considerations

The system SHALL inform users when sending artifacts to external APIs and provide local alternatives.

#### Scenario: Privacy warning for external APIs

- **WHEN** using external LLM API (Claude, Gemini, OpenAI) for the first time
- **AND** `.wai/config.toml` does not have `privacy_notice_shown = true`
- **THEN** the system displays a one-time notice informing the user that artifacts will be sent to the external provider
- **AND** suggests local Ollama as a privacy-preserving alternative
- **AND** records acknowledgment in `.wai/config.toml`
- **AND** proceeds with the query after displaying the notice

#### Scenario: Local-only mode

- **WHEN** configured to use Ollama
- **THEN** the system processes all queries locally without network calls
- **AND** indicates local-only operation at verbosity level 1 or higher
