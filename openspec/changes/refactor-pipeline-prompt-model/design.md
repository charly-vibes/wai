# Design: Prompt-Driven Pipeline Steps

## Context

The pipeline system was built to sequence agent skills through ordered stages.
The initial model tied each stage to a `skill` reference and `artifact` type,
which was good for tracking but left the agent without instructions. Every
session boundary required a human to re-explain what step to do next.

The new model makes the step prompt a navigation hint: a one-line task summary
plus wai commands (record artifact, advance). HOW to do the work stays in skill
files. The schema becomes minimal (id + prompt), wai's job is to surface the
right prompt at the right time — especially after context loss — and to track
position across sessions so the agent never has to re-orient manually.

## Goals / Non-Goals

**Goals:**
- Agent can run a full pipeline without human input between steps
- After `/clear` or context compaction, agent can re-orient with one command
- Pipeline definitions are human-readable and easy to create/edit
- Simple enough that `wai reflect` can generate them from artifact sequences

**Non-Goals:**
- wai as an executor (wai tracks state; the agent drives execution)
- Branching or conditional transitions (loop logic lives in the prompt text)
- Review/fix graph model (a prompt like "if issues found, repeat" is sufficient)
- Reflect-based pipeline discovery (future feature, out of scope here)

## Decisions

### Schema decisions

**Decision: Steps are (id, prompt) only — no skill/artifact fields**

Skill references were load-bearing when wai validated that each stage pointed
to a real skill file. In the prompt model, the prompt text mentions the skill
by name if needed. Removing the structural coupling eliminates an entire
validation path and makes pipelines portable across projects without requiring
matching skill installations.

Alternatives considered:
- Keep `skill` as optional field: adds complexity for no runtime benefit
- Add `type` field (action/review/fix): type is already implied by prompt content

**Decision: Step prompts are navigation hints, not skill-level instructions**

Skills (SKILL.md files) already define detailed HOW-to instructions for each
type of work. Pipeline step prompts must not duplicate that content. A step
prompt should contain: (1) a one-line task summary (what phase, optional skill
name hint), (2) the `wai add` command for capturing artifacts, and (3) the
`wai pipeline next` advancement command. Nothing more.

This keeps pipelines and skills non-overlapping. If a step's prompt is growing
toward a full how-to, that content belongs in a skill file instead.

✓ Correct step prompt:
```
Research {topic} — use skill `research-codebase` if installed.
Record: `wai add research "..."`
Next: `wai pipeline next`
```

✗ Wrong (instruction content that belongs in a skill, not a pipeline step):
```
Research {topic}. Find relevant files and understand existing patterns.
Look at tests to understand conventions. Read config files. Check the
existing implementation. Record your findings...
```

Alternatives considered:
- Allow rich prompts as tutorials: creates drift between skill and pipeline content
- Require skill field: reintroduces validation coupling we removed

**Decision: TOML format for definitions; YAML definitions are gone**

The project already uses TOML for `config.toml`. Consistency reduces friction.
TOML's `"""` multi-line strings are well-suited for prompt content. The old
YAML format is removed entirely — nobody is using the tool yet, so there is
no migration concern.

Alternatives considered:
- Keep YAML: inconsistent with project conventions
- Markdown with frontmatter: good for human reading but harder to parse

### State decisions

**Decision: `.last-run` pointer file for session recovery**

`WAI_PIPELINE_RUN` is set via `export` and lives in the shell session.
After `/clear` or a new terminal, it's gone. A pointer file at
`.wai/resources/pipelines/.last-run` containing just the run ID gives
`pipeline next`/`current` a fallback that survives session boundaries.
`pipeline start` writes it; `pipeline next` reads it when the env var
is absent.

If `.last-run` points to a run file that no longer exists (e.g., manually
deleted), commands that read it treat the pointer as absent and fall back
gracefully (no error, as if no run is active).

Alternatives considered:
- Require explicit `--run-id` argument: defeats the "no typing" goal
- Store in `.wai/config.toml`: config is project-level; last-run is ephemeral

**Decision: `{topic}` is the only v1 substitution variable**

A single variable covers the most common case (what are we working on?)
without introducing a template engine. The topic is the slug passed to
`pipeline start --topic=<slug>`. At render time, `{topic}` in the prompt
is replaced with the topic value. No other variables in v1.

### Behavioral decisions

**Decision: Loop logic lives in prompt text, not pipeline schema**

A review/fix loop is expressed as:
```
If issues found: fix them and repeat this step.
If clean: `wai pipeline next`
```
The agent handles the loop; wai's `current_step` index doesn't change until
the agent calls `next`. From wai's perspective the run is still on step N
during iteration. `wai pipeline status` shows "step 3/5" which is accurate
(the agent is still working on step 3). This is acceptable — wai is a guide,
not a finite state machine.

### Command decisions

**Decision: `pipeline init <name>` scaffolds a starter TOML template**

Dropping a TOML file manually is simple, but `pipeline init` lowers the
barrier to discovery. It writes a minimal two-step template to
`.wai/resources/pipelines/<name>.toml` and fails if the file already exists.
The directory is created if it doesn't exist.

**Decision: `pipeline suggest [description]` for conversation-start discovery**

Pipelines add value at the start of a conversation, not just during an active
run. The agent needs a way to discover which pipeline fits the current task
without manual inspection of TOML files. `pipeline suggest` lists all defined
pipelines with name, description, and step count. When an optional description
string is provided, results are ranked by keyword overlap (case-insensitive,
stop words not filtered in v1). Score is used for sorting only — it is not
printed. Ties are broken alphabetically by pipeline name. When all pipelines
score 0, output is alphabetical. An empty description string is treated as
absent (no scoring).

This enables the conversation-start workflow:
```
wai pipeline suggest "small regression in auth module"
→ quick-fix  (2 steps)  Quick bug diagnosis and fix
→ feature    (5 steps)  Full feature workflow: research → design → implement → review
Start: wai pipeline start quick-fix --topic=auth-regression
```

Alternatives considered:
- Leave discovery to `pipeline list`: list shows names only, no ranking
- LLM-based matching: out of scope for v1; keyword overlap is sufficient

**Decision: `wai status` shows available pipelines when no run is active**

The most valuable moment for pipeline selection is *before* starting — at the
beginning of a conversation. When no run is active and at least one pipeline
definition exists, `wai status` emits an "Available pipelines" section with
name and description. Malformed TOML files are skipped with a warning rather
than causing status to fail. This makes pipeline-assisted workflows a natural
first step rather than an afterthought.

When a run is active, status shows the active step. If `.last-run` points to
a missing run file, status treats it as no active run (stale pointer silently
ignored, falls back to idle state).

Alternatives considered:
- Only surface in `wai prime`: prime is session-recovery focused; status is broader
- Require explicit `wai pipeline list`: too much friction to discover organically

## Data Model Changes

| Artifact | Format | Notes |
|---|---|---|
| Pipeline definition | TOML (`.toml`) | Human-authored, drop in `.wai/resources/pipelines/` |
| Run state | YAML (`.yml`) | Machine-generated, not user-edited |
| `.last-run` pointer | Plain text | Single line: the active run ID |

**Pipeline definition** (TOML):
```toml
[pipeline]
name = "feature"
description = "Full feature workflow: research → design → implement → review"

[[steps]]
id = "research"
prompt = """
Research {topic} — use skill `research-codebase` if installed.
Record: `wai add research "..."`
Next: `wai pipeline next`
"""

[[steps]]
id = "design"
prompt = """
Design {topic} — use skill `design-practice` if installed.
Record: `wai add design "..."`
Next: `wai pipeline next`
"""

[[steps]]
id = "implement"
prompt = """
Implement {topic} — use skill `tdd` if installed.
Next: `wai pipeline next`
"""

[[steps]]
id = "close"
prompt = """
Close completed beads issues. Run `wai close` to capture handoff.
Next: `wai pipeline next`
"""
```

Step prompts follow the convention: one-line task summary (with optional skill
hint), `wai add` for artifacts, `wai pipeline next` to advance. No how-to
instructions — those live in the referenced skill files.

**Run state** (`current_stage: usize` renamed to `current_step: usize`):
- Integer index is sufficient; step ID is resolved at print time from the definition

**Last-run pointer**:
- Path: `.wai/resources/pipelines/.last-run`
- Content: single line — the run ID (e.g., `tdd-2026-02-27-auth-feature`)
- Written by: `pipeline start`
- Read by: `pipeline next`, `pipeline current` when `WAI_PIPELINE_RUN` unset

## Risks / Trade-offs

- **Loop detection absent**: An agent that never calls `pipeline next` stays
  on one step indefinitely. This is by design — the prompt governs termination.

- **`.last-run` is single-slot**: Only one "current" run is tracked. If a user
  starts two runs, the pointer points to the second. The first is still accessible
  via `WAI_PIPELINE_RUN` or explicit run ID in `pipeline status`.
