# Plugins Context

## Plugin

**Definition:** An integration module that connects wai with an external tool (git, beads, openspec). Plugins are auto-detected based on workspace markers (e.g., `.git/`, `.beads/`). They contribute hooks, passthroughs, and status context.

**Anti-terms:** Do not call it an "integration" or "extension" — "plugin" is the canonical term.

**Related:** Hook, Passthrough, Projection

---

## Hook

**Definition:** A plugin extension point triggered during wai operations. Hooks like `on_status` and `on_handoff_generate` let plugins inject context into wai's output.

**Anti-terms:** Do not confuse with git hooks or oracle scripts — a hook is a plugin callback within wai's lifecycle.

**Related:** Plugin

---

## Passthrough

**Definition:** A plugin command that delegates directly to the underlying tool. For example, `wai beads list` passes through to the beads CLI.

**Anti-terms:** Do not say "proxy" or "delegate" — "passthrough" is the canonical term.

**Related:** Plugin

---

## Projection

**Definition:** A sync strategy that maps source files in `.wai/resources/agent-config/` to tool-specific target locations. Strategies include `symlink`, `inline`, and `reference`. Managed by `wai sync`.

**Anti-terms:** Do not call it a "copy" or "export" — "projection" implies a live mapping, not a one-time copy.

**Related:** Plugin, Managed block

---

## Managed block

**Definition:** An auto-generated section in instruction files (`AGENTS.md`, `CLAUDE.md`) delimited by `<!-- WAI:START -->` / `<!-- WAI:END -->` markers. Updated automatically by `wai init` and `wai reflect`. Must not be edited manually.

**Anti-terms:** Do not call it a "section" or "generated comment" — "managed block" is the canonical term.

**Related:** Projection, Reflection
