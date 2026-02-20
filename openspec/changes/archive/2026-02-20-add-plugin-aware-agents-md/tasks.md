## 1. Core: Make managed block plugin-aware

- [x] 1.1 Change `wai_block_content()` signature to accept `detected_plugins: &[&str]` in `src/managed_block.rs`
- [x] 1.2 Change `inject_managed_block()` signature to accept and forward `detected_plugins: &[&str]`
- [x] 1.3 Change `inject_agent_instructions()` in `src/commands/init.rs` to accept `detected: &[&str]` and pass to `inject_managed_block()`
- [x] 1.4 Update both call sites in `init::run()` (fresh init at line 108, re-init at line 28) to pass detected plugins
- [x] 1.5 Add plugin detection to the re-init path (line 23-31 in init.rs) so it has access to detected plugins

## 2. Content: Build the new managed block

- [x] 2.1 Write the tool landscape section (conditional on companion tools being detected)
- [x] 2.2 Write the "When to Use What" decision table (conditional on beads or openspec)
- [x] 2.3 Write the unified "Starting a Session" section with conditional steps for beads/openspec
- [x] 2.4 Condense the wai core instructions (phases, artifact types) to fit alongside plugin sections
- [x] 2.5 Write the unified "Ending a Session" section with conditional steps for beads
- [x] 2.6 Write the multi-tool quick reference section with conditional beads/openspec subsections
- [x] 2.7 Keep the PARA structure explanation and "keep this managed block" footer

## 3. Verification

- [x] 3.1 `cargo build` compiles cleanly
- [x] 3.2 `cargo clippy` passes with no warnings
- [x] 3.3 `cargo test` passes all existing tests
- [x] 3.4 Manual test: `wai init` in empty dir produces wai-only AGENTS.md
- [x] 3.5 Manual test: `wai init` with `.beads/` + `openspec/` dirs produces plugin-aware AGENTS.md
- [x] 3.6 Manual test: re-run `wai init` preserves content outside WAI markers
- [x] 3.7 `wai doctor` still passes agent instructions check
