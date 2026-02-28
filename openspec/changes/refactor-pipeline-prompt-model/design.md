# Design: Prompt-Driven Pipeline Steps

## Context

The pipeline system was built to sequence agent skills through ordered stages.
The initial model tied each stage to a `skill` reference and `artifact` type,
which was good for tracking but left the agent without instructions. Every
session boundary required a human to re-explain what step to do next.

The new model inverts this: the prompt IS the instruction. The schema becomes
minimal (id + prompt), loop logic lives inside prompt text, and wai's job is
to surface the right prompt at the right time — especially after context loss.

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

**Decision: Steps are (id, prompt) only — no skill/artifact fields**

Skill references were load-bearing when wai validated that each stage pointed
to a real skill file. In the prompt model, the prompt text mentions the skill
by name if needed. Removing the structural coupling eliminates an entire
validation path and makes pipelines portable across projects without requiring
matching skill installations.

Alternatives considered:
- Keep `skill` as optional field: adds complexity for no runtime benefit
- Add `type` field (action/review/fix): type is already implied by prompt content

**Decision: TOML format for definitions; YAML definitions are gone**

The project already uses TOML for `config.toml`. Consistency reduces friction.
TOML's `"""` multi-line strings are well-suited for prompt content. The old
YAML format is removed entirely — nobody is using the tool yet, so there is
no migration concern.

Alternatives considered:
- Keep YAML: inconsistent with project conventions
- Markdown with frontmatter: good for human reading but harder to parse

**Decision: `.last-run` pointer file for session recovery**

`WAI_PIPELINE_RUN` is set via `export` and lives in the shell session.
After `/clear` or a new terminal, it's gone. A pointer file at
`.wai/resources/pipelines/.last-run` containing just the run ID gives
`pipeline next`/`current` a fallback that survives session boundaries.
`pipeline start` writes it; `pipeline next` reads it when the env var
is absent.

Alternatives considered:
- Require explicit `--run-id` argument: defeats the "no typing" goal
- Store in `.wai/config.toml`: config is project-level; last-run is ephemeral

**Decision: `{topic}` is the only v1 substitution variable**

A single variable covers the most common case (what are we working on?)
without introducing a template engine. The topic is the slug passed to
`pipeline start --topic=<slug>`. At render time, `{topic}` in the prompt
is replaced with the topic value. No other variables in v1.

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

**Decision: `pipeline init <name>` scaffolds a starter TOML template**

Dropping a TOML file manually is simple, but `pipeline init` lowers the
barrier to discovery. It writes a minimal two-step template to
`.wai/resources/pipelines/<name>.toml` and fails if the file already exists.
The directory is created if it doesn't exist.

## Data Model Changes

| Artifact | Format | Notes |
|---|---|---|
| Pipeline definition | TOML (`.toml`) | Human-authored, drop in `.wai/resources/pipelines/` |
| Run state | YAML (`.yml`) | Machine-generated, not user-edited |
| `.last-run` pointer | Plain text | Single line: the active run ID |

**Pipeline definition** (TOML):
```toml
[pipeline]
name = "tdd"
description = "Test-driven feature implementation"

[[steps]]
id = "research"
prompt = """
Research {topic}. Find relevant files and understand existing patterns.
Record findings: `wai add research "..."`
When done: `wai pipeline next`
"""

[[steps]]
id = "close"
prompt = """
Close the beads issue and run `wai close` to capture a handoff.
"""
```

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
