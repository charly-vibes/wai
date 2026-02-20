# Change: Add Plugin-Aware AGENTS.md Generation

## Why

When `wai init` runs, it injects a managed block into `AGENTS.md` and `CLAUDE.md`. This block is currently **static** — identical content regardless of whether beads, openspec, or git are detected alongside wai. This means agents get no guidance on when to use wai vs companion tools, session protocols conflict between tools, and the managed block doesn't acknowledge the broader tool landscape at all. In projects using wai + beads + openspec together, agents must piece together three siloed instruction sets with no integration story.

## What Changes

- Make `wai_block_content()` accept detected plugins and generate **context-aware** managed block content
- When companion tools are detected, include a "tool landscape" section with clear boundaries (wai = why, beads = what, openspec = specs)
- Add a unified "When to Use What" decision table conditional on detected plugins
- Consolidate session start/end protocols so agents get one coherent flow instead of competing instructions
- Pass detected plugin info through `inject_agent_instructions()` to the managed block generator
- Keep the managed block self-contained: agents should understand the tool landscape from AGENTS.md alone without needing to read three separate sources first

## Impact

- Affected specs: `agent-config-sync` (adds requirement for plugin-aware managed block injection)
- Affected code: `src/managed_block.rs` (block content generation), `src/commands/init.rs` (plugin context plumbing)
- No breaking changes: the `<!-- WAI:START -->` / `<!-- WAI:END -->` marker protocol is unchanged; content between markers is replaced as before
