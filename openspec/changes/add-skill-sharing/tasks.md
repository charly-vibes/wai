## Prerequisites

This change requires `add-hierarchical-skills` to be implemented first.

## 1. Global skill library

- [ ] 1.1 Define global skills path as `~/.wai/resources/skills/` in `src/config.rs`
- [ ] 1.2 Update `wai resource list skills` to show both local and global skills,
        marking the source of each
- [ ] 1.3 Update skill resolution to prefer local over global when names conflict
- [ ] 1.4 Add unit tests for priority resolution (local overrides global)

## 2. Install commands

- [ ] 2.1 Add `wai resource install <skill> --global` — copies skill from current
        project's `.wai/` into `~/.wai/resources/skills/`
- [ ] 2.2 Add `wai resource install <skill> --from-repo <path>` — copies skill from
        the specified repository's `.wai/resources/agent-config/skills/`
- [ ] 2.3 Validate that installed skill files use only `$ARGUMENTS`, `$PROJECT`,
        `$REPO_ROOT` placeholders; warn (don't block) on hardcoded project names
- [ ] 2.4 Add tests for `--global` install and `--from-repo` install

## 3. Export and import

- [ ] 3.1 Implement `wai resource export <skill>... --output <file.tar.gz>` — bundles
        specified `SKILL.md` files into a tar.gz archive preserving the skill
        subdirectory structure
- [ ] 3.2 Implement `wai resource import <file.tar.gz>` — extracts skill files into
        the current project's skills directory; prompt before overwriting; support
        `--yes` flag for non-interactive use
- [ ] 3.3 Validate archive structure on import: reject entries whose path does not
        match `<name>/SKILL.md` or `<category>/<name>/SKILL.md` (path traversal prevention)
- [ ] 3.4 Add tests for round-trip export → import preserving skill content
- [ ] 3.5 Add test for `--yes` non-interactive overwrite
- [ ] 3.6 Add test that a malformed archive (path traversal attempt) is rejected with error

## 4. Documentation

- [ ] 4.1 Update `--help` strings for `wai resource install`, `wai resource export`,
        and `wai resource import`
