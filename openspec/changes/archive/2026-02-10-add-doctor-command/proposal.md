# Change: Add Doctor Command

## Why

Users have no way to verify that their wai workspace is healthy — that the `.wai/` directory structure is complete, configs parse correctly, plugin tools are installed, projections are valid, and project state files are intact. When something silently breaks (e.g., a missing directory, a corrupt YAML file, or a plugin CLI that was uninstalled), they only discover it when an unrelated command fails with a confusing error. A dedicated `wai doctor` command provides proactive diagnosis with actionable fix suggestions, consistent with wai's self-healing error philosophy.

## What Changes

- Add `wai doctor` top-level command to cli-core
- Implement six diagnostic checks, each producing pass/warn/fail results:
  1. **Directory structure** — verifies all expected `.wai/` subdirectories exist
  2. **Configuration** — validates `config.toml` parses correctly
  3. **Plugin tools** — checks whether detected plugin CLIs are installed and reachable (`git`, `bd`, `openspec`)
  4. **Agent config sync** — validates `.projections.yml` and checks projection sync status
  5. **Project state** — validates `.state` files across all projects
  6. **Custom plugins** — validates YAML syntax for user-defined plugin configs in `.wai/plugins/`
- Each failing check includes a suggested fix command or action
- Summary line reports total pass/warn/fail counts
- Exit code 0 when all pass, 1 when any fail

## Impact

- Affected specs: `cli-core` (new command added to command structure)
- Affected code: `src/cli.rs`, `src/commands/mod.rs`, new `src/commands/doctor.rs`
- No breaking changes
- Leverages existing `config.rs`, `plugin.rs`, `state.rs` modules
