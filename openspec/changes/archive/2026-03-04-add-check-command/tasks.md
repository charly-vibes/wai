## 1. Unify `CheckResult` type

- [ ] 1.1 Define a shared `CheckResult` struct and `Status` enum (Pass/Info/Warn/Fail)
  in a new module (e.g., `src/checks.rs`) or expose from one command and import
  in the other
- [ ] 1.2 Update `way.rs` to use the shared type (replacing its local `Status { Pass, Info }`)
- [ ] 1.3 Update `doctor.rs` to use the shared type (replacing its local `Status { Pass, Warn, Fail }`)

## 2. Refactor `way.rs` — extract `run_checks()`

- [ ] 2.1 Extract the check-building logic from `way::run()` into
  `pub fn run_checks(repo_root: &Path) -> Vec<CheckResult>`
- [ ] 2.2 Keep `run()` calling `run_checks()` internally — presentation and exit
  code behaviour in `run()` must remain identical to before this refactor
- [ ] 2.3 Remove `check_beads()` and `check_openspec()` calls from `way::run_checks()`
  (they move to `doctor` in task 3)

## 3. Refactor `doctor.rs` — extract `run_checks()` and absorb beads/openspec

- [ ] 3.1 Extract the check-building logic from `doctor::run()` into
  `pub fn run_checks(project_root: &Path) -> Vec<CheckResult>`
- [ ] 3.2 Keep `run()` calling `run_checks()` internally — presentation and exit
  code behaviour in `run()` must remain identical to before this refactor
- [ ] 3.3 Add `check_beads()` and `check_openspec()` calls inside `doctor::run_checks()`
  (moved from `way.rs`)

## 4. Create `src/commands/check.rs`

- [ ] 4.1 Implement `pub fn run(way_only: bool, doctor_only: bool) -> Result<()>`
- [ ] 4.2 When `doctor_only` is false: call `way::run_checks()`, render under
  "Repo hygiene (wai way)" section header
- [ ] 4.3 When `way_only` is false: attempt `doctor::run_checks()`; if no `.wai/`
  workspace exists, print "(workspace checks skipped — run `wai init` first)"
  and skip without error
- [ ] 4.4 Combine summaries and print combined totals line
- [ ] 4.5 Exit with code 1 if any check across both sections has status Fail;
  exit 0 otherwise
- [ ] 4.6 Support `--json` flag: output `{ "way": { checks, summary }, "doctor": { checks, summary } }`

## 5. Register `wai check` in CLI (`src/cli.rs`)

- [ ] 5.1 Add `Check` variant to the `Commands` enum with `--way-only` and
  `--doctor-only` bool flags
- [ ] 5.2 Add `"check"` to the `expected` list in the `derived_list_contains_all_known_commands` test

## 6. Wire dispatch (`src/commands/mod.rs` or `src/main.rs`)

- [ ] 6.1 Add `Commands::Check { way_only, doctor_only }` arm in the command
  dispatch, calling `check::run(way_only, doctor_only)`

## 7. Update `wai init` next-steps

- [ ] 7.1 Add `wai check` to the Quick Reference section in the `wai init`
  managed block template, positioned after `wai doctor`

## 8. Update `wai way` and `wai doctor` help text

- [ ] 8.1 In `src/cli.rs`, append a sentence to the `Way` command's `long_about`
  pointing users to `wai check` as the umbrella command
- [ ] 8.2 In `src/cli.rs`, append a sentence to the `Doctor` command's `long_about`
  pointing users to `wai check` as the umbrella command

## 9. Tests

- [ ] 9.1 Integration test: `wai check` in a workspace with `.wai/` runs both sections
  and exits 0 when all checks pass
- [ ] 9.2 Integration test: `wai check` outside a workspace runs way section and
  prints the skip message for doctor section
- [ ] 9.3 Integration test: `wai check --way-only` runs only the way section
  (doctor section absent from output)
- [ ] 9.4 Integration test: `wai check --doctor-only` runs only the doctor section
- [ ] 9.5 Integration test: `wai check --json` outputs valid JSON with both
  `way` and `doctor` keys
- [ ] 9.6 Unit test: `wai check` exit code is 1 when any doctor check is Fail
- [ ] 9.7 Regression test: `wai way` still exits 0 after beads/openspec checks
  are removed from it (they have no Fail status in `way`, but verify output)
- [ ] 9.8 Regression test: `wai doctor` shows beads/openspec checks after move
