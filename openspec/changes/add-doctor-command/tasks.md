## 1. CLI Wiring

- [ ] 1.1 Add `Doctor` variant to `Commands` enum in `src/cli.rs`
- [ ] 1.2 Add `mod doctor` and dispatch arm in `src/commands/mod.rs`
- [ ] 1.3 Create `src/commands/doctor.rs` with `pub fn run() -> Result<()>` skeleton

## 2. Diagnostic Checks

- [ ] 2.1 Implement directory structure check (verify PARA dirs exist under `.wai/`)
- [ ] 2.2 Implement config.toml validation check (parse and report errors)
- [ ] 2.3 Implement plugin tool availability check (run `which`/`command -v` for detected plugin CLIs)
- [ ] 2.4 Implement agent config sync check (validate `.projections.yml`, check target sync status)
- [ ] 2.5 Implement project state check (validate `.state` YAML files across all projects)
- [ ] 2.6 Implement custom plugin YAML validation check (parse files in `.wai/plugins/`)

## 3. Output and Exit

- [ ] 3.1 Implement pass/warn/fail formatting with fix suggestions per check
- [ ] 3.2 Implement summary line with counts and exit code (0 = all pass, 1 = any fail)

## 4. Testing

- [ ] 4.1 Add integration tests for healthy workspace (all checks pass)
- [ ] 4.2 Add integration tests for missing directories (fail with fix suggestion)
- [ ] 4.3 Add integration tests for invalid config.toml (fail with suggestion)
- [ ] 4.4 Add integration tests for missing plugin tools (warn)
- [ ] 4.5 Add integration tests for invalid state files (fail)
- [ ] 4.6 Add integration test for uninitialised directory (error with init suggestion)
