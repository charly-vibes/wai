# Change: Replace pipeline stage model with prompt-driven steps

## Why

The existing pipeline model (`skill:artifact` stages) tracks _what type of artifact
each stage produces_ but doesn't tell the agent _what to do_. Between every step the
human must type instructions. This makes pipelines a bookkeeping tool, not a
workflow guide. Prompt-driven steps make each step self-describing: the agent reads
the prompt and knows exactly what to do next, enabling autonomous loops without
human intervention between steps.

## Design Principle: Pipelines Are Navigation, Not Instructions

Step prompts are thin coordination hints — one-line summary plus wai navigation
commands. Instructions for *how* to do the work live in skills. A pipeline step
should answer "what phase am I on and what command signals completion?", not
"here is a tutorial on how to research".

```
✗ Wrong: Research {topic}. Find relevant files and understand existing patterns.
         Look at tests to understand conventions. Read config files...

✓ Right: Research {topic} — use skill `research-codebase` if installed.
         Record: `wai add research "..."` | Next: `wai pipeline next`
```

This keeps pipelines and skills non-overlapping: skills provide the HOW,
pipelines provide sequencing + cross-session state tracking.

## What Changes

- **BREAKING**: Pipeline definition format changes from YAML (`<name>.yml`) to TOML
  (`<name>.toml`). Steps replace stages: each step has an `id` and a `prompt`
  string instead of `skill` and `artifact` fields.
- **BREAKING**: `pipeline create --stages="..."` is removed. Pipelines are defined
  by dropping a TOML file into `.wai/resources/pipelines/`. A `pipeline init <name>`
  command scaffolds a starter template.
- **BREAKING**: `pipeline run <name>` renamed to `pipeline start <name>`.
- **BREAKING**: `pipeline advance <run-id>` renamed to `pipeline next`. Run ID is
  resolved from `WAI_PIPELINE_RUN` env var or the `.last-run` pointer file — no
  argument required.
- **ADDED**: `pipeline current` command re-prints the active step's prompt. Core
  session-recovery mechanism after `/clear` or context compaction.
- **ADDED**: `pipeline suggest [description]` command for pipeline discovery. Lists
  all defined pipelines with name, description, and step count. If an optional
  description is provided, ranks results by keyword overlap so the agent can
  recommend the right pipeline for the current task.
- **ADDED**: `{topic}` variable substitution in prompt strings, resolved at render
  time from the `--topic` value passed to `pipeline start`.
- **ADDED**: `.wai/resources/pipelines/.last-run` pointer file, written by
  `pipeline start`, read by `pipeline next`/`current` when env var is absent.
- **ADDED**: Last-step completion signal — `pipeline next` on the final step prints
  a completion block and suggests `wai close`.
- **ADDED**: Load-time validation — non-empty prompts and unique step IDs checked
  when the pipeline TOML is loaded (at `start` time).
- **UPDATED**: `wai status` shows available pipelines (name + description) when no
  run is active, prompting the agent to consider using one at conversation start.
  When a run is active, shows the active pipeline step as before.

## Impact

- Affected specs: `pipeline-resource` (modified), `context-suggestions` (modified — suggestions block gains `wai pipeline suggest` / `wai pipeline current` entries; see task 11.1)
- Affected code: `src/commands/pipeline.rs`, `src/cli.rs`, `src/config.rs`,
  `src/commands/status.rs`
- `.yml` pipeline definitions are not supported; write new TOML definitions
- Depends on: `add-artifact-tags` (WAI_PIPELINE_RUN tagging, unchanged)
