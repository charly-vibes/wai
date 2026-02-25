# Introduction

**wai** — pronounced like *"why"* — is a workflow manager for AI-driven development. It can also be read as *"way"*, and that's intentional: the tool's focus is capturing **why it was built that way**.

Most specs define *what* to build. Wai extends the workflow to also *inform* — preserving the research, reasoning, and decisions that shaped the design. When you revisit a project months later, the spec tells you what exists; wai tells you why.

It organizes artifacts using the PARA method (Projects, Areas, Resources, Archives) with project phase tracking, agent config sync, handoff generation, and plugin integration.

## At a Glance

`wai` bridges the gap between your thoughts and your codebase:

1. **Capture:** Record research, designs, and decisions as you work.
2. **Context:** Advance projects through defined phases (Research → Design → Plan → Implement).
3. **Sync:** Keep AI instructions (`CLAUDE.md`, `.cursorrules`) automatically in sync with your latest project state.
4. **Handoff:** Generate high-context session summaries for yourself or other agents.

## Design Principles

- **Desire Path Alignment** — Pave the cowpaths, make common workflows shortest
- **Self-Healing Errors** — Errors suggest fixes, not just report problems
- **Progressive Disclosure** — Simple by default, powerful when needed
- **Context-Aware** — Offer next steps based on current state
