## Context

wai currently has no unified project resolution. Each command implements its own
logic: `add.rs` has `resolve_project()` (searches projects + areas), `mod.rs` has
`resolve_project_named()` (projects only), and `phase.rs` has
`find_active_project()` (first alphabetical, no flags). This divergence causes
silent wrong-project bugs when multiple projects exist.

Users doing parallel work — running agents in separate terminals, each on a
different project — need session-scoped isolation. A global `.current-project`
file is a race condition; per-command flags are verbose and easy to forget.

## Goals / Non-Goals

- **Goals:**
  - Single resolution algorithm shared by all project-scoped commands
  - Session-scoped project binding via env var (parallel-safe)
  - `--project` flag on all project-scoped commands (explicit override)
  - Backward-compatible: single-project workspaces behave identically

- **Non-Goals:**
  - Workspace-level persistent "default project" (race-prone)
  - Auto-detecting project from current working directory
  - Multi-project operations in a single command invocation
  - Project name validation rules (existing concern, separate ticket)

## Project-Scoped Commands

The following commands are project-scoped and participate in unified resolution:
`phase` (all subcommands), `add`, `close`, `prime`, `reflect`.

Commands that take the project as a positional arg (`handoff create <project>`,
`timeline <project>`) already require explicit naming and do not need env var
fallback. `search --in <project>` is similar. These commands SHOULD validate
against `WAI_PROJECT` if set (warn if positional arg differs from env var) but
this is a follow-up concern, not in scope for this change.

## Decisions

### Resolution order (highest priority wins)

1. `--project <name>` CLI flag
2. `WAI_PROJECT` environment variable (non-empty)
3. Auto-detect: if exactly 1 project in `.wai/projects/`, use it
4. Interactive selector (if TTY and `--no-input` not set)
5. Error with available project list and hint to set `WAI_PROJECT`

**Why this order:** Explicit flag beats session context beats implicit detection
beats interactive prompt beats error. This mirrors how `WAI_PIPELINE_RUN` already
works for pipelines — proven pattern. Interactive selection is preserved for
terminal users but never blocks non-interactive contexts.

### Single `resolve_project()` function

Replace the three separate resolution functions (`resolve_project` in add.rs,
`resolve_project_named` in mod.rs, `find_active_project` in phase.rs) with one
canonical `resolve_project()` in `mod.rs` that all commands call.

**Why:** Eliminates divergence. The current `add.rs` searches areas too — this
changes to projects-only for auto-detect (see below).

### Auto-detect counts `.wai/projects/` only

The unified resolution counts only `.wai/projects/` directories for
auto-detection, not areas or resources. This is a minor behavioral change for
`wai add` which currently counts projects + areas.

**Why:** Areas are ongoing responsibilities without phases — including them in
project resolution conflates two PARA concepts. If a user wants to add artifacts
to an area, they can use `--project <area-name>` explicitly (the flag validates
against all PARA categories, not just projects).

### `WAI_PROJECT` over `.current-project` file

Env var is naturally scoped to a shell session. Two terminals can each `export
WAI_PROJECT=different-project` without interference. A dotfile would require
locking or last-write-wins semantics.

**Why:** Parallel work is a first-class use case, not an edge case.

### Empty `WAI_PROJECT` treated as unset

An empty string (`export WAI_PROJECT=` or `WAI_PROJECT=""`) is treated as if
the variable is not set. This prevents confusing "project '' not found" errors
when users attempt to clear the variable.

### `wai project use <name>` convenience command

Prints the shell-appropriate export statement (and validates the project exists).
Detects the user's shell from `$SHELL` env var:
- bash/zsh: `export WAI_PROJECT=<name>`
- fish: `set -gx WAI_PROJECT <name>`

When stdout is a terminal, also prints a usage hint to stderr so the user knows
to eval or paste the output.

**Why:** Reduces friction for setting the env var. Also serves as a discoverability
mechanism — `wai project use` without args lists available projects.

### `--project` on Phase command, not subcommands

The `--project` flag is added to the `Phase` variant in the parent `Commands`
enum (alongside `Close`, `Prime`, etc.), not to each `PhaseCommands` variant
(`Next`, `Set`, `Back`, `Show`). This matches the pattern used by other commands.

**Why:** Avoids repeating the flag definition on every subcommand. Clap propagates
parent flags to subcommands automatically.

### Keep interactive selection for TTY users

When multiple projects exist and no flag/env var is set, the system presents an
interactive selector if stdin is a terminal (and `--no-input` is not set). Only
in non-interactive contexts (piped stdin, CI, agents) does it error.

**Why:** Removing interactive selection entirely would regress the experience for
human users who have 2 projects and just want to pick one quickly. The env var
and flag are the primary mechanisms; the interactive prompt is a safety net.

## Risks / Trade-offs

- **Risk:** Users forget they have `WAI_PROJECT` set and operate on the wrong
  project in a new context.
  → **Mitigation:** `wai status` and `wai phase show` display the resolved project
  name and how it was resolved (flag/env/auto). `wai prime` shows `[via WAI_PROJECT]`
  when env var is active.

- **Risk:** Env var doesn't persist across shell restarts.
  → **Acceptable:** Session-scoped is the feature, not a bug. Users add it to
  `.envrc` or shell profile if they want persistence.

- **Risk:** `wai add` auto-detect scope change (projects-only instead of
  projects+areas) could surprise users who have areas but no projects.
  → **Mitigation:** Error message lists available items across all PARA categories
  and suggests `--project <name>`.
