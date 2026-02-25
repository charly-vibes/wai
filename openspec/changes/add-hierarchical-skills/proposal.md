# Change: Add hierarchical skill names with category/action structure

## Why

Skill names are restricted to flat identifiers (`issue-gather`), but Claude Code
invokes skills via category-prefixed paths (`/issue:gather`). There is no mechanical
link between the wai name and the Claude Code invocation path, forcing users to
maintain two separate naming systems.

## What Changes

- Allow a single `/` separator in skill names: `issue/gather`, `impl/run`
- Storage: `.wai/resources/agent-config/skills/issue/gather/SKILL.md`
- `wai resource list skills` groups output by category when hierarchical skills exist
- Flat names (`my-skill`) continue to work unchanged
- Required as a prerequisite by `add-claude-code-projection`

## Impact

- Affected specs: agent-config-sync
- Affected code: `src/commands/resource.rs` (validation + listing)
- No breaking changes; flat names remain valid
