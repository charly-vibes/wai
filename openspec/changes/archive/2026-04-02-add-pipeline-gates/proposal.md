# Change: Add pipeline gate system for intermediate validation

## Why
AI-assisted research workflows need deterministic enforcement at step boundaries.
Currently pipelines are advisory — prompts tell the agent what to do but nothing
prevents advancement when work is incomplete, unreviewed, or invalid. The "vibe
physics" experiment demonstrated that unconstrained AI generation compounds errors
when intermediate results aren't validated before serving as context for subsequent
work.

## What Changes
- **Pipeline gates**: 4-tier validation protocol (structural, procedural, oracle,
  approval) evaluated when `wai pipeline next` is called
- **Step-level artifact tagging**: artifacts tagged with `pipeline-step:<step-id>`
  in addition to existing `pipeline-run:<run-id>`
- **Review artifact type**: new `wai add review` command producing artifacts with
  structured frontmatter (`reviews`, `verdict`, `severity`)
- **Oracle system**: user-defined verifier scripts in `.wai/resources/oracles/`
  with name-based resolution
- **Pipeline metadata**: `[pipeline.metadata]` TOML section for discoverability
  (`when`, `skills` fields)
- **New commands**: `wai pipeline gates`, `wai pipeline check`,
  `wai pipeline approve`, `wai pipeline show`, `wai pipeline validate`
- **Doctor integration**: pipeline validation in `wai doctor`, managed block
  staleness detection
- **Managed block update**: auto-generated "Available Pipelines" table from
  installed pipeline metadata

## Impact
- Affected specs: `pipeline-resource`, `managed-block`
- Affected code: `src/commands/pipeline.rs`, `src/commands/add.rs`,
  `src/managed_block.rs`, `src/workspace.rs`
- New directories: `.wai/resources/oracles/`
- New assets: oracle README template, example oracle script, built-in
  `scientific-research` pipeline template (embedded in binary)
- Non-breaking: existing pipelines without gates continue to work as before
