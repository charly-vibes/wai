# Change: Cross-project skill sharing via global library and export/import

## Why

Skills created for one project are generic but stored inside that project's `.wai/`.
Reusing the same pipeline skills in another project requires manually copying files,
recreating directory structures, and re-writing CLAUDE.md. There is no mechanism for
sharing reusable skills across projects or repositories.

## What Changes

- Global skill library at `~/.wai/resources/skills/`; project-local takes priority
- `wai resource install <skill> --global` installs a skill globally from current project
- `wai resource install <skill> --from-repo <path>` installs from another local repo
- `wai resource export <skill>... --output <file.tar.gz>` bundles skills for sharing
- `wai resource import <file.tar.gz>` installs bundled skills into current project
- Skills MUST NOT contain hardcoded project names; use `$PROJECT`, `$REPO_ROOT` placeholders

## Impact

- Affected specs: new capability `skill-sharing`
- Affected code: `src/commands/resource.rs`, `src/cli.rs`, `src/config.rs`
- Depends on: `add-hierarchical-skills`
