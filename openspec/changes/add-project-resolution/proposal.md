# Change: Add unified project resolution strategy

## Why

When multiple projects exist in `.wai/projects/`, the CLI behaves inconsistently.
`wai phase` commands silently operate on the first alphabetical project — no
`--project` flag, no env var fallback. Other commands (`add`, `close`, `prime`)
accept `--project` but each implements its own resolution logic. Users working on
multiple projects in parallel (different terminals, concurrent agents) have no
session-scoped way to bind a shell to a specific project.

## What Changes

- **All project-scoped commands** (`phase`, `add`, `close`, `prime`, `reflect`)
  gain a consistent resolution algorithm:
  `--project` flag → `WAI_PROJECT` env var → auto-detect (if exactly 1) →
  interactive selector (if TTY) → error with guidance
- **`wai phase`** subcommands (`show`, `next`, `back`, `set`) gain `--project`
  flag, bringing them in line with `close`/`prime`/`add`
- **`WAI_PROJECT` env var** provides session-scoped project binding, parallel-safe
  (same pattern as existing `WAI_PIPELINE_RUN`); empty string treated as unset
- **`wai project use <name>`** convenience command prints the shell-appropriate
  export line (detects bash/zsh/fish)
- **Auto-detect scope** unified to `.wai/projects/` only (minor change for `add`,
  which currently also searches areas)
- **Interactive selection preserved** for TTY users; non-interactive contexts
  (agents, CI) get deterministic error instead

Commands that take project as a positional arg (`handoff create`, `timeline`)
already require explicit naming and are not changed by this proposal.

## Impact

- Affected specs: `cli-core`, `project-state-machine`
- Affected code: `src/cli.rs`, `src/commands/phase.rs`, `src/commands/mod.rs`,
  `src/commands/add.rs`, `src/commands/close.rs`, `src/commands/prime.rs`,
  `src/commands/reflect.rs`
- **Not breaking**: all existing single-project workflows behave identically
  (auto-detect still works when exactly 1 project exists)
- **Minor behavioral change**: `wai add` auto-detect no longer counts areas,
  only projects. Users with areas can use `--project <area-name>` explicitly.
