# Reflection Command Idea Analysis

## Core Concept
User idea: Use LLM to inspect its own 'conversations' (sessions) and understand usage patterns, then inject project-specific learnings into CLAUDE.md/AGENTS.md for better codebase orientation.

## Key Insight: Conversations = Handoffs
Claude Code conversations are ephemeral — transcripts aren't persisted. But wai already captures distilled session context:
- **Handoffs** (.wai/projects/*/handoffs/) — session summaries, next steps, phase context
- **Research/Design/Plan** artifacts — explicit learnings captured mid-session
- **Git commits** — what changed and why (implicit session output)

Handoffs are the closest proxy for 'conversation history'. They're designed to transfer context across /clear boundaries.

## Gap Being Addressed
Current CLAUDE.md managed block (WAI:START/END) contains generic tool instructions:
- How to use wai, beads, openspec
- Phase definitions
- Standard workflow patterns

What's MISSING is **project-specific** context:
- This codebase's conventions (how Rust modules are organized, naming patterns)
- Recurring gotchas (what tripped up previous sessions)
- Architecture decisions that keep getting re-explained
- Common tasks and their correct approach in this specific project

## Proposed Command: wai reflect
1. Reads accumulated handoffs + research/design artifacts
2. Reads current CLAUDE.md to avoid duplication
3. Uses LLM prompt: 'What project-specific patterns, conventions, gotchas should be documented for AI assistants? What keeps coming up that isn't already covered?'
4. Proposes additions in a new managed block: <!-- WAI:REFLECT:START --> / <!-- WAI:REFLECT:END -->
5. Shows diff + asks confirmation before writing

## How It Differs from wai why
- wai why: answers a specific QUESTION about past decisions
- wai reflect: proactively SURFACES patterns without a specific question
- wai why: reactive, query-driven
- wai reflect: proactive, pattern-finding

## Implementation Approach
### New files:
- src/commands/reflect.rs (new command ~300 LOC)

### Modified files:
- src/cli.rs (add Reflect subcommand)
- src/managed_block.rs (add REFLECT marker type + injection logic)
- src/commands/mod.rs (add reflect module)

### Reuses:
- src/llm.rs (existing LLM abstraction)
- Context gathering from why.rs (artifact reading pipeline)

## The Reflect Block
New marker in CLAUDE.md:
<!-- WAI:REFLECT:START -->
## Project-Specific Context
Last reflected: YYYY-MM-DD (N sessions analyzed)
### Conventions
- ...
### Common Gotchas
- ...
### Architecture Notes
- ...
<!-- WAI:REFLECT:END -->

Separate from WAI:START/END (tool instructions vs project learnings).

## When to Run
- Manual: wai reflect (any time)
- Prompted: wai close suggests it after 5+ sessions without reflecting
- --dry-run flag: show proposed changes without writing
- --force flag: skip confirmation

## Trade-offs
- Requires LLM (same dependency as wai why)
- Quality depends on quality of handoffs — incentivizes better handoff capture
- May produce generic/useless content if artifacts are sparse
- Risk: LLM hallucinating 'patterns' that aren't real → user confirmation gate is essential

