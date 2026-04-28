# Ubiquitous Language

Domain-driven design talks about a *ubiquitous language* — a shared vocabulary that developers and domain experts use consistently in code, conversations, and documentation. Without it, agents guess at terminology and hallucinate synonyms: is "order" the same as "purchase"? Does "customer" overlap with "user"? Is "shipment" the same as "delivery"?

Wai gives teams a first-class home for this vocabulary under `.wai/resources/ubiquitous-language/`.

## The Directory Layout

```
.wai/resources/ubiquitous-language/
├── README.md              # navigation index (required)
├── shared.md              # cross-context terms (optional)
└── contexts/
    ├── orders.md
    ├── inventory.md
    └── billing.md
```

Three layers:

- **`README.md`** — a lightweight index that describes the tree, lists available bounded contexts (self-contained parts of the domain with their own consistent terminology, e.g. Orders, Billing), and tells agents how to navigate it. This is the only file agents read by default.
- **`shared.md`** — optional. Holds terms that are genuinely shared across multiple bounded contexts. Keep it short; most terms belong in a context file.
- **`contexts/*.md`** — one file per bounded context. Each file records the preferred term, a concise definition, discouraged synonyms or anti-terms, and related terms. Each bounded context should be a single `.md` file directly under `contexts/`. Subdirectories and non-Markdown files are not currently recognized by `wai way`.

## Getting Started

Create the directory and a minimal index:

```bash
mkdir -p .wai/resources/ubiquitous-language/contexts
```

Then write `README.md` as a short navigation guide:

```markdown
# Ubiquitous Language

This directory contains domain terminology for this project.

## Bounded Contexts

- [Orders](contexts/orders.md) — fulfillment, line items, cancellation
- [Inventory](contexts/inventory.md) — SKUs, stock levels, reservations

## Usage

Read this file first. Open only the context file(s) relevant to your task.
```

Add a bounded-context file when you encounter ambiguous or domain-specific terms:

```markdown
# Orders Context

## Order

**Definition:** A customer's confirmed intent to purchase one or more items.

**Anti-terms:** Do not use "cart" or "basket" — those refer to the pre-checkout state, not a confirmed order.

**Related:** Line Item, Fulfillment, Cancellation
```

Run `wai way` after setup to confirm the tree is fully configured.

## Progressive Disclosure

Agents work better when they load terminology on demand rather than dumping an entire glossary into context. The layout is designed for this:

1. Agent reads `README.md` to learn what contexts exist.
2. Agent opens only the bounded-context file(s) relevant to the current task.
3. Agent ignores unrelated context files entirely.

The managed block injection (see below) is what makes agents aware of this pattern — the directory layout alone is not enough. Running `wai sync` is the step that teaches agents to navigate it correctly.

This matters in large codebases with many domains — loading every term file wastes context and increases noise.

## `wai way` Detection

`wai way` checks the state of your ubiquitous-language tree and surfaces it alongside other project health signals:

| State | Result |
|---|---|
| `README.md` + at least one `contexts/*.md` | `PASS` — fully configured |
| `README.md` + `shared.md`, no context files | `INFO` — valid skeleton, add context files |
| `README.md` only | `INFO` — needs bounded-context files |
| Directory exists, no `README.md` | `INFO` — index required first |
| Directory absent | `INFO` — not yet set up, with suggestion |

Run `wai way` to see the result alongside other project health checks.

## Managed Block Injection

When `wai sync` regenerates your `CLAUDE.md`, `AGENTS.md`, or `.wai/AGENTS.md` managed blocks, it checks whether `README.md` exists. If it does, it injects a usage note:

```
## Ubiquitous Language

If `.wai/resources/ubiquitous-language/README.md` exists, read it first as the
navigation index, then open only the bounded-context files relevant to the task.
Avoid loading every terminology file unless the work truly spans multiple contexts.
```

Every agent reading your `CLAUDE.md` will automatically know where the vocabulary lives and how to use it without you having to explain it in each session prompt.

## Maintenance Workflow

Use the `ubiquitous-language` skill template to scaffold a skill that keeps the tree up to date:

```bash
wai add skill update-terminology --template ubiquitous-language
```

The generated skill guides an agent to:

1. Search existing artifacts and code for domain terminology.
2. Identify the appropriate bounded context for each term.
3. Update `README.md`, `shared.md`, or the relevant `contexts/*.md` file.
4. Prefer incremental edits over rewriting the whole tree.

Run the skill whenever new domain concepts emerge or existing terms need clarification. Invoke it directly with `wai skill run update-terminology` or by referencing the generated `SKILL.md` in your agent prompt.

## See Also

- [`wai way`](../commands.md) — project health check command that detects the ubiquitous-language tree state
- [Managed Blocks](managed-blocks.md) — how `wai sync` injects guidance into `CLAUDE.md` and `AGENTS.md`
- [`wai add skill`](../commands.md) — scaffold skill templates including `ubiquitous-language`
