## 1. Dependency
- [x] 1.1 Add the `genesis` crate to `Cargo.toml` (shipped as `genesis-vibes = "0.2"` on crates.io; lib name `genesis`. Originally specced as a git-tag dep on `v0.1.0`; switched to crates.io once published).
- [x] 1.2 Verify the build with genesis's `suggestions`/`managed_block`/`envelope` modules stable.

## 2. Migrate suggestions
- [x] 2.1 Delete `src/suggestions.rs`; replace with `pub use genesis::suggestions::*;` in `lib.rs`.
- [x] 2.2 Update `main.rs`/command handlers to register wai's command list with genesis's `SuggestionEngine`.
- [x] 2.3 Regression: `wai statos` (typo) still prints "Did you mean 'status'?".

## 3. Migrate managed_block
- [x] 3.1 Replace the injector mechanics in `src/managed_block.rs` with `genesis::managed_block` calls.
- [x] 3.2 Keep wai's block *content* generation (plugin-aware, slim Layer-1) in wai.
- [x] 3.3 Regression: `wai sync` still injects/refreshes `<!-- WAI:START -->` blocks.

## 4. Adopt shared envelope
- [x] 4.1 Route `output::print_json` through `genesis::envelope::Envelope`.
- [x] 4.2 Wrap `PrimePayload`, `status --json`, and other `--json` outputs.
- [x] 4.3 Test: `wai status --json` top-level keys match the shared shape.

## 5. Clean up
- [x] 5.1 Remove now-dead local code; `cargo clippy -- -D warnings` clean.
- [x] 5.2 Update `llm.txt` to note shared-infra origin.
- [x] 5.3 Verify tool-craft playbook (genesis `.wai` research) Appendix A.3 (wai row) is accurate once genesis is adopted; file a charly-monorepo ticket if not — do not edit the charly repo from this wai change. _(Verified 2026-07-28: wai row checkmarks still hold post-adoption — wai exercises all six artifacts, now partly via genesis. The broader matrix refresh for adoption effects is tracked in genesis ticket `genesis-9o5`.)_
