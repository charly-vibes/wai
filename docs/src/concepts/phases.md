# Project Phases

Projects in wai progress through six phases:

```
research → design → plan → implement → review → archive
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
