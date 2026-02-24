# Change: Add LLM-Powered Reflection Command (`wai reflect`)

## Why

As projects accumulate sessions and artifacts (research, designs, plans, handoffs),
valuable project-specific conventions, gotchas, and architectural patterns remain
buried in session history. The CLAUDE.md and AGENTS.md managed blocks provide generic
tool instructions but contain nothing project-specific — every new AI session must
rediscover the same codebase patterns from scratch.

`wai why` answers specific questions reactively. `wai reflect` complements it
proactively: given accumulated session context, ask the LLM "what project-specific
context should AI assistants know that isn't already documented?" and inject the
answer into CLAUDE.md and/or AGENTS.md as a persistent, project-aware section.

There are three tiers of session context, from richest to most distilled:
1. **Conversation transcripts** — the raw Claude session: every failed command,
   every surprising discovery, every step that took three attempts. The most
   information-dense source but ephemeral unless captured.
2. **Handoffs** — deliberate session summaries. Good for intent and next steps,
   but often written quickly at session end when the tedious details are already
   fading.
3. **Research/design/plan artifacts** — curated decisions. Rich in rationale,
   sparse in operational patterns.

`wai reflect` accepts all three. Handoffs are currently the system's primary proxy
for conversation history — so this change also improves their quality: a richer
handoff template with guided nuance capture means better reflect output.

## What Changes

- **NEW command**: `wai reflect [--project <name>] [--conversation <file>]
  [--output claude.md|agents.md|both] [--dry-run] [--yes]`
  — synthesizes accumulated context, proposes additions to CLAUDE.md/AGENTS.md

- **NEW managed block marker**: `<!-- WAI:REFLECT:START -->` / `<!-- WAI:REFLECT:END -->`
  — a second managed section in CLAUDE.md (and/or AGENTS.md), separate from
  `<!-- WAI:START -->`, holding project-specific AI guidance

- **NEW**: `--conversation <file>` flag — accepts a plain-text conversation
  transcript export, treated as the highest-priority context source

- **IMPROVED**: `wai handoff create` gets two new template sections —
  `## Gotchas & Surprises` and `## What Took Longer Than Expected` — so the nuances
  that are hardest to re-discover make it into the reflect input

- **MODIFIED**: `wai close` nudges the user to run `wai reflect` after 5 or more
  sessions have accumulated since the last reflect run

- **NEW**: `.wai/projects/<project>/.reflect-meta` — small TOML file storing
  reflect run metadata (last_reflected date, session_count); avoids fragile
  CLAUDE.md mtime/content parsing for the session nudge heuristic

- **NEW capability**: `project-reflection`

## Impact

- **Affected specs**:
  - `cli-core` (MODIFIED — add `reflect` to the top-level command list)
  - `project-reflection` (ADDED — new capability)
  - `handoff-system` (MODIFIED — add Gotchas and Longer-Than-Expected sections)

- **Affected code**:
  - `src/commands/reflect.rs` — new command (~350 LOC)
  - `src/cli.rs` — add `Reflect` subcommand with flags
  - `src/managed_block.rs` — add REFLECT marker read/write/detect for both
    CLAUDE.md and AGENTS.md
  - `src/commands/close.rs` — add reflect nudge; read `.reflect-meta`
  - `src/commands/handoff.rs` — extend template with nuance sections
  - `src/commands/mod.rs` — add reflect module

- **Reuses** (no changes needed):
  - `src/llm.rs` — existing LLM abstraction (Claude/Ollama) + `WhyConfig`
  - Artifact reading pattern from `src/commands/why.rs`

- **Dependencies**: Requires `src/llm.rs` to be merged (already implemented in
  `add-why-command`; spec archiving not required)

- **Cost**: ~$0.005–$0.03 per run with Claude Haiku depending on conversation
  transcript size (rare, deliberate operation)

- **User impact**:
  - CLAUDE.md and AGENTS.md become progressively more useful as sessions accumulate
  - Conversation transcripts can be fed directly as the richest input source
  - Handoffs become better inputs with dedicated nuance-capture sections
  - Zero additional config: reuses existing `[why]` LLM settings
  - Safe by design: shows diff of old vs new REFLECT content before writing;
    `--dry-run` previews without touching any file
