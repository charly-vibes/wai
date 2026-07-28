# Change: Adopt genesis (donor side)

## Why

wai is the canonical donor for three of genesis's modules (`suggestions`,
`managed_block`, `aix` block generation) and one of the JSON-envelope
holdouts. The genesis foundation proposal (`genesis/openspec/changes/
add-genesis-foundation`) extracts these into the shared crate. This change
makes wai a *consumer* of genesis: it removes its local copies and re-imports
them from `genesis`, so future improvements land once and propagate.

## What Changes

- Add the `genesis` crate to `Cargo.toml`. Originally specced as a git-tag
  dep on `v0.1.0`; switched to crates.io once published — shipped as
  `genesis-vibes = "0.2"` (crate name `genesis-vibes`, lib name `genesis`).
- Replace `src/suggestions.rs` with a re-export of `genesis::suggestions`.
- Replace `src/managed_block.rs` with a thin wrapper over
  `genesis::managed_block` (wai keeps its block *content* / plugin-aware
  logic, but the injector mechanics come from genesis).
- Route `--json` output through `genesis::envelope` (wai currently emits
  bare domain objects like `PrimePayload`).
- Keep all domain logic in wai (PARA state, phases, handoffs, `way`/`why`/
  `reflect`, plugin system). The boundary rule (genesis §design) protects this.

## Impact

- Affected specs: `context-suggestions`, `managed-block`, `cli-core`
  (envelope wrapping). Deltas to follow.
- Affected code: `Cargo.toml`, `src/suggestions.rs`, `src/managed_block.rs`,
  `src/json.rs`, callers of `print_json`.
- Blocked by: `genesis` foundation proposal tagging `v0.1.0`.
- No user-visible behavior change except `--json` envelopes gain shared keys.
