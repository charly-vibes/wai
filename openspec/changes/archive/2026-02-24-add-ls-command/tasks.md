## 1. CLI definition
- [x] 1.1 Add `Ls { root: Option<PathBuf>, depth: Option<usize> }` variant to `Commands` enum in `src/cli.rs`
- [x] 1.2 Add `"ls"` to `valid_commands` in the external subcommand handler

## 2. Workspace discovery
- [x] 2.1 Add `discover_workspaces(root: &Path, max_depth: usize) -> Vec<PathBuf>` that walks the filesystem up to `max_depth` levels, using `follow_links: false` to prevent symlink cycles; for each directory, check if `.wai/config.toml` exists within it (match = wai workspace) and do NOT recurse into `.wai/` itself; skip all other hidden directories (names starting with `.`) during traversal
- [x] 2.2 For each workspace, read project names from `.wai/config.toml`
- [x] 2.3 For each project, read phase from `.wai/projects/<project>/.state`; default to `unknown` on missing file or parse error

## 3. Plugin integration (beads, when detected)
- [x] 3.1 If `.beads/` exists in the workspace directory, invoke `bd stats --json` to get `open` and `ready` counts; skip gracefully if `bd` is not installed or exits non-zero
- [x] 3.2 Store per-project counts alongside the phase for use in rendering

## 4. Command implementation
- [x] 4.1 Create `src/commands/ls.rs`
- [x] 4.2 Resolve root: use `--root <path>` if provided (fail with diagnostic if path does not exist), else `$HOME` via `dirs::home_dir()` (already in `Cargo.toml`); fail with diagnostic if home directory cannot be determined
- [x] 4.3 Resolve depth: use `--depth <n>` if provided, else default to 3
- [x] 4.4 Run workspace discovery, collect results sorted by project name
- [x] 4.5 Render table: one line per (workspace, project) pair with columns left-aligned and padded to the longest name; **show the counts column for ALL rows** when at least one project has beads data, leave the cell blank for projects without beads data; omit the counts column entirely when no project has beads data. When two projects share the same name (from different workspaces), append a short disambiguating path suffix to each: `name (~/path/to/repo)`
- [x] 4.6 Print `No wai workspaces found under <root>` when discovery returns nothing

## 5. Wire up
- [x] 5.1 Dispatch `Commands::Ls` in `src/commands/mod.rs`
- [x] 5.2 Verify `wai ls --help` output describes the command and references `--root` and `--depth`

## 6. Tests
- [x] 6.1 Integration test: root with one workspace and one project → one-line output with correct phase
- [x] 6.2 Integration test: root with multiple workspaces → all projects shown, sorted by name
- [x] 6.3 Integration test: no workspaces found under root → empty-output message
- [x] 6.4 Integration test: workspace with beads detected → counts column shown for all rows; blank cell for rows without beads
- [x] 6.5 Integration test: no workspace has beads → counts column omitted entirely
- [x] 6.6 Integration test: `--depth 1` stops recursion at the expected level
- [x] 6.7 Integration test: `--root /nonexistent` → diagnostic error with the invalid path
