LLM-First Oracle Design: wai as Context Orchestrator

## Core Insight

Don't reimplement what LLMs already do well (ranking, synthesis, reasoning). Instead,  should:

1. **Gather** relevant context efficiently
2. **Format** it for optimal LLM consumption  
3. **Prompt** the LLM to synthesize
4. **Present** the LLM's answer

## Architecture Shift: From Smart Search to Smart Prompting

### Old Approach (Complex)
```
wai implements:
- Keyword extraction
- Jaccard similarity
- Scoring algorithms (5 signals × weights)
- Relevance ranking
- Deduplication
- Chain building
```

### New Approach (Simple)
```
wai implements:
- Context gathering (fast!)
- Prompt construction (string formatting)
- LLM already does: ranking, synthesis, chain building, deduplication
```

## Minimal Implementation Design

### Component 1: Context Gatherer

Dead simple - no intelligence:

```rust
struct ContextGatherer {
    wai_dir: PathBuf,
    git: GitRepo,
}

impl ContextGatherer {
    fn gather(&self, query: &str) -> GatheredContext {
        GatheredContext {
            query: query.to_string(),
            
            // 1. All artifacts (cheap - just file reads)
            artifacts: self.read_all_artifacts(),
            
            // 2. Git context if file path detected
            git_info: if looks_like_path(query) {
                Some(self.git.blame_and_log(query))
            } else {
                None
            },
            
            // 3. Project metadata
            current_phase: self.current_phase(),
            recent_commits: self.git.log("-10"),
        }
    }
}
```

No scoring, no ranking, no keyword extraction. Just gather everything potentially relevant.

### Component 2: Prompt Constructor

Format context for LLM consumption:

```rust
fn build_prompt(ctx: &GatheredContext) -> String {
    format!(r#"
You are an oracle helping a developer understand why code exists as it does.

# User Question
{query}

# Project Context
- Current phase: {phase}
- Recent activity: {commits}

# Available Artifacts

{artifacts_formatted}

# Git Context
{git_info_formatted}

# Your Task

Analyze the above context and:

1. Identify the 3-5 most relevant artifacts that explain the user's question
2. For each artifact, explain:
   - Why it's relevant
   - What reasoning it provides
   - How confident you are (high/medium/low)

3. Synthesize a narrative:
   - Show the decision chain (research → design → plan)
   - Explain tradeoffs that were considered
   - Surface alternatives that were rejected

4. Suggest next steps for deeper exploration

Format your response as:

## Answer
[Direct answer to the question]

## Relevant Artifacts
[Ranked list with explanations]

## Decision Chain
[Timeline of reasoning evolution]

## Suggestions
[What to explore next]
"#,
        query = ctx.query,
        phase = ctx.current_phase,
        commits = format_commits(&ctx.recent_commits),
        artifacts_formatted = format_artifacts(&ctx.artifacts),
        git_info_formatted = format_git(&ctx.git_info),
    )
}
```

### Component 3: LLM Invoker

Just call the LLM:

```rust
fn invoke_llm(prompt: &str) -> String {
    // Use whatever LLM is available:
    // - Claude API (via env var ANTHROPIC_API_KEY)
    // - Local model (ollama, llama.cpp)
    // - Gemini, OpenAI, etc.
    
    llm::complete(prompt)
}
```

### Component 4: Output Formatter

Parse LLM response and pretty-print:

```rust
fn format_output(llm_response: &str) -> String {
    // Parse markdown sections
    // Add colors, icons
    // Make clickable links to artifacts
    
    format!("
📍 {query}

{llm_answer}

💡 {suggestions}
")
}
```

## Full Flow

```
$ wai why "why use TOML for config?"

1. Gather all artifacts + git context     [10ms]
2. Build prompt with context              [1ms]
3. Call LLM (e.g., claude-haiku)          [500ms]
4. Format and display response            [1ms]

Total: ~500ms (mostly LLM latency)
```

## What the LLM Does (That We Don't Have To)

1. **Keyword Matching**: LLM understands "config" relates to "configuration", "settings", "options"
2. **Relevance Ranking**: LLM knows design docs > research notes for architectural questions
3. **Temporal Correlation**: LLM sees dates and understands "this research led to this design"
4. **Chain Building**: LLM naturally constructs narratives and timelines
5. **Deduplication**: LLM merges similar content automatically
6. **Synthesis**: LLM writes coherent explanations, not just lists

## Optimization: Smart Context Filtering

To keep prompts small (cheaper, faster):

```rust
fn filter_context(ctx: GatheredContext, query: &str) -> FilteredContext {
    // SIMPLE heuristics (no ML):
    
    // 1. If query looks like file path, prioritize:
    //    - Artifacts mentioning that file
    //    - Artifacts from ±30 days of file creation
    
    // 2. If query is question:
    //    - Include all artifacts (let LLM filter)
    //    - But limit to most recent 20
    
    // 3. Always include:
    //    - Current phase artifacts
    //    - Most recent of each type (research/design/plan)
}
```

Even "dumb" filtering is fine - LLMs are great at ignoring irrelevant context.

## Graceful Degradation

```rust
fn wai_why(query: &str) -> Result<String> {
    let ctx = gather_context(query);
    
    // Try LLM first
    if let Ok(llm_response) = invoke_llm(&build_prompt(&ctx)) {
        return Ok(format_output(&llm_response));
    }
    
    // Fallback: simple search (existing 'wai search' logic)
    eprintln!("⚠ LLM unavailable, falling back to keyword search");
    wai_search(query)
}
```

## Configuration

```toml
# .wai/config.toml

[why]
# LLM backend
llm = "auto"  # auto-detect: claude > gemini > openai > ollama
model = "haiku"  # fast model for quick answers

# Or specify explicitly
# llm = "claude"
# api_key = "sk-ant-..."  # or use ANTHROPIC_API_KEY env var

# Or use local
# llm = "ollama"
# model = "llama3.1:8b"

# Fallback if no LLM available
fallback = "search"  # or "error"

# Context limits
max_artifacts = 20  # prevent huge prompts
max_artifact_length = 5000  # truncate long artifacts
```

## Implementation Complexity Comparison

### Old Approach (Implementing Intelligence)
- Keyword extraction: 200 LOC
- Similarity metrics: 150 LOC  
- Ranking algorithm: 300 LOC
- Deduplication: 100 LOC
- Chain building: 200 LOC
- Index management: 400 LOC
- **Total: ~1350 LOC + ongoing tuning**

### New Approach (Delegating to LLM)
- Context gathering: 100 LOC
- Prompt building: 150 LOC
- LLM integration: 100 LOC
- Output formatting: 50 LOC
- **Total: ~400 LOC, no tuning needed**

## Cost Analysis

### Prompt size (typical query)
- Artifacts: 20 × 2KB = 40KB ≈ 10K tokens
- Git context: 2KB ≈ 500 tokens
- Prompt template: 1KB ≈ 250 tokens
- **Total input: ~11K tokens**

### Response size
- Typical answer: 2KB ≈ 500 tokens

### Cost per query (Claude Haiku)
- Input: 11K tokens × $0.25/M = $0.00275
- Output: 500 tokens × $1.25/M = $0.000625
- **Total: ~$0.003 per query**

Dirt cheap. Even heavy users (100 queries/day) = $0.30/day.

## UX Examples

### Example 1: File Query

```
$ wai why src/plugins/mod.rs

📍 Why: src/plugins/mod.rs

## Answer

The plugin system uses trait-based architecture for type safety and 
extensibility. This decision was made in December 2024 after evaluating 
three approaches: dynamic loading, trait-based, and macro-based.

## Relevant Artifacts (3 found)

1. **Design: Plugin Architecture** (Dec 10, 2024) · High confidence
   - Explicitly discusses this file and the trait-based decision
   - Rejected dynamic loading due to safety concerns
   - Chose traits over macros for compile-time verification

2. **Research: Plugin Discovery** (Dec 12, 2024) · Medium confidence
   - Evaluated configuration approaches (TOML vs glob)
   - Chose TOML for explicit plugin declaration
   - Related to how plugins are discovered at runtime

3. **Plan: Git Plugin Implementation** (Dec 14, 2024) · Medium confidence
   - Uses plugins/mod.rs as the trait definition
   - Git plugin serves as reference implementation

## Decision Chain

Dec 10: Research phase → explored dynamic loading (like vim plugins)
Dec 10: Design phase → chose traits for Rust's type system strengths
Dec 12: Research phase → decided on TOML-based discovery
Dec 14: Plan phase → spec'd out first implementation (git plugin)
Dec 15: Implementation → created src/plugins/mod.rs with trait

## Suggestions

• Read full design doc: wai show plugin-architecture
• See how git plugin implements this: wai why src/plugins/git.rs
• Timeline view: wai timeline --from 2024-12-10
```

### Example 2: Question Query

```
$ wai why "why TOML instead of YAML?"

📍 Why: why TOML instead of YAML?

## Answer

TOML was chosen over YAML for configuration to avoid YAML's footguns
(Norway problem, type coercion surprises) and because TOML's explicit
syntax aligns with wai's philosophy of deliberate, documented decisions.

## Relevant Artifacts (2 found)

1. **Research: Config Format Evaluation** (Jan 15) · High confidence
   - Directly addresses this question
   - Lists YAML issues: implicit typing, indentation errors
   - Notes TOML's Rust ecosystem support (serde, config crates)

2. **Design: Agent Config Philosophy** (Jan 18) · Medium confidence  
   - Doesn't mention YAML directly, but emphasizes "explicit over implicit"
   - TOML choice aligns with this principle

## Decision Chain

Jan 10: Initial implementation used JSON
Jan 15: Research → discovered YAML footguns, evaluated alternatives
Jan 18: Design → established "explicit" principle
Jan 20: Implementation → migrated to TOML

## Suggestions

• See the research details: wai show config-format-eval
• Related: "why not JSON?" - wai why "config format json"
```

## Advanced: Streaming Responses

For better UX, stream LLM response:

```
$ wai why src/config.rs

📍 Why: src/config.rs

## Answer
The config system uses TOML because... [streaming text]
```

```rust
fn invoke_llm_streaming(prompt: &str) -> impl Stream<Item = String> {
    llm::complete_streaming(prompt)
}

// Display incrementally as tokens arrive
for chunk in invoke_llm_streaming(&prompt) {
    print!("{}", chunk);
    stdout().flush();
}
```

## Edge Cases (Now Trivial)

### No LLM available?
→ Fall back to  with warning

### No artifacts match?
→ LLM says "I couldn't find relevant artifacts, but based on your question..."

### Too many artifacts?
→ Simple heuristic: most recent 20, LLM handles the rest

### Circular references?
→ LLM detects and explains: "These designs evolved iteratively..."

### Deleted files?
→ Include in git context, LLM explains: "This file was removed in commit X because..."

## Why This Is Better

### For Users
- More intelligent answers (LLM > our heuristics)
- Natural language works out of the box
- Synthesis, not just search results
- Evolves as LLMs improve (no code changes)

### For Maintainers  
- 1/3 the code
- No ML/ranking algorithms to tune
- No index corruption bugs
- No performance optimization needed (LLM is the bottleneck)

### For the Project
- Faster to ship MVP
- Easier to maintain
- More powerful from day 1
- Future-proof (new LLMs, better models)

## Implementation Phases

**Phase 1: MVP (1-2 days)**
- Context gathering (all artifacts + git)
- Basic prompt template
- Claude API integration
- Simple output formatting

**Phase 2: Polish (1 day)**  
- Streaming responses
- Better prompt engineering
- Clickable artifact links
- Color output

**Phase 3: Flexibility (1 day)**
- Multiple LLM backends (Gemini, OpenAI, Ollama)
- Config file for model selection
- Fallback to search if no LLM

**Phase 4: Advanced (future)**
- Conversation mode (follow-up questions)
- Artifact caching (reduce prompt size)
- Custom system prompts per project

## Key Insight

**wai's job is to be a great librarian, not a great reader.**

Gather the books (artifacts), organize them by date (git), hand them to the expert (LLM), let the expert explain.

Don't try to be the expert - you're not as good as an LLM at reasoning about text.

