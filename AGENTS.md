# This Repo

This is the **wai source code repository** — the Rust CLI (`src/`) is the wai tool itself.

The repo also **dogfoods wai**: `.wai/` tracks wai's own development using wai. This means `.wai/projects/` holds active feature work, beads issues track wai's own tasks, and openspec manages its change proposals.

## Two Kinds of Work

When creating or evaluating tickets, distinguish:

| Type | Description | Touches |
|------|-------------|---------|
| **Tool work** | Adding or changing wai functionality | `src/`, `tests/`, `Cargo.toml`, openspec |
| **Repo maintenance** | Workflows, scripts, docs, wai artifacts | `.wai/`, `CLAUDE.md`, `scripts/`, `.github/` |

Tool tickets require Rust implementation and typically need an openspec change first (`openspec/AGENTS.md`). Maintenance tickets do not touch `src/`.

<!-- WAI:START --># Workflow Tools

## PRIMARY OBJECTIVE

**wai** will enable **AI-assisted software developers** to **recover** **the full context of a project not touched in 30+ days** by **reducing context-recovery time from >15 minutes to <2 minutes** within **6 months (by February 2027)**, as measured by **time from `wai prime` to correctly identifying the project's phase, purpose, and most recent decision**, compared to **reading git log, source comments, and README without wai**.

Support and evolve **wai** — the workflow manager for AI-driven development —
by shipping correct, well-tested, well-governed changes to the Rust CLI.
Every action should trace back to: does this make wai more reliable,
more capable, or better documented for its users?

**wai** will enable **AI-assisted software developers** to **recover** **the full context of a project not touched in 30+ days** by **reducing context-recovery time from >15 minutes to <2 minutes** within **6 months (by February 2027)**, as measured by **time from `wai prime` to correctly identifying the project's phase, purpose, and most recent decision**, compared to **reading git log, source comments, and README without wai**.

This project uses **wai** to track the *why* behind decisions — research,
reasoning, and design choices that shaped the code. Run `wai status` first
to orient yourself.

Detected workflow tools:
- **wai** — research, reasoning, and design decisions
- **beads** — issue tracking (tasks, bugs, dependencies). CLI command: **`bd`** (not `beads`)
- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)

> **CRITICAL**: Apply TDD and Tidy First throughout — not just when writing code:
> - **Planning/task creation**: each ticket should map to a red→green→refactor cycle; refactoring tasks must be separate tickets from feature tasks.
> - **Design**: define the test shape (inputs/outputs) before designing the implementation.
> - **Implementation**: write the failing test first, then make it pass, then tidy in a separate commit.

> **When beginning research or creating a ticket**: run `wai search "<topic>"` to check for existing patterns before writing new content.
> **Ro5**: The Rule of 5 skill is installed. Run `/ro5` after key phase transitions — implement, research, design — for iterative quality review.

## Quick Start

1. `wai sync` — ensure agent tools are projected
2. `wai status` — see active projects, phase, and suggestions
3. `bd ready` — find available work items

When context reaches ~40%: stop and tell the user — responses degrade past
this point. Recommend `wai close` then `/clear` to resume cleanly.
Do NOT skip `wai close` — it enables resume detection.

## Available Pipelines

| Pipeline | When to Use | Start |
|----------|-------------|-------|
| epic-autonomy-tdd-ro5 | Use when autonomously executing one ready child issue from epic wai-fvhv without routine confirmation | `wai pipeline start epic-autonomy-tdd-ro5 --topic=<topic>` |
| scientific-research | Frontier-level research requiring systematic validation | `wai pipeline start scientific-research --topic=<topic>` |

> Pipeline steps may have gates that enforce artifact creation, review coverage, and oracle checks before advancement. Run `wai pipeline gates <name>` for details.

## Ubiquitous Language

If `.wai/resources/ubiquitous-language/README.md` exists, read it first as the
navigation index, then open only the bounded-context files relevant to the task.
Avoid loading every terminology file unless the work truly spans multiple contexts.



## Detailed Instructions

Full workflow reference — session lifecycle, capturing work, command cheat
sheets, cross-tool sync, and PARA structure — lives in **`.wai/AGENTS.md`**.
Read it at the start of your first session or when you need detailed guidance.

## PRIMARY OBJECTIVE (echo)

**wai** will enable **AI-assisted software developers** to **recover** **the full context of a project not touched in 30+ days** by **reducing context-recovery time from >15 minutes to <2 minutes** within **6 months (by February 2027)**, as measured by **time from `wai prime` to correctly identifying the project's phase, purpose, and most recent decision**, compared to **reading git log, source comments, and README without wai**.

Support and evolve **wai** — the workflow manager for AI-driven development —
by shipping correct, well-tested, well-governed changes to the Rust CLI.
Every action should trace back to: does this make wai more reliable,
more capable, or better documented for its users?

**wai** will enable **AI-assisted software developers** to **recover** **the full context of a project not touched in 30+ days** by **reducing context-recovery time from >15 minutes to <2 minutes** within **6 months (by February 2027)**, as measured by **time from `wai prime` to correctly identifying the project's phase, purpose, and most recent decision**, compared to **reading git log, source comments, and README without wai**.

Keep this managed block so `wai init` can refresh the instructions.

<!-- WAI:END -->

## Value Proposition

**wai** will enable **AI-assisted software developers** to **recover** **the full context of a project not touched in 30+ days** by **reducing context-recovery time from >15 minutes to <2 minutes** within **6 months (by February 2027)**, as measured by **time from `wai prime` to correctly identifying the project's phase, purpose, and most recent decision**, compared to **reading git log, source comments, and README without wai**.

**Kill criteria:** Context recovery >2 min for 30+ day old project by Feb 2027.
**Owner:** charly vibes

## Behavioral Constraints

These constraints are **persistent** — they live outside the WAI managed
block so they survive `wai init`. Do not remove or edit them without
deliberate intent.

### Prohibited (DON'T)

- **DON'T** make breaking changes without an openspec proposal and approval
- **DON'T** push directly to main — all changes go through feature branches with PR review
- **DON'T** modify `<!-- WAI: -->` / `<!-- OPENSPEC: -->` / `<!-- BEADS: -->` managed blocks — they are overwritten by tool commands
- **DON'T** skip tests, clippy, or fmt — CI gates are mandatory
- **DON'T** refactor code without test coverage for the refactored paths
- **DON'T** commit generated artifacts (`target/`, vendored deps)

### Stop and Ask

Pause and request human input when any of these triggers fire:
1. **Ambiguity** — the ticket text itself is contradictory or underspecified
2. **Scope uncertainty** — the ticket is clear but the change naturally touches code or features not mentioned in it
3. **Irreversibility** — data loss, force-push, schema migration, or destructive action
4. **Secrets/credentials** — any external service, API key, or credential not yet authorized
5. **Test failure persistence** — unresolved test failure after two repair attempts, or the same failure across 3 different approaches
6. **Push/release** — pushing to remote, creating a release, or deploying
7. **Context saturation** — context approaching ~40%; recommend `wai close` then `/clear`

### Minimal Footprint

- Prefer small, focused changes over large refactors — one ticket, one concern
- Delete unused code, don't leave commented-out code behind
- Keep PRs under 400 lines changed. If you cannot, split the work into multiple PRs before proceeding.
- Use existing abstractions (genesis, wai patterns) before introducing new ones
- Do not add dependencies unless the cost is justified by the benefit

### Drift Detection

Proceed without routine confirmation when the next step is clear.
Do not ask to continue, fix, or commit — just do it. After each major
action (edit, test run, commit), pause and self-check:
1. **ALIGNMENT** — does this still serve the PRIMARY OBJECTIVE?
2. **SCOPE** — did I stay within the ticket scope or did I expand into unticketed work?
3. **FOOTPRINT** — did I leave dead code, debug prints, or unnecessary changes?
4. **GOVERNANCE** — did I follow openspec workflow for spec changes?

If any check fails: undo the last change (`git checkout -- <files>` for
uncommitted edits, `git revert HEAD` for committed) before proceeding,
or open a follow-up ticket.

<!-- WAI:REFLECT:REF:START -->
## Accumulated Project Patterns

Project-specific conventions, gotchas, and architecture notes live in
`.wai/resources/reflections/`. Run `wai search "<topic>"` to retrieve relevant
context before starting research or creating tickets.

> **Before research or ticket creation**: always run `wai search "<topic>"` to
> check for known patterns. Do not rediscover what is already documented.

<!-- WAI:REFLECT:REF:END -->

















<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

<!-- BEADS:START -->
# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

Keep this managed block so `bd onboard` can refresh the instructions.
<!-- BEADS:END -->

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->

## Git & Workflow Discipline

- **Never use `git add -A`** — always stage specific files with explicit paths
- **Per-ticket pipeline**: always follow `TDD → ro5u → fix → commit → next ticket`

<!-- ah:managed:start -->
## espectacular

Run `ah check` to verify spec-test correspondence before committing.

- `ah check` — validate all deployed specs
- `ah check --changes <name>` — validate with a change overlay
- `ah init` — set up or refresh espectacular project files
- `ah doctor` — diagnose setup issues
- `ah explain <topic>` — playbook guidance for finding kinds and suggested actions
- `ah doctor --enable <adapter>` — write adapter config into .espectacular/config.toml
- `ah signals` — emit dont drift signals
<!-- ah:managed:end -->
