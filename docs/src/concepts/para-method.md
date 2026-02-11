# PARA Method

Wai organizes artifacts using the **PARA** method — a system for organizing digital information into four categories:

## Projects

Active work with a defined goal and deadline. Each project has its own directory under `.wai/projects/` containing research notes, plans, designs, and handoff documents.

```
.wai/projects/my-app/
├── .state           # Phase state machine (YAML)
├── research/        # Date-prefixed research notes
├── plans/           # Date-prefixed plan documents
├── designs/         # Date-prefixed design documents
└── handoffs/        # Date-prefixed handoff documents
```

## Areas

Ongoing areas of responsibility with no end date. These live under `.wai/areas/`.

## Resources

Reference material and reusable assets. Stored under `.wai/resources/`:

```
.wai/resources/
├── agent-config/    # Single source of truth for agent configs
│   ├── skills/
│   ├── rules/
│   ├── context/
│   └── .projections.yml
├── templates/
└── patterns/
```

## Archives

Completed or inactive items moved here for reference. Located at `.wai/archives/`.

## Moving between categories

Use `wai move` to reclassify items as their status changes:

```bash
wai move my-app archives    # Archive a completed project
```
