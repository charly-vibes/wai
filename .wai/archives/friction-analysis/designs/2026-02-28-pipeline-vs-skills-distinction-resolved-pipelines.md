---
date: 2026-02-28
project: friction-analysis
phase: design
---

# Design: Pipeline vs Skills Distinction

## Problem Statement

The `wai pipeline` and `wai resource add skill` commands were being conflated. Both
involve structured, multi-step agent interaction — but for fundamentally different
purposes. Agents and users were unclear on when to use which, and pipeline step
prompts were being written as full how-to guides (duplicating skill content).

## Options Considered

**Option A: Merge pipelines into skills**
Treat pipelines as a special skill type with sequential steps. Simpler surface area
but loses the state-tracking and cross-session resume capability.

**Option B: Keep separate with clear semantic boundary** *(chosen)*
Pipelines = navigation + cross-session state tracking (the *where* and *when*).
Skills = HOW-to instructions (the *how*). Step prompts stay thin; they reference
skills rather than duplicating them.

**Option C: Replace pipelines with a simple checklist format**
Lower overhead, but loses the `wai pipeline advance` state machine and the
conversation-start discovery path.

## Decision

Pipelines and skills are distinct and complementary:

| Concept | Role | Format |
|---------|------|--------|
| Pipeline | Navigates a multi-step workflow; tracks active step across sessions | TOML with thin step prompts |
| Skill | Provides reusable HOW-to instructions for a specific action | SKILL.md with full context |

**Step prompts must be thin** — one-line task description + wai commands only.
Any detailed how-to belongs in a referenced skill, not the step prompt itself.

## Consequences

- `wai pipeline suggest` enables conversation-start discovery via keyword ranking.
- `wai status` shows available pipelines when idle, active step when running.
- Step prompt verbosity is a code smell — flag it in review.
- Skills remain self-contained and reusable across pipelines.
- `refactor-pipeline-prompt-model` expanded from 35 to 44 tasks (10 sections)
  to implement this model fully.

## Status

Implemented via `refactor-pipeline-prompt-model` (50/50 tasks, completed 2026-03).
