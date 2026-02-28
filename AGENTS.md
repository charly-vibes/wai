# This Repo

This is the **wai source code repository** — the Rust CLI (`src/`) is the wai tool itself.

The repo also **dogfoods wai**: `.wai/` tracks wai's own development using wai. This means `.wai/projects/` holds active feature work, beads issues track wai's own tasks, and openspec manages its change proposals.

## Two Kinds of Work

When creating or evaluating tickets, distinguish:

| Type | Description | Touches |
|------|-------------|---------|
| **Tool work** | Adding or changing wai functionality | `src/`, `tests/`, `Cargo.toml`, openspec |
| **Repo maintenance** | Workflows, scripts, docs, wai artifacts | `.wai/`, `CLAUDE.md`, `scripts/`, `.github/` |

Tool tickets require Rust implementation and typically need an openspec change first (`openspec/AGENTS.md`). Maintenance tickets do not touch `src/`.

<!-- WAI:START -->
# Workflow Tools

This project uses **wai** to track the *why* behind decisions — research,
reasoning, and design choices that shaped the code. Run `wai status` first
to orient yourself.

Detected workflow tools:
- **wai** — research, reasoning, and design decisions
- **beads (bd)** — issue tracking (tasks, bugs, dependencies)
- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)

## When to Use What

| Need | Tool | Example |
|------|------|---------|
| Record reasoning/research | wai | `wai add research "findings"` |
| Capture design decisions | wai | `wai add design "architecture choice"` |
| Session context transfer | wai | `wai handoff create <project>` |
| Track work items/bugs | beads | `bd create --title="..." --type=task` |
| Find available work | beads | `bd ready` |
| Manage dependencies | beads | `bd dep add <blocked> <blocker>` |
| Propose system changes | openspec | Read `openspec/AGENTS.md` |
| Define requirements | openspec | `openspec validate --strict` |

Key distinction:
- **wai** = *why* decisions were made (reasoning, context, handoffs)
- **beads** = *what* needs to be done (concrete tasks, status tracking)
- **openspec** = *what the system should look like* (specs, requirements, proposals)

## Starting a Session

1. Run `wai status` to see active projects, current phase, and suggestions.
2. Run `bd ready` to find available work items.
   Before claiming: read the relevant source files to confirm
   the issue is not already implemented.
3. Check `openspec list` for active change proposals.
4. Check the phase — it tells you what kind of work is expected:
   - **research** → gather information, explore options
   - **design** → make architectural decisions
   - **plan** → break work into tasks
   - **implement** → write code, guided by research/plans
   - **review** → validate against plans
   - **archive** → wrap up
5. Read existing artifacts with `wai search "<topic>"` before starting new work.

## Capturing Work

Record the reasoning behind your work, not just the output:

```bash
wai add research "findings"         # What you learned, trade-offs
wai add plan "approach"             # How you'll implement, why
wai add design "decisions"          # Architecture choices, rationale
wai add research --file notes.md    # Import longer content
```

Use `--project <name>` if multiple projects exist. Otherwise wai picks the first one.

Phases are a guide, not a gate. Use `wai phase show` / `wai phase next`.

## Tracking Work Across Tools

When beads and openspec are both active, keep them in sync:
- When creating a beads ticket for an openspec task, include the task
  reference in the description (format: `<change-id>:<phase>.<task>`,
  e.g. `add-why-command:7.1`)
- When closing a beads ticket linked to a task, also check the box
  (`[x]`) in the change's `tasks.md`

## Ending a Session

Before saying "done", run this checklist:

```
[ ] wai handoff create <project>   # capture context for next session
[ ] bd close <id>                  # close completed issues; also close parent epic if last sub-task
[ ] bd sync --from-main            # pull beads updates
[ ] openspec tasks.md — mark completed tasks [x]
[ ] wai reflect                    # update CLAUDE.md with project patterns (every ~5 sessions)
[ ] git add <files> && git commit  # commit code + handoff
```

### Autonomous Loop

One task per session. The resume loop:

1. `wai prime` — orient (shows ⚡ RESUMING if mid-task)
2. Work on the single task
3. `wai close` — capture state (run this before every `/clear`)
4. `git add <files> && git commit`
5. `/clear` — fresh context

→ Next session: `wai prime` shows RESUMING with exact next steps.

When context reaches ~40%: run `wai close`, then `/clear`.
Do NOT skip `wai close` — it enables resume detection.

## Quick Reference

### wai
```bash
wai status                    # Project status and next steps
wai add research "notes"      # Add research artifact
wai add plan "plan"           # Add plan artifact
wai add design "design"       # Add design artifact
wai search "query"            # Search across artifacts
wai why "why use TOML?"       # Ask why (LLM-powered oracle)
wai why src/config.rs         # Explain a file's history
wai reflect                   # Synthesize project patterns into CLAUDE.md
wai handoff create <project>  # Session handoff
wai phase show                # Current phase
wai doctor                    # Workspace health
```

### beads
```bash
bd ready                     # Available work
bd show <id>                 # Issue details
bd create --title="..."      # New issue
bd update <id> --status=in_progress
bd close <id>                # Complete work
```

### openspec
Read `openspec/AGENTS.md` for full instructions.
```bash
openspec list              # Active changes
openspec list --specs      # Capabilities
```

## Structure

The `.wai/` directory organizes artifacts using the PARA method:
- **projects/** — active work with phase tracking and dated artifacts
- **areas/** — ongoing responsibilities (no end date)
- **resources/** — reference material, agent configs, templates
- **archives/** — completed or inactive items

Do not edit `.wai/config.toml` directly. Use `wai` commands instead.

Keep this managed block so `wai init` can refresh the instructions.

<!-- WAI:END -->

<!-- WAI:REFLECT:START -->
## Project-Specific AI Context
_Last reflected: 2026-02-28 · 3 sessions analyzed_

### Conventions

- **Error handling pattern**: All IO errors use `.into_diagnostic()?` (miette crate). Never use `.unwrap()` or `.expect()` in production paths.
- **Config access**: LLM config is now `config.llm_config()` (returns `Cow<LlmConfig>`), not `config.why`. The `[why]` TOML section still deserializes for backwards compat but emits a deprecation warning.
- **Global flags**: Commands must OR-merge local flags with `current_context()` globals: `let json = json || current_context().json`. Never read *only* the subcommand-local flag — `wai --json <cmd>` must work.
- **Shared helpers**: `beads_summary`, `resolve_project_named`, `list_projects` live in `src/commands/mod.rs` as `pub(crate)` functions. Do not duplicate them in individual command files.
- **Managed block template**: `src/managed_block.rs::wai_block_content()` is the source of truth for the WAI managed block. Editing `CLAUDE.md` directly only changes the current repo — the template must also be updated for the change to propagate to other repos via `wai init`.
- **Phase badge colors**: research=yellow, design=magenta, plan=blue, implement=green, review=cyan, archive=dim. Defined in `status.rs::format_phase` — follow the same mapping in any new command that displays phases.

### Common Gotchas

- **`create_dir_all` before writes**: Always call `std::fs::create_dir_all(&dir).into_diagnostic()?` before `std::fs::write(dir.join(&filename), ...)`. The subdirectory may not exist (especially for fresh projects or manually-pruned repos).
- **`fs::rename` is not cross-device**: Use `move_item()` helper in `move_cmd.rs` which falls back to recursive copy+delete on EXDEV. Never call `std::fs::rename` directly for PARA item moves.
- **State captured BEFORE write**: When distinguishing Created vs Updated, always capture `let already_existed = path.exists()` *before* calling `fs::write`. The check after write always returns true.
- **Stdout vs stderr for diagnostics**: Any diagnostic/warning that fires inside a JSON output path must go to `eprintln!` (stderr), never `println!`. Stale-resume notices, deprecation warnings, and progress indicators all belong on stderr.
- **`ensure_workspace_current` does NOT update tool_commit**: As of the wai-cuq8 fix, `ensure_workspace_current` no longer touches `config.toml`. Call `sync_tool_commit()` explicitly from `wai init` only.
- **Doctor suppresses projection warnings on empty projections**: If `.projections.yml` parses with `projections: []`, `check_agent_tool_coverage` returns early with no warnings. This is intentional — explicit empty means "no projections wanted".
- **Pipeline state file**: Active run ID is stored in `.wai/.pipeline-run` (not committed). `wai add` reads env var first then falls back to the state file. `wai pipeline advance` clears it on last stage.

### Steps That Tend to Require Multiple Tries

- **Refactoring shared helpers**: When extracting a shared function, check ALL callers — grep for both the function name and any inline equivalent. `add.rs` has a `resolve_project` with a *different* signature from the shared one in `mod.rs`; don't conflate them.
- **Managed block changes**: After editing `managed_block.rs`, the actual `CLAUDE.md` must also be updated (run `wai init` or manually invoke `inject_managed_block`). Template and committed file must stay in sync.
- **Integration test helpers**: `tests/integration.rs` has helpers like `force_why_llm` and `set_privacy_notice_shown` that write TOML directly. After a config schema rename, update these helpers — they'll silently write the wrong section name otherwise.

### Architecture Notes

- **wai way vs wai doctor**: `wai doctor` = wai workspace health (broken .wai/, config.toml, projections, plugins). `wai way` = repo hygiene and agent workflow conventions (skills, rules — works without a wai workspace). They cross-reference each other in help text. Do not conflate.
- **Help system**: `src/help.rs` provides custom `HelpContent` structs for all top-level commands. Commands without entries fall back to clap. When adding a new command, add a corresponding entry in `command_help()`.
- **Pipeline runs**: Run IDs stored in `.wai/projects/<name>/pipelines/<id>.yaml`. Active run pointer is `.wai/.pipeline-run` (single-line, gitignored). `wai pipeline run` writes it; `wai pipeline advance` clears it on completion.
- **Suggestion thresholds**: `src/workflows.rs` uses `research_count <= 1` for "add more research" and `>= 2` for "ready to advance". Thresholds are adjacent with no dead zones. Maintain adjacency when adding new threshold-based suggestions.
- **JSON output pattern**: Use `print_json(&payload)` from `src/json.rs`. Structs derive `Serialize`. Use `#[serde(skip_serializing_if = "Option::is_none")]` and `#[serde(skip_serializing_if = "Vec::is_empty")]` to keep output clean.
<!-- WAI:REFLECT:END -->


<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

<!-- BEADS:START -->
# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

Keep this managed block so `bd onboard` can refresh the instructions.
<!-- BEADS:END -->
