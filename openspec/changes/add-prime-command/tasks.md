## 1. CLI definition
- [ ] 1.1 Add `Prime { project: Option<String> }` variant to `Commands` enum in `src/cli.rs`
- [ ] 1.2 Add `"prime"` to `valid_commands` in the external subcommand handler

## 2. Handoff reading
- [ ] 2.1 Add helper `find_latest_handoff(project_root: &Path, project: &str) -> Result<Option<PathBuf>>` that globs `.wai/projects/<project>/handoffs/*.md`, sorts by filename descending, and returns the most recent path
- [ ] 2.2 Add helper `read_handoff_summary(path: &Path) -> Result<(NaiveDate, String)>` that parses YAML frontmatter for `date` and reads the first non-empty, non-heading paragraph as the summary snippet (truncated to 80 characters)

## 3. Command implementation
- [ ] 3.1 Create `src/commands/prime.rs`
- [ ] 3.2 Auto-detect project: reuse same logic as `wai close` (fail when none, use single, interactive prompt when multiple, fail with `--no-input` flag set)
- [ ] 3.3 Read current phase from `.wai/projects/<project>/.state`
- [ ] 3.4 Call plugin status hook to collect per-plugin one-line summaries (same mechanism as `wai status`)
- [ ] 3.5 Invoke `bd ready` to get the highest-priority ready issue; capture its ID + title for the suggested-next line; skip silently if beads plugin not detected or command fails
- [ ] 3.6 Render the prime view:
  ```
  ◆ wai prime — <today>
  • Project: <name> [<phase>]
  • Handoff: <date> — '<snippet>'      (omit when no handoff exists)
  • <plugin-summary-line>              (one per detected plugin, in detection order)
  → Suggested next: bd show <id>       (omit when beads not detected or no ready issues)
  ```

## 4. Wire up
- [ ] 4.1 Dispatch `Commands::Prime` in `src/commands/mod.rs`
- [ ] 4.2 Verify `wai prime --help` output describes the command and references `--project`

## 5. Tests
- [ ] 5.1 Integration test: single project workspace with handoff → full output rendered correctly
- [ ] 5.2 Integration test: no handoff exists → handoff line omitted, rest of output intact
- [ ] 5.3 Integration test: `--project <name>` flag selects correct project without prompting
- [ ] 5.4 Integration test: zero projects workspace → diagnostic error suggesting `wai new project`
