# Change: Add built-in skill templates for common agent pipeline patterns

## Why

Creating skills from scratch requires knowing the `$ARGUMENTS` convention, wai
artifact patterns, and convergence check structure. Writing 6 pipeline skills from
a bare stub takes significant effort. Built-in templates reduce setup friction for
new projects adopting the gather → create/implement → review pattern.

## What Changes

- Add `--template <name>` flag to `wai resource add skill`
- Built-in templates: `gather`, `create`, `tdd`, `rule-of-5`
- `gather`: research stub with `wai search`, codebase exploration, `wai add research`
- `create`: creation stub with artifact retrieval, item loop, dependency wiring
- `tdd`: test-first implementation stub with RED/GREEN/REFACTOR steps
- `rule-of-5`: 5-pass review stub with convergence check and verdict output
- Templates use `$ARGUMENTS` and `$PROJECT` placeholders (no hardcoded project names)

## Impact

- Affected specs: agent-config-sync
- Affected code: `src/commands/resource.rs`, `src/cli.rs`
- Templates are embedded in the binary; no external files required
- Depends on: `add-artifact-tags` (the `create` template uses `wai search --latest`,
  which is introduced by that change; templates will reference a non-existent flag
  if `add-artifact-tags` has not landed first)
