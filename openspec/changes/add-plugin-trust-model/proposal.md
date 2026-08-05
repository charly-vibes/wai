# Change: Add plugin hook trust model

## Why

Repository plugin hooks auto-execute with no trust gate, creating a critical RCE
vulnerability (SEC-1). Any `.toml` file under `.wai/plugins/` (attacker-controlled
repo content) is loaded and executed on `wai status`, `prime`, `new`, `phase`,
and `handoff` — without user confirmation or allowlist. This is the single most
consequential finding across all 18 audit reports.

## What Changes

- **Trust store**: User-owned XDG state directory (`~/.local/share/wai/`) stores
  approved plugin digests, keyed by SHA-256 of the plugin manifest + hook definition.
- **Trust gate**: Before executing any hook, `execute_hook()` checks whether the
  hook's digest is in the trust store. Unapproved hooks are skipped with a warning.
- **Built-in plugins**: Always trusted — they ship with wai and are not attacker-controlled.
- **Commands**: `wai plugin trust <name> [--hook <hook-name>]` to approve a hook,
  `wai plugin trust --list` to list approved digests, `wai plugin trust --revoke <digest>`
  to remove approval.
- **Non-interactive/safe mode**: Fail-closed — unapproved hooks are never executed
  and never prompt. Only explicit trust commands can change the store.
- **Content invalidation**: Any modification to the plugin manifest or hook command
  changes the digest, invalidating prior approval.

## Impact

- Affected specs: `plugin-system`
- Affected code:
  - `src/plugin.rs` (trust gate in `execute_hook`, digest computation, trust store)
  - `src/commands/plugin.rs` (trust/list/revoke subcommands)
  - `src/commands/status.rs`, `prime.rs`, `new.rs`, `phase.rs`, `handoff.rs` (trust gate already centralized in `run_hooks`)
  - `src/config.rs` (XDG state dir path)
- Security: P0 RCE fix — malicious `.wai/plugins/*.toml` hooks are no longer auto-executed