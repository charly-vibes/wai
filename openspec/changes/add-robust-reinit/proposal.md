# Change: Add robust doctor-to-reinit workflow

## Why

When `wai doctor` detects issues and recommends `wai init`, the re-init path is shallow — it only updates `config.toml` and managed blocks. It doesn't create missing directories, missing default files, or detect version staleness. Users end up in a loop where doctor says "run init" but init doesn't fix the actual problem.

Three edge cases need handling:
1. **Older version**: workspace initialized with older wai, new version expects more artifacts (e.g. new subdirs, .gitignore)
2. **Incomplete init**: some directories/files missing (partial init, manual deletion)
3. **Missing new artifacts**: new wai version introduces files that old init didn't create

## What Changes

- Extract shared `ensure_workspace_current` function from init.rs that comprehensively creates/repairs all expected workspace artifacts
- Add version staleness check to doctor.rs (compares config.toml version against running binary)
- Expand directory completeness check in doctor.rs to cover all expected dirs (agent-config, templates, patterns), not just top-level PARA dirs
- Wire doctor `--fix` to call the shared function for workspace repair
- Re-init path in `wai init` calls the same shared function for consistency

## Impact

- Affected specs: `doctor-auto-fix`, `onboarding`
- Affected code: `src/commands/init.rs`, `src/commands/doctor.rs`, `tests/integration.rs`
