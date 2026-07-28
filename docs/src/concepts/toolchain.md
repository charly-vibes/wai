# Toolchain Synergy

Wai is the suite orchestrator for the full [charly-vibes](https://github.com/charly-vibes) toolchain — eight tools that together cover the
complete lifecycle from proposal to implementation to archival. Each tool owns
a distinct concern and is detected automatically by its on-disk marker:

| Tool | Owns | Detection Signal | Question it answers |
|------|------|------------------|---------------------|
| **wai** | Reasoning and context | `.wai/` | *Why* was this decision made? |
| **bd** (beads) | Tasks and work items | `.beads/` | *What* needs to be done? |
| **openspec** | Specifications and proposals | `openspec/` | *What should the system look like?* |
| **pretender** | Structural code quality (AST linting) | `pretender.toml` | *Does the code match our rules?* |
| **dont** | Decision-logged conventions | `.dont/` | *What have we agreed not to do?* |
| **espectacular** | Spec-to-test correspondence | `.espectacular/` | *Does the test match the spec?* |
| **testaruda** | Test harness & property testing | `testaruda.toml` | *Is the behavior correct?* |
| **vampiro / crua / livin** | Spec-stage static analysis (planned) | TBD | *Does the design hold up under analysis?* |

All tools live under the [charly-vibes](https://github.com/charly-vibes) GitHub organisation except:

- **beads** — [gastownhall/beads](https://github.com/gastownhall/beads) (issue tracking with Dolt-backed sync).
- **openspec** — [openspecio/openspec](https://github.com/openspecio/openspec) (specification management lifecycle).

## When to Use What

| I need to... | Use |
|---|---|
| Record *why* I chose approach X over Y | `wai add research "..."` |
| Track a bug or task | `bd create --title="..."` |
| Propose a system change with requirements | `openspec create <id>` |
| Lint code for structural quality | `pretender check .` |
| Enforce a team convention with a decision log | `dont define` |
| Verify spec-to-test correspondence | `ah check` (espectacular) |
| Run property or harness tests | `testaruda check` |
| Resume where I left off | `wai prime` |
| Find available work | `bd ready` |
| Validate a proposal is complete | `openspec validate --strict` |
| Search past decisions | `wai search "..."` |
| Close a completed task | `bd close <id>` |
| Archive a deployed change | `openspec archive <id>` |

## How They Integrate

All tools connect through wai's [plugin system](./plugins.md), which
auto-detects each tool by its on-disk marker:

- **beads** — detected when `.beads/` exists. Open issue counts appear in
  `wai status`, and `wai handoff create` includes issue context in handoff
  documents.
- **openspec** — detected when `openspec/` exists. Active change proposals
  and their progress appear in `wai status`.
- **pretender** — detected when `pretender.toml` exists. Wai can help
  bootstrap its config via `wai way`.
- **dont** — detected when `.dont/` exists. Dont managed blocks in `AGENTS.md`
  are refreshed by `wai sync`.
- **espectacular** — detected when `.espectacular/` exists. Espectacular
  signals (spec-test gaps) are surfaced in `wai status`.
- **testaruda** — detected when `testaruda.toml` exists. Test harness state
  is reported in `wai status`.
- **vampiro / crua / livin** — detection signals to be defined as the tools
  ship. The trio's state will appear in `wai status` once detected.

**Cross-references** tie them together: beads tickets reference openspec tasks
(e.g., `add-why-command:7.1` in the description), and completing a beads ticket
means checking the box in the openspec `tasks.md`. Wai's `wai doctor --suite`
will eventually validate suite-wide consistency (see `wai-bdqw.8`).

Detection is additive and optional — each tool works independently, but when
multiple are present wai becomes the unified dashboard.

## Worked Example: Adding a Feature

Here's how a ticket flows through the core trio (wai + beads + openspec):

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

With the suite tools integrated, the workflow extends further:

```bash
# Enforce code quality conventions
dont define "No unwrap() in library code"

# Verify spec-to-test coverage before merging
ah check       # espectacular — does the test match the spec?
testaruda check  # property-based test coverage

# Lint for structural quality
pretender check .
```

## What You Lose by Skipping One

- **Without wai**: Tasks get done, but nobody remembers *why* decisions were
  made. Six months later, the code is a black box.
- **Without beads**: Reasoning is captured, but there's no task decomposition,
  no dependency tracking, and no way to find available work.
- **Without openspec**: Changes happen ad hoc — no requirements, no acceptance
  criteria, no validation that the system matches the spec.
- **Without pretender**: Code quality standards are enforced by human review
  alone — slower, more inconsistent, and harder to enforce in CI.
- **Without dont**: Conventions live in READMEs or Slack — unenforceable and
  easy to forget.
- **Without espectacular**: Specs and tests drift independently; no automated
  signal that they disagree.
- **Without testaruda**: Property-based and harness testing must be set up
  manually, with no unified test dashboard.
- **Without vampiro/crua/livin** (planned): Spec-stage analysis must be done
  manually or deferred until implementation.

Each tool is optional. Wai works fine alone. But together, the full suite
covers the entire lifecycle — from proposal through implementation, quality
assurance, and archival.

> **See also**: [Suite Conventions](./suite-conventions.md) for canonical
> policies on edition, license, version scheme, and justfile setup.