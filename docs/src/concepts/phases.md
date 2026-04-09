# Project Phases

> **Why phases?** When teams skip straight to code, they build the wrong thing — or the right thing the wrong way. Decisions get made implicitly, trade-offs go unrecorded, and the first sign of trouble triggers a rewrite. Formalizing research → design → plan → implement → review → archive doesn't slow you down; it forces the reasoning to exist *before* the code, so when you revisit the project later, the *why* is already captured.

Projects in wai progress through six phases:

```
          ┌──────────────────────────────────────────┐
          │                                          │
          ▼                                          │
  ┌──────────┐   ┌────────┐   ┌──────┐   ┌─────────────┐   ┌────────┐   ┌─────────┐
  │ research │──▶│ design │──▶│ plan │──▶│ implement   │──▶│ review │──▶│ archive │
  └──────────┘   └────────┘   └──────┘   └─────────────┘   └────────┘   └─────────┘
       ▲              ▲                        │
       └──────────────┴────────────────────────┘
                  (back-transitions allowed)
```

## Phases

1. **Research** — Gather information, explore the problem space, understand constraints
2. **Design** — Define architecture, make key decisions, document trade-offs
3. **Plan** — Break work into tasks, define milestones, set priorities
4. **Implement** — Build the solution, write code, integrate components
5. **Review** — Validate the work, run tests, gather feedback
6. **Archive** — Complete the project, generate final handoff, move to archives

## Managing phases

```bash
wai phase              # Show current phase
wai phase next         # Advance to next phase
wai phase back         # Return to previous phase
wai phase set design   # Jump to a specific phase
```

## Flexibility

Transitions are flexible — skip forward or go back as needed. Phase history is tracked with timestamps so you can see how a project evolved over time.

## Adding artifacts by phase

Each phase has associated artifact types. Wai encourages capturing the right kind of documentation at each stage:

- **Research** phase → `wai add research "..."`
- **Design** phase → `wai add design "..."`
- **Plan** phase → `wai add plan "..."`

## Archive Phase vs. Category

The word "archive" is used in two ways in wai, and they serve different purposes:

### 1. The Archive **Phase** (`wai phase set archive`)
This is a **status** for a project. It indicates that the project is complete, all final handoffs are generated, and no further work is planned. It stays in its current PARA category (usually **Projects**).

**Use when:** A project is finished but you still want it to appear in `wai status` and your active workspace overview.

### 2. The Archives **Category** (`wai move my-project archives`)
This is a **storage location**. Moving an item to archives reclassifies it in the PARA system, moving its directory from `.wai/projects/` to `.wai/archives/`.

**Use when:** You want to declutter your workspace. Archived items are hidden from default `wai status` views and most context suggestions, but remain fully searchable.

**Recommended Workflow:**
1. Set phase to `archive` when the work is done.
2. Generate a final handoff (`wai close`).
3. Move to the `archives` category when you no longer need it in your active projects list.
