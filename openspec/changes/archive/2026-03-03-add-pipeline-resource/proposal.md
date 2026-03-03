# Change: Add pipeline resource type for ordered multi-skill workflows

## Why

The 3-stage agent pattern (gather → create/implement → review) must be documented
as unstructured prose in CLAUDE.md. There is no machine-readable way to define
stage ordering, expected artifact types per stage, or the status of a pipeline run.
This makes pipelines non-transferable across projects and provides no runtime guidance.

## What Changes

- New `pipeline` resource type with ordered stages, each declaring a skill and output artifact type
- `wai pipeline create <name> --stages="skill1:artifact-type,skill2:artifact-type,..."`
- `wai pipeline run <name> --topic="<slug>"` creates a run ID and tags stage artifacts
- `wai pipeline status <name>` shows per-stage completion with artifact paths
- Depends on `add-hierarchical-skills` for skill name format and `add-artifact-tags` for run tagging

## Impact

- Affected specs: new capability `pipeline-resource`
- Affected code: `src/cli.rs`, new `src/commands/pipeline.rs`, `src/config.rs`
- Depends on: `add-hierarchical-skills`, `add-artifact-tags`
