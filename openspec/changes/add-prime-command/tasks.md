## 1. CLI definition
- [x] 1.1 Add `Prime { project: Option<String> }` variant to `Commands` enum in `src/cli.rs`
- [x] 1.2 Add `"prime"` to `valid_commands` in the external subcommand handler

## 2. Handoff reading
- [x] 2.1 Add helper `find_latest_handoff(project_root: &Path, project: &str) -> Result<Option<PathBuf>>` that globs `.wai/projects/<project>/handoffs/*.md`, sorts by filename descending, and returns the most recent path
- [x] 2.2 Add helper `read_handoff_summary(path: &Path) -> (NaiveDate, String)` that parses YAML frontmatter for `date` and reads the first non-empty, non-heading paragraph as the summary snippet (truncated to 80 characters); if frontmatter is missing, unparseable, or the `date` field is absent, return today's date as fallback; if no paragraph is found after all headings (e.g. freshly generated handoff with no content yet), use `"no summary yet"` as the snippet

## 3. Command implementation
- [x] 3.0 Verify the plugin status hook entry points used by `wai status`: `plugin::run_hooks(&project_root, "on_status")` (in `src/commands/status.rs:89`) and `openspec::read_status(&project_root)` (in `src/commands/status.rs:92`). These are the two functions prime.rs MUST call — do not duplicate the logic.
- [x] 3.1 Create `src/commands/prime.rs`
- [x] 3.2 Auto-detect project: reuse same logic as `wai close` (fail when none, use single, interactive prompt when multiple, fail with `--no-input` flag set)
- [x] 3.3 Read current phase from `.wai/projects/<project>/.state`; if the file is missing or the YAML fails to parse, use `"unknown"` as the phase — do not fail the command
- [x] 3.4 Collect plugin status summaries by calling `plugin::run_hooks(project_root, "on_status")` and `openspec::read_status(project_root)` (see task 3.0); format each hook output as a single bullet line using the same condensed format as `wai status`'s Plugin Info block
- [x] 3.5 Get suggested next action: invoke `bd ready --json`, parse the first element's `id` field; skip silently (no suggested-next line) if the beads plugin is not detected, the command fails, or the JSON array is empty
- [x] 3.6 Render the prime view using today's date in `YYYY-MM-DD` format (local system clock):
  ```
  ◆ wai prime — <YYYY-MM-DD>
  • Project: <name> [<phase>]
  • Handoff: <date> — '<snippet>'      (omit when no handoff exists)
  • <plugin-summary-line>              (one per detected plugin, in detection order)
  → Suggested next: bd show <id>       (omit when beads not detected or no ready issues)
  ```

## 4. Wire up
- [x] 4.1 Dispatch `Commands::Prime` in `src/commands/mod.rs`
- [x] 4.2 Verify `wai prime --help` output describes the command and references `--project`

## 5. Tests
- [x] 5.1 Integration test: single project workspace with handoff → full output rendered correctly
- [x] 5.2 Integration test: no handoff exists → handoff line omitted, rest of output intact
- [x] 5.3 Integration test: `--project <name>` flag selects correct project without prompting
- [x] 5.4 Integration test: zero projects workspace → diagnostic error suggesting `wai new project`
- [x] 5.5 Integration test: multiple projects, `--no-input` → diagnostic error suggests `wai prime --project <name>`
- [x] 5.6 Integration test: `--project <name>` with unknown name → diagnostic error listing available projects
- [x] 5.7 Integration test: handoff with no paragraph content (all headings, no body) → snippet shows `"no summary yet"`
