# Reasoning

Capturing artifacts — research notes, design decisions, plans — is the foundation. But artifacts alone don't answer the questions that come up later: *Why did we choose this approach? What patterns have emerged across sessions? What does this file's history tell us?*

Wai's reasoning features bridge this gap with two LLM-powered commands that query and synthesize your captured knowledge.

## `wai why` — The Reasoning Oracle

`wai why` answers natural-language questions about your project by combining your artifacts with git history and feeding them to an LLM.

### How It Works

1. **Context gathering** — Wai collects relevant PARA artifacts (research, designs, plans) and recent git history, scoped to your query.
2. **LLM query** — The gathered context and your question are sent to the configured LLM backend.
3. **Structured response** — The answer comes back with a decision chain and references to the artifacts that informed it.

### Query Types

You can ask `wai why` two kinds of questions:

```bash
# Ask a question about your project
wai why "Why did we choose TOML over YAML for config?"

# Ask about a specific file's history and purpose
wai why src/config.rs
```

Questions work best when they're specific and tied to decisions your project has already made. The oracle is only as good as the artifacts you've captured — it queries what you recorded.

### When It Shines

- **Onboarding**: A new contributor asks "why does the plugin system use auto-detection?" and gets an answer grounded in actual design artifacts, not guesses.
- **Revisiting decisions**: Six months later, you wonder why a particular architecture was chosen. The oracle traces it back through research and design notes.
- **File archaeology**: Pointing `wai why` at a file explains its role in the project based on commit history and related artifacts.

## `wai reflect` — Project Synthesis

Where `wai why` answers specific questions, `wai reflect` steps back and synthesizes patterns across your entire project history.

### How It Works

1. **Context aggregation** — Wai gathers conversation transcripts, handoff documents, and session history up to a character budget.
2. **Pattern synthesis** — An LLM distills the accumulated context into recurring patterns, conventions, and architectural notes.
3. **Persistent output** — The synthesis is written as a versioned Markdown file in `.wai/resources/reflections/`.
4. **Instruction updates** — Wai detects instruction files (`CLAUDE.md`, `AGENTS.md`) and updates them with the new reflection, so agents pick up the patterns automatically.

### When to Run It

Run `wai reflect` approximately every 5 sessions, or whenever you feel the project has accumulated enough new context to be worth synthesizing. It's not meant to run after every change — it's a periodic distillation.

```bash
wai reflect
```

### What It Produces

The output is a reflection file containing:

- **Project conventions** — Naming patterns, code style preferences, framework choices that emerged organically
- **Architectural notes** — How components relate, where boundaries are, what invariants matter
- **Gotchas** — Things that tripped you up and shouldn't trip someone else

These reflections feed back into agent instructions via managed blocks, so the next agent session starts with accumulated project wisdom — not just the raw artifacts.

## LLM Configuration

Both commands support multiple backends:

- **Claude API** — Uses the Anthropic REST API directly (requires `ANTHROPIC_API_KEY`)
- **Claude CLI** — Delegates to the `claude` binary
- **Ollama** — Local LLM for offline use

Wai auto-detects the available backend. No configuration is needed if you have one of these set up.

## How They Build on Artifacts

Both `wai why` and `wai reflect` are only as useful as the artifacts you've captured. They don't hallucinate project context — they query and synthesize what exists in your `.wai/` directory and git history.

This creates a virtuous cycle:

1. You capture research, designs, and decisions with `wai add`
2. `wai why` makes that knowledge queryable on demand
3. `wai reflect` distills it into persistent patterns
4. Those patterns inform the next session via managed blocks
5. The next session captures more artifacts, and the cycle continues
