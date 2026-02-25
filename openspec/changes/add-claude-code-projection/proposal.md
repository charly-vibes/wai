# Change: Add built-in Claude Code projection target

## Why

Every project using wai skills in Claude Code must manually maintain two copies of
each skill file — one in `.wai/resources/agent-config/skills/` and one in
`.claude/commands/` — with different frontmatter formats. There is no automated
way to project from the wai source of truth to the Claude Code format.

## What Changes

- Add `claude-code` as a built-in projection target name in `wai sync`
- When `target: claude-code`, the system scans `skills/<category>/<action>/SKILL.md`
  and generates `.claude/commands/<category>/<action>.md` with translated frontmatter
- Frontmatter translation: wai `name`/`description` → Claude Code `name`/`description`/`category`
- Creates all parent directories automatically
- Requires hierarchical skill names (`add-hierarchical-skills`) to be implemented first

## Impact

- Affected specs: agent-config-sync
- Affected code: `src/commands/sync.rs`, `src/sync_core.rs`
- Depends on: `add-hierarchical-skills`
