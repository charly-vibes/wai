# Why Wai?

**wai** — pronounced like *"why"* — is a workflow manager for AI-driven development. It can also be read as *"way"*, and that's intentional: the tool's focus is capturing **why it was built that way**.

## The Problem

You come back Monday morning. The code is there — merged, deployed, working. But nobody remembers *why* approach X was chosen over Y. The research that ruled out three alternatives? Lost in a chat thread. The design constraint that shaped the API? Buried in a meeting note nobody bookmarked. The trade-off that made the architecture click? It lived in someone's head, and they moved on.

This happens constantly in AI-assisted development, where code is produced fast but reasoning evaporates even faster:

- **Context resets every session.** AI agents start fresh each time. The research you did yesterday is gone unless you manually re-explain it.
- **Decisions outlive their rationale.** Code survives; the thinking behind it doesn't. Six months later, nobody can explain why the system works the way it does.
- **Handoffs lose signal.** Switching between agents, or between human and agent, means re-discovering context that already existed.
- **Config drift.** AI instructions (`CLAUDE.md`, `.cursorrules`) fall out of sync across tools, so agents get inconsistent guidance.

## How Wai Helps

Wai addresses each of these pain points directly:

| Pain point | How wai solves it |
|---|---|
| Context resets every session | `wai close` captures session state; `wai prime` restores it — agents resume where they left off. |
| Decisions outlive their rationale | `wai add research`, `wai add design`, and `wai add plan` preserve the *why* alongside the *what*. |
| Handoffs lose signal | `wai handoff create` generates a high-context summary that carries reasoning across sessions and agents. |
| Config drift | The sync engine keeps agent configs aligned from a single source of truth. |

## At a Glance

`wai` bridges the gap between your thoughts and your codebase:

- **🧠 Capture** — Record research, designs, and decisions as you work.
- **🗺️ Context** — Advance projects through phases (Research → Design → Plan → Implement → Review → Archive).
- **🔄 Sync** — Keep AI instructions automatically aligned from one source of truth.
- **📦 Handoff** — Generate high-context session summaries for yourself or other agents.

## How It Fits Together

```
┌──────────────────────────────────────────────────────────┐
│                        .wai/                             │
│                                                          │
│  ┌─ Projects ──┐  ┌─ Areas ──┐  ┌─ Resources ─────────┐ │
│  │ my-feature/ │  │ ops/     │  │ agent-config/        │ │
│  │  research/  │  │ docs/    │  │   skills/ rules/     │ │
│  │  designs/   │  └──────────┘  │ reflections/         │ │
│  │  plans/     │                │ pipelines/           │ │
│  │  handoffs/  │  ┌─ Archives ┐ └──────────────────────┘ │
│  └─────────────┘  │ old-proj/ │                          │
│                   └───────────┘                          │
├──────────────────────────────────────────────────────────┤
│  Phases: research → design → plan → implement → review   │
│  Plugins: git · beads · openspec · custom                │
│  Sync: .wai/resources/ ──→ .claude/ .cursorrules etc.    │
└──────────────────────────────────────────────────────────┘
```

## Key Capabilities

- **PARA Method Storage**: Logical organization into Projects, Areas, Resources, and Archives.
- **Phase Tracking**: Structured workflow from initial research to final archive.
- **Sync Engine**: Single source of truth for agent configurations across multiple tools.
- **Plugin System**: Seamless integration with `git`, `beads`, `openspec`, and more.
- **Reasoning Oracle**: Ask natural language "why" questions about your codebase with `wai why`.

## Where to Go Next

Depending on your role, different parts of the docs will be most useful:

- **👤 Adopting wai in your own repo?** Start with the [Quick Start](./quick-start.md), then read [How to Adopt Wai](./how-to/adopt-wai.md) for a step-by-step guide.
- **🔧 Contributing to wai itself?** This repo dogfoods wai — it uses wai, beads, and openspec to manage its own development. See [Development](./development.md) and [Architecture](./architecture.md), and read `AGENTS.md` for the session workflow.
- **🤖 AI assistant?** Follow the instructions in `AGENTS.md` at the repo root. Key commands: `wai prime` (session start), `wai close` (session end), `bd ready` (find work).

## Design Principles

- **🛣️ Desire Path Alignment** — Pave the cowpaths; make common workflows shortest.
  `wai status` suggests the next command based on your current phase — if you're in research, it nudges you toward `wai add research`; when research is captured, it suggests advancing to design.

- **🩹 Self-Healing Errors** — Errors suggest fixes, not just report problems.
  `wai doctor --fix` auto-repairs common workspace issues. Typos in commands get "Did you mean?" suggestions. Error messages include the exact command to fix the problem.

- **🌊 Progressive Disclosure** — Simple by default, powerful when needed.
  `wai init` asks nothing beyond a project name. `-v` shows more detail, `-vv` reveals internals, `-vvv` shows everything. Beginners see clean output; power users get full diagnostics.

- **💡 Context-Aware** — Offer smart next steps based on your current project state.
  Wai auto-detects your toolchain (git, beads, openspec) by scanning for workspace markers and adapts its suggestions accordingly — no configuration required.

---

## A note on authorship

All the code in this repository was generated by a large language model. This is not a confession, nor an apology. It's a fact, like the one that says water boils at a hundred degrees at sea level: neutral, technical, and with consequences one discovers later.

What the human did is what tends to happen before and after things come into existence: thinking. Reviewing requirements, arguing about edge cases, understanding what needs to be built and why, deciding how the system should behave when reality —which is capricious and does not read documentation— confronts it with situations nobody anticipated. The hours of planning, of design, of reading specifications until exhaustion dissolves the boundary between understanding and hallucination.

The LLM writes. The human knows what it should say.

There is a distinction, even if looking at the commit history makes it hard to find. The distinction is that a machine can produce correct code without understanding anything, the same way a calculator can solve an integral without knowing what time is. Understanding what that integral is *for*, whether it actually solves the problem, whether the problem was the right problem to begin with — that remains human territory. For now.

*[Leer en español](https://charly-vibes.github.io/charly-vibes/)*
