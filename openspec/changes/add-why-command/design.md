# Design: LLM-First Reasoning Oracle

## Context

Wai stores rich reasoning artifacts (research, designs, plans) but lacks an intelligent way to surface relevant context when developers ask "why was this built this way?". The existing `wai search` provides keyword matching, but synthesizing reasoning chains requires semantic understanding, relevance ranking, and narrative construction - all things LLMs excel at.

**Core insight**: Don't reimplement what LLMs already do well. Make wai a **context orchestrator** that gathers artifacts and delegates reasoning to the LLM.

## Goals / Non-Goals

### Goals
- Enable natural language queries: `wai why "why use TOML?"`
- Support file-based queries: `wai why src/config.rs`
- Synthesize reasoning chains (research → design → plan)
- Work with multiple LLM backends (Claude, Ollama, etc.)
- Gracefully degrade when no LLM available
- Keep implementation simple (<500 LOC total)

### Non-Goals
- Building custom ranking algorithms or ML models
- Implementing vector embeddings or semantic search
- Creating a conversational chatbot (no multi-turn for MVP)
- Training or fine-tuning models
- Local-only operation (LLM is optional but recommended)

## Architecture

### Component Overview

```
┌─────────────────────────────────────┐
│      wai why <query>                │
└─────────────┬───────────────────────┘
              │
              ├──> ContextGatherer
              │    ├─ Read all artifacts
              │    ├─ Get git context (if file)
              │    └─ Collect metadata
              │
              ├──> PromptBuilder
              │    └─ Format context into prompt
              │
              ├──> LLMClient (trait)
              │    ├─ ClaudeClient (API)
              │    ├─ OllamaClient (local)
              │    └─ (future: GeminiClient, etc.)
              │
              └──> OutputFormatter
                   └─ Pretty-print markdown
```

### Data Flow

1. **Input**: User query (string)
2. **Gather**: All artifacts + git info → `GatheredContext`
3. **Build**: Context → formatted prompt (string)
4. **Call**: Prompt → LLM → response (string)
5. **Format**: Response → pretty output (stdout)

## Decisions

### Decision 1: LLM-First Architecture

**What**: Delegate reasoning to LLMs, wai only gathers context.

**Why**:
- LLMs are better at semantic matching than regex/keywords
- LLMs naturally build reasoning chains and synthesize narratives
- Avoids reimplementing ranking algorithms (which would be inferior)
- Implementation is 3x simpler (~400 LOC vs ~1200 LOC for custom ranking)

**Alternatives considered**:
- Custom keyword extraction + Jaccard similarity: Complex, brittle, worse results
- Vector embeddings + cosine similarity: Requires local model, adds complexity
- Hybrid (simple ranking + LLM refinement): More complexity, marginal benefit

**Trade-offs**:
- ✅ Better results with less code
- ✅ Improves automatically as LLMs improve
- ❌ Requires LLM access (mitigated by local Ollama support)
- ❌ ~500ms latency (acceptable for reasoning queries)
- ❌ Small cost per query (mitigated by cheap models like Haiku)

### Decision 2: Multi-Backend Support

**What**: Support multiple LLM backends via trait abstraction.

**Why**:
- Users have different preferences (API vs local, cost vs speed)
- Enables graceful degradation (try Claude, fall back to Ollama)
- Future-proof for new providers

**Implementation**:
```rust
trait LLMClient {
    fn complete(&self, prompt: &str) -> Result<String>;
    fn is_available(&self) -> bool;
}

struct ClaudeClient { api_key: String }
struct OllamaClient { model: String }
```

**Backend Selection Priority**:
1. Explicit config `[why] llm = "..."` (always takes precedence)
2. Auto-detection (only when no explicit config):
   a. Check for `ANTHROPIC_API_KEY` env var AND Claude API responds → use Claude
   b. Check for `ollama` binary AND model available → use Ollama
3. Fall back to `wai search` with warning if none available

### Decision 3: Prompt Design

**What**: Simple, structured prompt with clear sections.

**Template**:
```
You are an oracle helping understand why code exists as it does.

# User Question
{query}

# Project Context
- Current phase: {phase}
- Recent commits: {commits}

# Available Artifacts
{artifacts}

# Git Context (if file query)
{git_info}

# Task
Identify 3-5 most relevant artifacts, explain why they're relevant,
synthesize a narrative showing decision evolution, suggest next steps.

Format: Markdown with sections (Answer, Relevant Artifacts,
Decision Chain, Suggestions)
```

**Why this structure**:
- Clear role definition ("oracle")
- Explicit context sections (easy to parse and debug)
- Structured output format (easy to parse for future JSON mode)
- Specific task instructions (get consistent responses)

### Decision 4: Graceful Degradation

**What**: Fall back to `wai search` if no LLM available.

**Flow**:
```rust
fn wai_why(query: &str) -> Result<String> {
    let ctx = gather_context(query);

    if let Some(llm) = detect_llm() {
        match llm.complete(&build_prompt(&ctx)) {
            Ok(response) => Ok(format_output(response)),
            Err(e) => {
                eprintln!("⚠ LLM error: {}. Falling back to search.", e);
                wai_search(query)
            }
        }
    } else {
        eprintln!("⚠ No LLM available. Falling back to search.");
        eprintln!("💡 Install ollama or set ANTHROPIC_API_KEY for better results.");
        wai_search(query)
    }
}
```

**Why**:
- User always gets *something* (search results)
- Helpful diagnostic explains how to improve
- No hard dependency on LLM access

### Decision 4b: Partial Degradation (Future Consideration)

The current fallback is binary: LLM works → full synthesis, OR LLM fails → raw search. Future versions could explore intermediate modes:
- Use search to pre-filter artifacts, then LLM to synthesize only the filtered set (cheaper, faster)
- Show search results AND attempt LLM synthesis in parallel
- Use LLM for ranking only, skip narrative synthesis on error

These are deferred for MVP. The binary fallback is simpler and the UX difference is minimal for the common case.

### Decision 5: Simple Context Gathering (No Indexing)

**What**: On each query, read all artifacts fresh (no persistent index).

**Why**:
- Simpler implementation (no index corruption bugs)
- Artifacts are small and few (typically <100 total)
- File reads are fast (<10ms for typical project)
- Avoids index staleness issues

**Performance**:
- Read 20 artifacts @ 2KB each = 40KB = ~10ms on SSD
- Git blame + log = ~50ms
- Prompt construction = ~1ms
- LLM call = ~500ms
- **Total: ~560ms** (dominated by LLM, not I/O)

**When to add indexing**: If projects routinely have >1000 artifacts. Current data suggests this is rare.

### Decision 6: Cost Management

**What**: Default to cheap, fast models (Claude Haiku, Llama 3.1 8B).

**Cost per query** (Claude Haiku - typical project):
- Input: ~11K tokens (20 artifacts × 500 tokens) × $0.25/M = $0.00275
- Output: ~500 tokens × $1.25/M = $0.000625
- **Total: ~$0.003**

**Note**: Token count varies with project size. Large projects (100+ artifacts) may use 50K+ tokens = ~$0.015 per query. System truncates to 100K tokens max to prevent excessive costs.

Even heavy users (100 queries/day) on typical projects = $0.30/day = $110/year.

**Local alternative**: Ollama with Llama 3.1 8B (free, ~2-5s latency on modest GPU).

## Risks / Trade-offs

### Risk 1: LLM Unavailability
- **Impact**: Users without API keys or Ollama can't use `wai why`
- **Mitigation**: Fall back to `wai search`, provide clear setup instructions
- **Severity**: Low (search is acceptable fallback)

### Risk 2: LLM Hallucination
- **Impact**: LLM might claim artifact says something it doesn't
- **Mitigation**: Show artifact file paths so users can verify
- **Future**: Add `--verify` mode that quotes exact text from artifacts
- **Severity**: Medium (mitigated by transparency)

### Risk 3: Latency
- **Impact**: 500ms feels slow compared to instant `wai search`
- **Mitigation**: Show spinner, add streaming for perceived speed
- **Acceptance**: Users expect reasoning to take time (vs simple search)
- **Severity**: Low (acceptable for reasoning queries)

### Risk 4: Cost
- **Impact**: API costs could accumulate for heavy users
- **Mitigation**: Default to cheap Haiku, support free Ollama, add usage tracking
- **Acceptance**: $0.003/query is negligible for value provided
- **Severity**: Very Low

### Risk 5: Privacy
- **Impact**: Sending artifacts to external LLM API
- **Mitigation**:
  - Make LLM backend configurable (prefer local Ollama for sensitive projects)
  - Add `--no-llm` flag to force local-only fallback
  - Document in help text: "Sends artifacts to {provider}"
- **Severity**: Medium (serious for some users, mitigated by local option)
- **Limitation**: The `privacy_notice_shown` flag in config is single-user scoped. In team environments where multiple developers share a wai workspace, only the first user sees the notice. Per-session consent or per-user tracking may be needed for compliance-sensitive teams. Not addressed in MVP.

## Migration Plan

N/A - This is a new feature with no breaking changes.

**Rollout**:
1. Ship with Claude + Ollama support in MVP
2. Add usage telemetry (opt-in) to understand adoption
3. Add more backends (Gemini, OpenAI) based on demand
4. Consider caching layer if latency becomes issue

**Rollback**: Remove command, no data migration needed.

## Future Enhancements (Post-MVP)

These features are intentionally deferred to keep MVP focused:

### Conversation Mode (Multi-Turn)
- **Benefit**: More natural UX, can refine queries
- **Complexity**: Requires state management, session tracking
- **Decision**: Add if users request it. Most reasoning questions are one-shot.

### Query Result Caching
- **Benefit**: Instant response for repeated queries
- **Complexity**: Cache invalidation when artifacts change, storage management
- **Decision**: Add with 15min TTL if latency becomes issue.

### Custom System Prompts
- **Benefit**: Power users can tune for their domain
- **Complexity**: Prompt engineering is hard, most users won't use it effectively
- **Decision**: Consider `[why] system_prompt = "..."` config if demand exists.

### Streaming Output
- **Benefit**: Better perceived performance, feels more responsive
- **Complexity**: More complex client implementation (async token handling)
- **Decision**: Implement in Phase 2 (polish). MVP shows spinner until complete.

### Extended Query Types
During research, several additional query modes were considered and explicitly deferred:
- `wai why --file src/foo.rs --since 2024-01` — time-scoped file queries
- `wai why --feature plugin-system` — feature-name queries
- `wai why --grep "phase tracking"` — grep-based reasoning search
- `wai why HEAD~3..HEAD` — commit-range queries
- `wai why --commit abc123` — single-commit queries
- `wai why --here` — context-aware suggestions based on current directory
- `wai why --suggest` — proactive suggestions based on recent changes

MVP supports only `wai why <query>` (natural language string or file path). These richer query types should be considered if users find the simple input too limiting.

## Implementation Notes

These details are intentionally kept out of the spec (which focuses on *what*, not *how*) but guide initial implementation.

### File Path Detection Heuristics

The system needs to distinguish file path queries from natural language. Recommended heuristics:
- File exists at the given path
- Path starts with `./`, `../`, or `src/`
- Contains `/` without spaces (e.g., `path/to/file.rs` but not `config vs yaml`)
- Does NOT match patterns like version numbers (`tokio 1.0`)

These heuristics should be tuned based on real usage. False positives (treating a question as a file) are more disruptive than false negatives (treating a file as a question, which still works via keyword matching).

### Context Size Budget

Default context budget: ~100,000 tokens (~400KB of text). When exceeded, select a subset prioritizing:
1. Artifacts whose content mentions query terms
2. Most recent artifacts by date
3. At least one artifact of each type (research, design, plan) if available

The token threshold should be configurable and may need adjustment per backend (e.g., Ollama models often have smaller context windows than Claude).

### Model Alias Mappings (Current)

These mappings are implementation defaults and will change as new models are released:
- `"haiku"` → `claude-haiku-3-5`
- `"sonnet"` → `claude-sonnet-4`

Consider loading these from a config file or updating them via `wai` releases rather than hard-coding.

### UX Message Patterns

User-facing messages should follow wai's existing patterns (miette diagnostics, owo_colors). Example messages for reference:
- No LLM available: warning + remediation guidance (install Ollama or set API key)
- No artifacts: warning + suggest `wai add`
- Git unavailable: informational + suggest git init or file tracking
- Context truncated: informational + suggest `wai search` for broader coverage
- Privacy notice: informational + suggest local Ollama alternative
- Malformed LLM response: warning + show raw response anyway

Exact message copy is an implementation concern — keep it consistent with existing wai output style.

### Cost Estimates (At Time of Writing)

Based on Claude Haiku pricing as of early 2026:
- Input: ~11K tokens (20 artifacts x 500 tokens) x $0.25/M = $0.00275
- Output: ~500 tokens x $1.25/M = $0.000625
- **Typical query: ~$0.003**

Token count varies with project size. Large projects (100+ artifacts) may use 50K+ tokens. System truncates to the context budget to prevent excessive costs. Even heavy users (100 queries/day) on typical projects = ~$0.30/day.

Local alternative: Ollama with Llama 3.1 8B (free, ~2-5s latency on modest GPU).

**Note**: These prices are snapshots. Update this section when model pricing changes materially.

## Success Metrics

How to measure if this is working:

1. **Adoption**: What % of users run `wai why` at least once?
2. **Retention**: Do users come back to `wai why` or try it once and quit?
3. **Fallback rate**: How often does it fall back to search? (LLM availability)
4. **Qualitative**: User reports - does it help understand decisions?

**Target**: 50% of active wai users try `wai why` within first month.
