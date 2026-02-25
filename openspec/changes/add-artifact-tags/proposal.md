# Change: Add tags to plan and design artifacts, and tag-based search filtering (P8 + P5-A)

## Why

The `--tags` flag works only on `wai add research`, making it impossible to tag
plans and designs. Without tags on all artifact types, multi-stage agent pipelines
cannot reliably retrieve the correct artifact from a previous stage — a topic word
like "performance" may match dozens of unrelated artifacts via full-text search.

## What Changes

- Add `--tags` flag to `wai add plan` and `wai add design` (consistent with research)
- Tags are written as YAML frontmatter, same format as research
- Extend `wai search` with `--tag <value>` filter (frontmatter-based, repeatable)
- Add `--latest` flag to `wai search` to return only the most recently dated match
- `--type` already exists on search; ensure it covers plan/design as well as research

## Impact

- Affected specs: research-management, timeline-search (search requirement)
- Affected code: `src/cli.rs`, `src/commands/add.rs`, `src/commands/search.rs`
- No breaking changes; all flags are additive

## Scope Note

This change bundles two related concerns: P8 (tags on all artifact types) and P5-A
(tag-based search filtering). P5-A depends on P8 — tags on plan/design are only useful
if search can filter by them. Implementing them together avoids a half-usable intermediate state.
