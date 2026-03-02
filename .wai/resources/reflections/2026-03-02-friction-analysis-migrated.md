---
date: "2026-03-02"
project: "friction-analysis"
type: reflection-migrated
---

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
- **Pipeline runs**: Run IDs stored in `.wai/projects/<name>/pipelines/<id>.yaml`. Active run pointer is `.wai/.pipeline-run` (single-line, gitignored). `wai pipeline run` writes it; `wai pipeline advance` clears it on last stage.
- **Suggestion thresholds**: `src/workflows.rs` uses `research_count <= 1` for "add more research" and `>= 2` for "ready to advance". Thresholds are adjacent with no dead zones. Maintain adjacency when adding new threshold-based suggestions.
- **JSON output pattern**: Use `print_json(&payload)` from `src/json.rs`. Structs derive `Serialize`. Use `#[serde(skip_serializing_if = "Option::is_none")]` and `#[serde(skip_serializing_if = "Vec::is_empty")]` to keep output clean.
