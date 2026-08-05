## 1. Implementation

- [x] 1.1 Add `TrustStore` type with `is_approved(digest)`, `approve(digest)`, `revoke(digest)`, `list()` methods, backed by XDG state file
- [x] 1.2 Add `compute_hook_digest(plugin_def, hook_name, hook_def) -> String` — SHA-256 of canonical JSON representation
- [x] 1.3 Add trust gate in `execute_hook()`: skip unapproved hooks, emit warning (human) or structured output (machine)
- [x] 1.4 Built-in plugins auto-approved: their definitions are compiled into binary, not attacker-controlled
- [x] 1.5 Add `wai plugin trust <name> [--hook <hook-name>]` subcommand
- [x] 1.6 Add `wai plugin trust --list` subcommand
- [x] 1.7 Add `wai plugin trust --revoke <digest>` subcommand
- [x] 1.8 Add `TrustStore` path to config (XDG state dir)
- [x] 1.9 Tests: malicious-repository test proving `wai status`, `prime`, `new`, `phase`, `handoff` do not execute unapproved hooks
- [x] 1.10 Tests: approving a plugin digest enables it
- [x] 1.11 Tests: modifying manifest or command invalidates approval
- [x] 1.12 Tests: revocation disables it
- [x] 1.13 Tests: non-interactive/CI fail-closed without prompting
- [x] 1.14 Tests: built-in plugins remain functional
- [x] 1.15 OpenSpec validation and `cargo test plugin && cargo test --test status_test` suites pass