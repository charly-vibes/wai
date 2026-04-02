## 1. Core resolution function

- [x] 1.1 Create unified `resolve_project()` in `src/commands/mod.rs` with
      resolution order: flag → env → auto-detect → interactive/error
- [x] 1.2 Add `WAI_PROJECT` env var reading (treat empty string as unset,
      validate project existence)
- [x] 1.3 Add resolution-source tracking (returns how project was resolved:
      Flag, EnvVar, AutoDetect, Interactive) for display in status commands
- [x] 1.4 Auto-detect counts `.wai/projects/` only (not areas/resources/archives)
- [x] 1.5 Preserve interactive selection when stdin is TTY and `--no-input` not
      set; error with hint otherwise
- [x] 1.6 Write unit tests for each resolution tier and edge cases
      (missing env project, empty env, multiple projects no env, single project,
      non-interactive fallback)

## 2. Add `--project` to phase commands

- [x] 2.1 Add `--project` optional arg to the `Phase` variant in the parent
      `Commands` enum in `cli.rs` (not on individual `PhaseCommands` variants)
- [x] 2.2 Update `phase.rs` to call unified `resolve_project()` instead of
      `find_active_project()`
- [x] 2.3 Remove `find_active_project()` function
- [x] 2.4 Write integration tests: `wai phase show --project <name>`,
      `wai phase next --project <name>`, `WAI_PROJECT=<name> wai phase show`

## 3. Migrate existing commands to unified resolution

- [x] 3.1 Migrate `add.rs` to use unified `resolve_project()` (note: auto-detect
      scope changes from projects+areas to projects-only)
- [x] 3.2 Migrate `close.rs` to use unified `resolve_project()`
- [x] 3.3 Migrate `prime.rs` to use unified `resolve_project()`
- [x] 3.4 Migrate `reflect.rs` to use unified `resolve_project()`
- [x] 3.5 Remove old `resolve_project_named()` and `resolve_project()` from
      add.rs after migration
- [x] 3.6 Verify all commands produce consistent error messages when project
      not found

## 4. `wai project use` command

- [x] 4.1 Add `project use <name>` subcommand to CLI definitions
- [x] 4.2 Implement: validate project exists, detect shell from `$SHELL`,
      print appropriate export syntax (bash/zsh: `export`, fish: `set -gx`)
- [x] 4.3 When stdout is TTY, print usage hint to stderr
- [x] 4.4 Without args: list available projects with phases
- [x] 4.5 Write tests for valid project, invalid project, no-args listing,
      fish shell detection

## 5. Resolution source display

- [x] 5.1 Update `wai status` to show resolved project name with source indicator
      (e.g., `[via --project]`, `[via WAI_PROJECT]`, `[auto-detected]`)
- [x] 5.2 Update `wai prime` to show source indicator in project header
- [x] 5.3 Update `wai phase show` to show source indicator

## 6. Doctor integration

- [x] 6.1 Add `wai doctor` check: if `WAI_PROJECT` is set, verify the named
      project exists (warn if not)
- [x] 6.2 Add `wai doctor` check: if `WAI_PROJECT` is set to empty string,
      suggest unsetting it

## 7. Documentation

- [x] 7.1 Update CLAUDE.md quick reference with `WAI_PROJECT` usage
- [x] 7.2 Update `wai --help` / `wai phase --help` to mention env var
