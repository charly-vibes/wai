# Multi-Project Resolution — Current State Analysis

## Problem
When multiple projects exist in .wai/projects/, several commands behave inconsistently:

- `wai phase *` commands accept NO --project flag. They use `find_active_project()` which silently picks the first alphabetical project directory.
- `wai pipeline *` commands are workspace-level with no project binding.
- Other commands (`add`, `close`, `prime`) do accept --project but fall back to interactive prompts that block agents.

## Inconsistency Matrix

| Command | Has --project? | Resolution |
|---------|---------------|------------|
| wai add | Yes | projects + areas auto-detect |
| wai close | Yes | projects only, interactive fallback |
| wai prime | Yes | projects only, interactive fallback |
| wai phase * | NO | first alphabetical, silently |
| wai pipeline * | NO | workspace-level, no project binding |

## Key Constraint: Parallel Work
Users may work on multiple projects simultaneously (different terminals, agents working in parallel). This rules out a single mutable `.current-project` pointer — it's a global singleton that creates race conditions.

## Existing Pattern
Pipelines already use `WAI_PIPELINE_RUN` env var for session-scoped binding. This pattern is parallel-safe and proven.

## Design Direction
Layered resolution (highest priority first):
1. `--project` flag (explicit, per-command)
2. `WAI_PROJECT` env var (session-scoped, parallel-safe)
3. Auto-detect if exactly 1 project
4. Error with guidance
