Oracle-like 'why' command design - complementing 'wai way'

## Context

The 'wai way' command (in progress) shows **prescriptive** best practices.
The 'wai why' command would provide **descriptive** reasoning discovery.

## The Oracle Pattern for LLMs

What makes a command oracle-like for LLMs? It's not just search - it's **guided discovery**:

### 1. Contextual Awareness
- Git blame + artifacts → connect code to reasoning
- File/function awareness → surface relevant context
- Time-based correlation → match code dates to artifact dates

### 2. Reasoning Chains
- Decision lineage: 'This was chosen because X, which required Y'
- Alternatives considered and rejected
- Tradeoffs and constraints captured

### 3. Just-in-Time Context
- Code-triggered suggestions: 'You're in auth.rs - here's the auth design doc'
- Proactive rather than purely reactive
- Context surfaces when you need it

### 4. Question-Guided Search
Natural language queries that understand intent:

```bash
# Natural language
wai why "why use tokio instead of async-std?"

# File-based
wai why src/config.rs
wai why --file src/plugins/mod.rs --since 2024-01

# Feature-based
wai why --feature plugin-system
wai why --grep "phase tracking"

# Diff-based
wai why HEAD~3..HEAD
wai why --commit abc123

# Suggestion mode
wai why --here        # suggest based on current dir
wai why --suggest     # suggest based on recent changes
```

### 5. Multi-Modal Synthesis

Combines multiple sources:
- Git history (commits, blame, diffs)
- Artifacts (research, design, plans)
- Code comments and docs
- Dependencies and relationships

Prioritizes intelligently:
- Recent > old (exponential decay)
- Design > plan > research (by phase)
- Explicit > implicit (direct mentions > keyword overlap)

## Example Oracle Interaction

```
$ wai why src/plugins/mod.rs

📍 File: src/plugins/mod.rs (created 2024-12-15)

🔍 Found 3 relevant artifacts:

[1] Design: Plugin Architecture (2024-12-10) [design phase]
    → Chose trait-based plugins for type safety
    → Rejected dynamic loading (complexity/safety tradeoff)
    → Git plugin as reference implementation
    Relevance: 95% (file mentioned explicitly, overlapping dates)
    
[2] Research: Plugin Discovery (2024-12-12) [research phase]
    → Evaluated: glob patterns, TOML config, env vars
    → Chose TOML for explicit declaration
    Relevance: 82% (related keywords, prior to implementation)
    
[3] Plan: Plugin Implementation (2024-12-14) [plan phase]
    → Hook system for lifecycle events
    → Status() method for health checks
    Relevance: 78% (implementation details, same timeframe)

🔗 Related commits:
    abc123 (2024-12-15) feat: add plugin trait
    def456 (2024-12-16) feat: implement git plugin
    
💡 Next steps:
    • wai show plugin-architecture (full design doc)
    • wai why --grep 'hook system' (related decisions)
    • wai timeline why-command --from 2024-12-10 (chronological view)
```

## Implementation Strategy

### Phase 1: Smart Indexing
- Build reverse index: code files → artifacts
- Track: explicit mentions, keyword overlap, date ranges
- Update automatically on: wai add, wai sync

### Phase 2: Context Extraction
- Git blame for authorship + commit dates
- AST parsing for function/struct names (optional, later)
- Artifact text search for code references

### Phase 3: Ranking Algorithm

Relevance score = Σ(weights × signals):
- Recency: exp(-days_ago / 30) × 0.3
- Type priority: {design: 0.3, plan: 0.2, research: 0.15}
- Keyword match: jaccard_similarity × 0.25
- Phase context: in_same_phase × 0.1
- Explicit mention: is_mentioned × 0.2

### Phase 4: Progressive Disclosure
- Summary view (default): top 3-5 artifacts
- Detail view: --verbose for full artifact excerpts
- Drill-down: suggest related searches
- Export: --json for programmatic use

## Key Differentiators from 'wai search'

| Feature | wai search | wai why (oracle) |
|---------|-----------|------------------|
| Input | Keyword/regex | File, feature, question, or context |
| Output | Raw matches | Ranked, synthesized, explained |
| Sources | Artifacts only | Artifacts + git + code |
| Mode | Reactive | Proactive + reactive |
| Goal | Find text | Understand reasoning |

## Oracle Intelligence Features

### Relationship Mapping
- Track which artifacts reference which files
- Build dependency graph: artifact → artifact
- Detect reasoning chains automatically

### Temporal Intelligence
- Match code change dates to artifact dates
- Weight recent context higher
- Show evolution: 'Initially X, then Y, finally Z'

### Suggestion Engine
- Based on: recent commits, current directory, open files
- 'You might want to know why...'
- Learn from usage: track what was helpful

## Open Questions

1. **Performance**: How to index efficiently for large projects?
   - Incremental indexing on wai add
   - Cache computed relevance scores
   - Lazy load artifact content

2. **Accuracy**: How to avoid false positives?
   - Require minimum relevance threshold (60%?)
   - Show confidence scores
   - Allow user feedback to tune

3. **UX**: How to format dense information?
   - Default: compact summary
   - --verbose: full excerpts
   - Interactive: prompt to drill down

4. **Scope**: File-level or function-level granularity?
   - Start: file-level (simpler, faster)
   - Later: function-level with AST parsing

