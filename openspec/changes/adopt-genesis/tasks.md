## 1. Dependency
- [ ] 1.1 Add `genesis = { git = "https://github.com/charly-vibes/genesis", tag = "v0.1.0" }` to `Cargo.toml`.
- [ ] 1.2 Verify the build with genesis's `suggestions`/`managed_block`/`envelope` modules stable.

## 2. Migrate suggestions
- [ ] 2.1 Delete `src/suggestions.rs`; replace with `pub use genesis::suggestions::*;` in `lib.rs`.
- [ ] 2.2 Update `main.rs`/command handlers to register wai's command list with genesis's `SuggestionEngine`.
- [ ] 2.3 Regression: `wai statos` (typo) still prints "Did you mean 'status'?".

## 3. Migrate managed_block
- [ ] 3.1 Replace the injector mechanics in `src/managed_block.rs` with `genesis::managed_block` calls.
- [ ] 3.2 Keep wai's block *content* generation (plugin-aware, slim Layer-1) in wai.
- [ ] 3.3 Regression: `wai sync` still injects/refreshes `<!-- WAI:START -->` blocks.

## 4. Adopt shared envelope
- [ ] 4.1 Route `output::print_json` through `genesis::envelope::Envelope`.
- [ ] 4.2 Wrap `PrimePayload`, `status --json`, and other `--json` outputs.
- [ ] 4.3 Test: `wai status --json` top-level keys match the shared shape.

## 5. Clean up
- [ ] 5.1 Remove now-dead local code; `cargo clippy -- -D warnings` clean.
- [ ] 5.2 Update `llm.txt` to note shared-infra origin.
- [ ] 5.3 Verify tool-craft playbook (genesis `.wai` research) Appendix A.3 (wai row) is accurate once genesis is adopted; file a charly-monorepo ticket if not — do not edit the charly repo from this wai change.
