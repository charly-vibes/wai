# Proposal: refactor-reflect-to-resource

## Summary

Move `wai reflect` output from embedded blocks in `CLAUDE.md`/`AGENTS.md` into
versioned resource files under `.wai/resources/reflections/`. Replace the full
embedded block with a slim reference that tells agents where to look and — crucially
— instructs them to search for known patterns before starting research or creating
tickets.

## Problem

The current `WAI:REFLECT:START/END` block injects the full reflection content
directly into `CLAUDE.md`. This has three problems:

1. **CLAUDE.md bloat** — reflections accumulate in a file that is already the
   primary instruction surface for AI agents. Each reflect run overwrites the
   previous block, discarding historical patterns.

2. **No history** — there is no way to see how patterns evolved across sessions,
   or which session introduced a particular gotcha.

3. **Discovery mismatch** — the patterns are loaded passively into context on
   every session whether or not they are relevant. A searchable resource file
   lets agents pull relevant patterns on demand.

## Proposed Solution

### Storage: `.wai/resources/reflections/`

`wai reflect` writes each run to a new dated file:
```
.wai/resources/reflections/YYYY-MM-DD-<project>.md
```

Files accumulate across sessions. They are committed, versioned, and searchable
via `wai search`. The existing `WAI:REFLECT:START/END` block in `CLAUDE.md` and
`AGENTS.md` is removed and replaced by a slim reference block.

### Reference in CLAUDE.md / AGENTS.md

The `WAI:REFLECT:REF` block (generated and managed by `wai init`, not by
`wai reflect`) contains only:

```markdown
<!-- WAI:REFLECT:REF:START -->
## Accumulated Project Patterns

Project-specific conventions, gotchas, and architecture notes live in
`.wai/resources/reflections/`. Run `wai search "<topic>"` to retrieve relevant
context before starting research or creating tickets.

> **Before research or ticket creation**: always run `wai search "<topic>"` to
> check for known patterns. Do not rediscover what is already documented.
<!-- WAI:REFLECT:REF:END -->
```

### Managed Block Integration

The `wai init` managed block template (`src/managed_block.rs`) gains a
"Before you start" instruction that applies project-wide:

> When beginning research or creating a ticket, run
> `wai search "<topic>"` to check for existing patterns before writing new content.

This appears when any companion tool (beads, openspec) is detected — i.e., in
projects where agents do implementation work. The instruction intentionally
appears in both the `WAI:START/END` managed block and the `WAI:REFLECT:REF`
block — the managed block ensures it reaches new repos immediately via
`wai init`; the REF block reinforces it specifically in the reflections context.

### Migration

On first run after the change, `wai reflect` detects an existing `WAI:REFLECT`
block in `CLAUDE.md` or `AGENTS.md`, migrates its content to
`.wai/resources/reflections/<today>-<project>-migrated.md`, replaces the block
with the slim `WAI:REFLECT:REF` reference, and prints a migration notice.

## Out of Scope

- Changing the core LLM prompt structure (context sources are extended to
  include previous reflections, but the prompt format is unchanged)
- Merging or deduplicating historical reflection files
- Extending `wai search --project` to include shared resources (global search
  already covers `.wai/resources/reflections/` without `--project`)
- Automatic reflection scheduling
