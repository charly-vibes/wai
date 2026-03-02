# Tasks: refactor-reflect-to-resource

## Phase 1: Storage and Config

- [x] 1.1 Add `REFLECTIONS_DIR = "reflections"` constant to `src/config.rs`
- [x] 1.2 Add `reflections_dir(project_root: &Path) -> PathBuf` helper to `src/config.rs`
  (returns `.wai/resources/reflections/`)
- [x] 1.3 Add `REFLECTIONS_DIR` to the dirs created by `ensure_workspace_current()`
  (alongside agent-config, pipelines, templates, patterns)

## Phase 2: Write to Resource File

- [ ] 2.1 Add `write_reflect_resource(project_root, project_name, content, handoff_count)`
  to `src/commands/reflect.rs` (NOT managed_block.rs — this writes a filesystem
  resource file, not a managed block; project_name is the dir name string already
  available in run())
- [ ] 2.2 Add suffix logic: if `<date>-<project>.md` already exists, try `-2`, `-3`, etc.
- [x] 2.3 Extend `ReflectContext` struct with `handoff_count: usize` (count of handoff
  files loaded); populate in `gather_reflect_context()`
- [ ] 2.3a Add YAML front-matter writer: `date`, `project`, `sessions_analyzed` (=
  handoff_count from 2.3), `type: reflection` — note: sessions_analyzed is for
  human inspection only; no wai command reads it back
- [x] 2.4 Update `run()` in `src/commands/reflect.rs` to call `write_reflect_resource()`
  instead of `inject_reflect_block()` — remove all writes to CLAUDE.md/AGENTS.md
- [x] 2.5 Update `--dry-run` path to show the resource file path that would be written
- [x] 2.6 Update success output: print the resource file path, not the target file

## Phase 3: Migration

- [x] 3.1 In `run()`, scan all target files (CLAUDE.md, AGENTS.md if present) for
  existing `WAI:REFLECT:START/END` blocks
- [x] 3.2 Apply unified migration rule when any old block is detected:
  - If no `*-migrated.md` exists in `.wai/resources/reflections/`: extract
    content from the first file that has the block, write to
    `<today>-<project>-migrated.md` with front-matter `type: reflection-migrated`
  - Replace `WAI:REFLECT:START/END` block in ALL target files that have it with
    the slim `WAI:REFLECT:REF:START/END` block
  - Print migration notice once regardless of how many files were migrated

## Phase 4: Managed Block Updates

- [x] 4.1 Add `WAI:REFLECT:REF:START/END` injection to `inject_managed_block()` in
  `src/managed_block.rs` — injects slim reference block after the `WAI:END` marker
- [x] 4.2 Add `wai_reflect_ref_content()` function that returns the slim reference block text
- [x] 4.3 Add search-before-research instruction to `wai_block_content()` in
  `src/managed_block.rs` — gated on `has_companions`, placed after TDD disclaimer
- [x] 4.4 No change needed at wai init call site — `inject_managed_block()` is already
  called by `wai init`; extending it internally (4.1) is sufficient
- [ ] 4.5 Update this repo's own `CLAUDE.md` by running `wai init` (or manually) after
  implementing 4.1–4.4

## Phase 5: Gather Context from Resource Files

- [ ] 5.1 Extend `gather_reflect_context()` to also read
  `.wai/resources/reflections/*.md` files for the current project as an additional
  low-priority context tier (after artifacts, before nothing)
- [ ] 5.2 Label this tier in the LLM prompt: "Previous reflections (extend and correct,
  do not repeat)"
- [ ] 5.3 Cap at ~20K chars (newest files first)

## Phase 6: Tests

- [ ] 6.1 Unit test: `write_reflect_resource()` writes to correct path with front-matter
- [ ] 6.2 Unit test: suffix logic (`-2`, `-3`) when file already exists
- [ ] 6.3 Unit test: `wai_reflect_ref_content()` contains "wai search" and the resource path
- [ ] 6.4 Unit test: search-before-research instruction present with companions, absent without
- [ ] 6.5 Unit test: `WAI:REFLECT:REF:START/END` block injected by `inject_managed_block()`
- [ ] 6.6 Integration test: migration path — old block detected → resource file created →
  old block replaced with REF block
- [ ] 6.7 `cargo check` passes with no warnings
- [ ] 6.8 `cargo test` passes

## Phase 7: Cleanup

- [ ] 7.1 Remove `inject_reflect_block()` from `src/managed_block.rs` (replaced in 2.1)
- [ ] 7.2 Remove `read_reflect_block()` — no longer needed for injection (keep if used
  by gather_reflect_context; adapt as needed)
- [ ] 7.3 Update `docs/src/commands.md` — reflect section now describes resource output
- [ ] 7.4 Update `README.md` if it mentions the REFLECT block injection behavior
- [ ] 7.5 Run `wai init` on this repo to inject `WAI:REFLECT:REF` block into
  CLAUDE.md/AGENTS.md, then run `wai reflect` to migrate the existing
  `WAI:REFLECT` block (committed earlier this session) to
  `.wai/resources/reflections/`
