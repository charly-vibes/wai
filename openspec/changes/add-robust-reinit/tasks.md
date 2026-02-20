## 1. Shared workspace repair function

- [ ] 1.1 Extract `ensure_workspace_current(project_root: &Path) -> Result<Vec<RepairAction>>` in `init.rs`
- [ ] 1.2 Function creates all expected directories (PARA + agent-config subdirs + templates + patterns)
- [ ] 1.3 Function creates missing default files (.gitignore, .projections.yml, PLUGINS.md)
- [ ] 1.4 Function updates config.toml version to current binary version
- [ ] 1.5 Function injects/refreshes managed blocks in agent instruction files
- [ ] 1.6 Function returns a list of actions taken (for reporting to caller)

## 2. Doctor version staleness check

- [ ] 2.1 Add `check_version` function to `doctor.rs`
- [ ] 2.2 Compare `config.toml` `project.version` against `env!("CARGO_PKG_VERSION")`
- [ ] 2.3 Report Warn status when versions differ, with fix suggestion
- [ ] 2.4 Wire fix_fn to call `ensure_workspace_current`

## 3. Doctor expanded directory check

- [ ] 3.1 Expand `check_directories` to check agent-config subdirs (skills, rules, context)
- [ ] 3.2 Check resource subdirs (templates, patterns)
- [ ] 3.3 Check for missing default files (.gitignore, .projections.yml)
- [ ] 3.4 Wire fix_fn to call `ensure_workspace_current`

## 4. Re-init uses shared function

- [ ] 4.1 Refactor `init.rs` re-init branch to call `ensure_workspace_current`
- [ ] 4.2 Report actions taken to user (created dirs, updated config, etc.)

## 5. Integration tests

- [ ] 5.1 Test: doctor detects version mismatch and fix repairs it
- [ ] 5.2 Test: doctor detects missing agent-config subdirs and fix creates them
- [ ] 5.3 Test: re-init creates missing directories and updates version
- [ ] 5.4 Test: re-init is idempotent (running twice produces same result)
