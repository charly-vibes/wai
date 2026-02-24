## 1. CLI definition
- [ ] 1.1 Add `Close { project: Option<String> }` variant to `Commands` enum in `src/cli.rs`

## 2. Command implementation
- [ ] 2.1 Create `src/commands/close.rs`
- [ ] 2.2 Auto-detect project: fail with "no projects" diagnostic if none exist; use the single project if only one exists; prompt (cliclack select) if multiple; fail with diagnostic when `--no-input` and multiple projects
- [ ] 2.3 Extract `create_handoff(project_root: &Path, project: &str) -> Result<PathBuf>` from `src/commands/handoff.rs`; both `wai handoff create` and `wai close` call it
- [ ] 2.4 Read git status: shell out to `git status --porcelain`; treat non-zero exit (git not installed, not a git repo) as "no git" and skip the git section silently
- [ ] 2.5 Render uncommitted files as a single `!`-prefixed line; cap at 10 files and append `… and N more` when exceeded
- [ ] 2.6 Print plugin-aware next-steps: include `bd sync --from-main &&` prefix only when beads plugin is detected; include the uncommitted filenames in the `git add` suggestion, each wrapped in double-quotes to handle paths with spaces

## 3. Wire up
- [ ] 3.1 Dispatch `Commands::Close` in `src/commands/mod.rs`
- [ ] 3.2 Add `"close"` to `valid_commands` in the external subcommand handler
- [ ] 3.3 Verify `wai close --help` output matches the expected format and references `--project`

## 4. Tests
- [ ] 4.1 Integration test: single project workspace → handoff created, output matches expected format
- [ ] 4.2 Integration test: `--project <name>` flag creates handoff without any interactive prompt
- [ ] 4.3 Integration test: unknown project name → diagnostic error lists available projects
- [ ] 4.4 Integration test: zero projects workspace → diagnostic error suggests `wai new project`
