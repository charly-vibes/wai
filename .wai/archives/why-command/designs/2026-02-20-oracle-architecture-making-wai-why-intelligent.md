Oracle Architecture: Making 'wai why' Intelligent

## Overview

The 'wai why' command transforms wai from a passive archive into an active reasoning assistant. It's not just search - it's an **oracle** that synthesizes context from multiple sources and guides LLMs to understanding.

## Architecture

### Core Components

```
┌─────────────────────────────────────────┐
│         wai why <query>                 │
└────────────┬────────────────────────────┘
             │
             ├──> Query Parser
             │    └─> Intent detection
             │        • file path
             │        • natural language
             │        • feature name
             │        • commit/diff
             │
             ├──> Context Extractors
             │    ├─> Git Extractor (blame, log, diff)
             │    ├─> Artifact Extractor (research/design/plan)
             │    ├─> Code Extractor (AST, comments)
             │    └─> Dependency Extractor (relationships)
             │
             ├──> Relevance Ranker
             │    ├─> Temporal scoring
             │    ├─> Semantic similarity
             │    ├─> Explicit mentions
             │    └─> Phase context
             │
             ├──> Synthesizer
             │    ├─> Deduplication
             │    ├─> Chain building (A→B→C)
             │    ├─> Confidence scoring
             │    └─> Progressive disclosure
             │
             └──> Output Formatter
                  ├─> Summary view (default)
                  ├─> Detailed view (--verbose)
                  ├─> JSON export (--json)
                  └─> Suggestions (next steps)
```

## Query Parser Design

### Intent Detection

Parse user input into structured query:

```rust
enum QueryIntent {
    File { path: PathBuf, since: Option<Date> },
    Feature { name: String, grep: Option<String> },
    Question { text: String },
    Commit { range: String },
    Suggest { context: SuggestContext },
}
```

### Examples

| Input | Parsed Intent |
|-------|--------------|
| `src/config.rs` | File { path, since: None } |
| `--feature plugin-system` | Feature { name: "plugin-system" } |
| `why use tokio?` | Question { text: "why use tokio?" } |
| `HEAD~3..HEAD` | Commit { range: "HEAD~3..HEAD" } |
| `--suggest` | Suggest { context: Recent } |

## Context Extractors

### Git Extractor

Provides temporal and authorship context:

```rust
struct GitContext {
    blame: Vec<BlameEntry>,    // who, when, commit
    commits: Vec<Commit>,       // related commits
    file_history: Vec<Change>,  // evolution of file
}

impl GitExtractor {
    fn extract_for_file(&self, path: &Path) -> GitContext {
        // git blame -> map lines to commits
        // git log -> get commit messages
        // detect artifact references in commit msgs
    }
}
```

### Artifact Extractor

Searches research/design/plan artifacts:

```rust
struct ArtifactContext {
    matches: Vec<ArtifactMatch>,
    relationships: Vec<(ArtifactId, ArtifactId)>,
}

struct ArtifactMatch {
    artifact: Artifact,
    score: f64,              // relevance 0.0-1.0
    matched_text: Vec<String>,
    match_type: MatchType,   // Explicit | Keyword | Temporal
}

impl ArtifactExtractor {
    fn search(&self, query: &Query) -> ArtifactContext {
        // keyword search
        // date range filtering
        // file path mentions
        // cross-reference following
    }
}
```

### Code Extractor (Future)

For deeper understanding:

```rust
struct CodeContext {
    functions: Vec<FunctionDef>,
    comments: Vec<Comment>,
    imports: Vec<Import>,
}

// Phase 2 feature - AST parsing
// Enables function-level granularity
```

## Relevance Ranker

### Scoring Algorithm

```rust
fn compute_relevance(
    artifact: &Artifact,
    query: &Query,
    git_ctx: &GitContext,
) -> f64 {
    let mut score = 0.0;
    
    // 1. Temporal proximity (30%)
    let days_ago = (now - artifact.date).days();
    score += 0.3 * f64::exp(-days_ago as f64 / 30.0);
    
    // 2. Type priority (30%)
    score += match artifact.type {
        Design => 0.30,
        Plan => 0.20,
        Research => 0.15,
    };
    
    // 3. Keyword match (25%)
    let keywords = extract_keywords(&query);
    let similarity = jaccard_similarity(
        &keywords,
        &artifact.keywords
    );
    score += 0.25 * similarity;
    
    // 4. Explicit mention (20%)
    if artifact.mentions_file(&query.file) {
        score += 0.20;
    }
    
    // 5. Phase context (10%)
    if artifact.phase == current_phase() {
        score += 0.10;
    }
    
    score.min(1.0)
}
```

### Threshold

- Minimum relevance: 0.4 (configurable)
- Show top N: 5 (default), unlimited with --all
- Confidence tiers:
  - 0.8+ = High (show first, highlight)
  - 0.6-0.8 = Medium (standard)
  - 0.4-0.6 = Low (show count, expand with --verbose)

## Synthesizer

### Deduplication

Multiple artifacts may discuss same decision:

```rust
fn deduplicate(matches: Vec<ArtifactMatch>) -> Vec<SynthesizedEntry> {
    // Group by topic similarity
    // Prefer: design > plan > research
    // Merge overlapping date ranges
}
```

### Chain Building

Trace reasoning evolution:

```
Research (2024-01-10): "Evaluated tokio, async-std, smol"
  └─> Design (2024-01-15): "Chose tokio for ecosystem maturity"
      └─> Plan (2024-01-18): "Use tokio::spawn for background tasks"
```

### Progressive Disclosure

```rust
struct OutputLevel {
    Summary,    // top 3, one-line each
    Standard,   // top 5, short excerpt
    Detailed,   // all matches, full excerpts
    Json,       // machine-readable
}
```

## Output Formatter

### Summary View (Default)

```
$ wai why src/plugins/mod.rs

📍 src/plugins/mod.rs · created 2024-12-15

🔍 3 artifacts explain this:

  1. [design] Plugin Architecture (Dec 10) · 95% confidence
     Trait-based design for type safety, rejected dynamic loading

  2. [research] Plugin Discovery (Dec 12) · 82% confidence
     Evaluated TOML/glob/env, chose TOML for explicitness
     
  3. [plan] Plugin Implementation (Dec 14) · 78% confidence
     Hook system, status method, git plugin as reference

💡 wai show plugin-architecture
💡 wai timeline why-command --from 2024-12-10
```

### Detailed View (--verbose)

Shows excerpts from artifacts, commit messages, relationships.

### JSON Export (--json)

Machine-readable for tooling:

```json
{
  "query": {"type": "file", "path": "src/plugins/mod.rs"},
  "matches": [
    {
      "artifact_id": "...",
      "type": "design",
      "title": "Plugin Architecture",
      "date": "2024-12-10",
      "relevance": 0.95,
      "excerpt": "...",
      "match_type": "explicit"
    }
  ],
  "related": {
    "commits": [...],
    "dependencies": [...]
  },
  "suggestions": [...]
}
```

## Suggestion Engine

### Context-Based Suggestions

When to suggest automatically:

1. **After commits**: "Recent changes to auth.rs - wai why src/auth.rs"
2. **In directories**: "Multiple plugin files here - wai why --feature plugins"
3. **During errors**: "Build failed in config.rs - wai why src/config.rs"

### Learning from Usage

Track what queries were helpful:

```rust
struct UsageStats {
    query: String,
    followed_up: bool,  // did user run suggested command?
    helpful: Option<bool>,  // explicit feedback
}

// Use to tune ranking weights over time
```

## Integration Points

### With Git
- Pre-commit hook: suggest documenting major changes
- Post-merge hook: wai why on merge conflicts

### With Editor/IDE
- LSP integration: hover → show why
- Quick actions: "Explain this file"

### With CI/CD
- PR comments: auto-explain changed files
- Documentation gate: require reasoning for new features

## Performance Considerations

### Indexing Strategy

```
On 'wai add':
  1. Extract keywords from artifact
  2. Update reverse index: keyword → artifacts
  3. Detect file mentions → update file → artifacts map

On 'wai why':
  1. Load relevant indexes (subset)
  2. Compute scores lazily
  3. Cache recent queries (15 min TTL)
```

### Scaling

- Small projects (<100 artifacts): in-memory, no indexing needed
- Medium projects (100-1000): file-based index, loaded on demand
- Large projects (1000+): SQLite index, prepared statements

## Open Design Decisions

1. **Should we support fuzzy file matching?**
   - User types `plugins` → matches `src/plugins/mod.rs`
   - Pro: more intuitive
   - Con: ambiguous, might match too many

2. **How to handle deleted files?**
   - Keep artifacts in archive
   - Show in results with "[deleted]" marker
   - Helpful for "why did we remove X?"

3. **Interactive mode?**
   - `wai why --interactive` starts REPL
   - Follow-up questions in context
   - Use previous query to refine

4. **Integration with LLM?**
   - `wai why --llm` sends to LLM for natural language summary
   - Local LLM or API key required
   - Privacy considerations

## Implementation Phases

**Phase 1**: MVP (File queries + artifact search)
- Query parser (file paths only)
- Artifact extractor (keyword search)
- Basic ranker (recency + keyword match)
- Summary output

**Phase 2**: Git integration
- Git extractor (blame, log, diff)
- Temporal correlation
- Commit message parsing

**Phase 3**: Intelligence
- Advanced ranking (all signals)
- Chain building
- Deduplication
- Suggestions

**Phase 4**: Advanced features
- AST parsing (function-level)
- Interactive mode
- LLM integration
- Editor plugins

## Success Metrics

How to know if oracle is effective:

1. **Adoption**: How often is `wai why` used?
2. **Precision**: Are top results relevant? (user feedback)
3. **Time-to-understanding**: How quickly can LLM/human grok reasoning?
4. **Coverage**: % of code with documented reasoning
5. **Efficiency**: Query latency <100ms for MVP, <500ms for advanced

## Conclusion

The 'wai why' oracle transforms wai from documentation archive to intelligent assistant. By synthesizing git history, artifacts, and code structure, it helps LLMs (and humans) quickly understand *why* the code exists as it does - the missing link between specs (what) and implementation (how).

