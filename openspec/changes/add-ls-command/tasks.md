## 1. CLI definition
- [ ] 1.1 Add `Ls { root: Option<PathBuf>, depth: Option<usize> }` variant to `Commands` enum in `src/cli.rs`
- [ ] 1.2 Add `"ls"` to `valid_commands` in the external subcommand handler

## 2. Workspace discovery
- [ ] 2.1 Add `discover_workspaces(root: &Path, max_depth: usize) -> Vec<PathBuf>` that walks the filesystem looking for `.wai/config.toml`, skipping hidden directories other than `.wai/`; stops recursing at `max_depth`
- [ ] 2.2 For each workspace, read project names from `.wai/config.toml`
- [ ] 2.3 For each project, read phase from `.wai/projects/<project>/.state`; default to `unknown` on parse error

## 3. Plugin integration
- [ ] 3.1 If `.beads/` exists in the workspace directory, invoke `bd stats --json` to get `open` and `ready` counts; skip gracefully if `bd` is not installed or exits non-zero
- [ ] 3.2 Store per-project counts alongside the phase for use in rendering

## 4. Command implementation
- [ ] 4.1 Create `src/commands/ls.rs`
- [ ] 4.2 Resolve root: use `--root <path>` if provided, else `$HOME` via `dirs::home_dir()`; fail with diagnostic if home directory cannot be determined
- [ ] 4.3 Resolve depth: use `--depth <n>` if provided, else default to 3
- [ ] 4.4 Run workspace discovery, collect results sorted by project name
- [ ] 4.5 Render table: one line per (workspace, project) pair with columns left-aligned and padded to the longest name; include counts column only when at least one project has beads data
- [ ] 4.6 Print `No wai workspaces found under <root>` when discovery returns nothing

## 5. Wire up
- [ ] 5.1 Dispatch `Commands::Ls` in `src/commands/mod.rs`
- [ ] 5.2 Verify `wai ls --help` output describes the command and references `--root` and `--depth`

## 6. Tests
- [ ] 6.1 Integration test: root with one workspace and one project → one-line output with correct phase
- [ ] 6.2 Integration test: root with multiple workspaces → all projects shown, sorted by name
- [ ] 6.3 Integration test: no workspaces found under root → empty-output message
- [ ] 6.4 Integration test: workspace with beads detected → counts column shown
- [ ] 6.5 Integration test: workspace without beads → counts column omitted
- [ ] 6.6 Integration test: `--depth 1` stops recursion at the expected level
