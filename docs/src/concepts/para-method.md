# PARA Method

> **Why PARA?** Without structure, project artifacts end up scattered across flat directories, wikis, Slack threads, and local notes. Finding that design decision from three weeks ago becomes an archaeological dig. PARA gives every artifact a clear home based on its lifecycle stage — active work goes in Projects, ongoing responsibilities in Areas, reference material in Resources, and completed work in Archives. You always know where to look.

Wai organizes artifacts using the **PARA** method — a system for organizing digital information into four categories:

```
.wai/
├── projects/          ← Active work with a goal and phases
│   ├── auth-feature/     research/ designs/ plans/ handoffs/
│   └── api-redesign/     research/ designs/ plans/ handoffs/
├── areas/             ← Ongoing responsibilities (no end date)
│   └── security/
├── resources/         ← Reference material and reusable assets
│   ├── agent-config/     skills/ rules/ context/
│   ├── reflections/
│   └── pipelines/
└── archives/          ← Completed items (searchable, hidden from status)
    └── old-feature/
```

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
