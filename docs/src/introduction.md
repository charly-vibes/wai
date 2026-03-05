# Introduction

**wai** — pronounced like *"why"* — is a workflow manager for AI-driven development. It can also be read as *"way"*, and that's intentional: the tool's focus is capturing **why it was built that way**.

Most specs define *what* to build. Wai extends the workflow to also *inform* — preserving the research, reasoning, and decisions that shaped the design. When you revisit a project months later, the spec tells you what exists; wai tells you why.

It organizes artifacts using the PARA method (Projects, Areas, Resources, Archives) with project phase tracking, agent config sync, handoff generation, and plugin integration.

## At a Glance

`wai` bridges the gap between your thoughts and your codebase:

- **🧠 Capture** — Record research, designs, and decisions as you work.
- **🗺️ Context** — Advance projects through phases (Research → Design → Plan → Implement).
- **🔄 Sync** — Keep AI instructions (`CLAUDE.md`, `.cursorrules`) automatically in sync.
- **📦 Handoff** — Generate high-context session summaries for yourself or other agents.

## Key Capabilities

- **PARA Method Storage**: Logical organization into Projects, Areas, Resources, and Archives.
- **Phase Tracking**: Structured workflow from initial research to final archive.
- **Sync Engine**: Single source of truth for agent configurations across multiple tools.
- **Plugin System**: Seamless integration with `git`, `beads`, `openspec`, and more.
- **Reasoning Oracle**: Ask natural language "why" questions about your codebase.

## Design Principles

- **🛣️ Desire Path Alignment** — Pave the cowpaths; make common workflows shortest.
- **🩹 Self-Healing Errors** — Errors suggest fixes, not just report problems.
- **🌊 Progressive Disclosure** — Simple by default, powerful when needed.
- **💡 Context-Aware** — Offer smart next steps based on your current project state.
