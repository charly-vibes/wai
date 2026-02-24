## 1. `wai close` — write pending-resume signal
- [x] 1.1 After `create_handoff()` returns `Ok(handoff_path)`, compute the path
      relative to `project_dir` using `strip_prefix`; if strip_prefix fails
      (path is not under project_dir), skip writing the signal silently
- [x] 1.2 Write the relative path as UTF-8 text to
      `.wai/projects/<project>/.pending-resume`, overwriting if it exists

## 2. `wai prime` — resume detection helpers
- [x] 2.1 Add `read_pending_resume(project_dir: &Path) -> Option<PathBuf>`:
      reads `.pending-resume`, resolves path relative to `project_dir`,
      returns `None` if file missing, path invalid, or target file absent
- [x] 2.2 Add `extract_next_steps(handoff_path: &Path) -> Vec<String>`:
      reads file, finds `## Next Steps` heading, collects lines until the next
      `##` heading or EOF; skips blank lines and lines starting with `<!--`;
      returns items as-is

## 3. `wai prime` — resume rendering
- [x] 3.1 In `prime::run()`, before the current handoff-snippet logic, call
      `read_pending_resume()` and parse the resolved handoff's frontmatter `date`;
      if date parsing fails, treat as stale and fall through to normal output
- [x] 3.2 If date equals today: render `⚡ RESUMING: {date} — '{snippet}'` then
      call `extract_next_steps()` and print each item at four-space indentation
      under a `  Next Steps:` label (two-space indented); skip the normal
      `• Handoff:` line; do NOT modify or delete `.pending-resume`
- [x] 3.3 If stale or absent: fall through to existing `• Handoff:` logic
      unchanged

## 4. `src/managed_block.rs` — autonomous loop section
- [x] 4.1 In the `wai_block_content()` Rust string in `src/managed_block.rs`,
      find the `## Ending a Session` push_str block and append the Autonomous
      Loop subsection verbatim from `design.md § Autonomous Loop Content`;
      this section is added unconditionally (no plugin detection guard)

## 5. Refresh CLAUDE.md
- [x] 5.1 Run `wai init --yes` in the repo root to regenerate the WAI:START
      block in `CLAUDE.md` with the new Autonomous Loop section
      (manual step; not a code deliverable)

## 6. Tests
- [x] 6.1 Unit test: `close` writes `.pending-resume` with correct relative path
- [x] 6.2 Unit test: second `close` call overwrites `.pending-resume`
- [x] 6.3 Integration test: `prime` renders `⚡ RESUMING` block when
      `.pending-resume` present and handoff dated today; `prime` called twice
      in succession still shows RESUMING (signal not consumed)
- [x] 6.4 Integration test: `prime` renders normal `• Handoff:` line when
      `.pending-resume` handoff is dated yesterday (stale signal ignored)
- [x] 6.5 Integration test: `prime` renders normally when `.pending-resume`
      is absent (no regression on existing behaviour)
- [x] 6.6 Integration test: `prime` resume block with empty `## Next Steps`
      section (or only HTML comments) shows only the header line, no items
- [x] 6.7 End-to-end loop test: run `close` → verify `.pending-resume` written →
      run `prime` → verify RESUMING block with correct next steps → run `close`
      again → run `prime` → verify new RESUMING block reflects second handoff
