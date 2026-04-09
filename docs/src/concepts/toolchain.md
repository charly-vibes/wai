# Toolchain Synergy

Wai is designed to work alongside two companion tools — **beads** (issue tracking) and **openspec** (specification management). Each tool owns a distinct concern:

| Tool | Owns | Question it answers |
|------|------|---------------------|
| **wai** | Reasoning and context | *Why* was this decision made? |
| **bd** (beads) | Tasks and work items | *What* needs to be done? |
| **openspec** | Specifications and proposals | *What should the system look like?* |

## When to Use What

| I need to... | Use |
|---|---|
| Record why I chose approach X over Y | `wai add research "..."` |
| Track a bug or task | `bd create --title="..."` |
| Propose a system change with requirements | `openspec create <id>` |
| Resume where I left off | `wai prime` |
| Find available work | `bd ready` |
| Validate a proposal is complete | `openspec validate --strict` |
| Search past decisions | `wai search "..."` |
| Close a completed task | `bd close <id>` |
| Archive a deployed change | `openspec archive <id>` |

## How They Integrate

The three tools are connected through wai's [plugin system](./plugins.md):

- **beads** is detected automatically when `.beads/` exists. Open issue counts appear in `wai status`, and `wai handoff create` includes issue context in handoff documents.
- **openspec** is detected when `openspec/` exists. Active change proposals and their progress appear in `wai status`.
- **Cross-references** tie them together: beads tickets reference openspec tasks (e.g., `add-why-command:7.1` in the description), and completing a beads ticket means checking the box in the openspec `tasks.md`.

## Worked Example: Adding a Feature

Here's how a ticket flows through all three tools:

```bash
# 1. Propose the change in openspec
openspec create add-search-filters
# Edit the spec: requirements, acceptance criteria, task breakdown

# 2. Create beads tickets for the implementation tasks
bd create --title="Add --tag flag to search" --description="add-search-filters:3.1"
bd create --title="Add --type flag to search" --description="add-search-filters:3.2"

# 3. Research the approach in wai
wai add research "Evaluated regex vs glob for tag matching — chose glob for simplicity"

# 4. Work the ticket
bd update wai-abc1 --status in_progress
wai add design "Tags stored in YAML frontmatter, filtered at search time"

# 5. Close the loop
bd close wai-abc1
# Check [x] for task 3.1 in openspec's tasks.md
```

## What You Lose by Skipping One

- **Without wai**: Tasks get done, but nobody remembers *why* decisions were made. Six months later, the code is a black box.
- **Without beads**: Reasoning is captured, but there's no task decomposition, no dependency tracking, and no way to find available work.
- **Without openspec**: Changes happen ad hoc — no requirements, no acceptance criteria, no validation that the system matches the spec.

Each tool is optional. Wai works fine alone. But together, they cover the full lifecycle from proposal to implementation to archival.
