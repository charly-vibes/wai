## Phase 1: Session-close checklist step

- [ ] 1.1 In `src/managed_block.rs`, add a conditional `push_str` inside the
  "Ending a Session" block, after the beads steps and before `wai reflect`:
  ```rust
  if has_openspec {
      block.push_str(
          "[ ] openspec tasks.md — mark completed tasks [x]\n",
      );
  }
  ```
- [ ] 1.2 Unit test: `wai_block_content(&["openspec"])` → output contains
  `openspec tasks.md`
- [ ] 1.3 Unit test: `wai_block_content(&[])` → output does NOT contain
  `openspec tasks.md`
- [ ] 1.4 Unit test: ordering — `openspec tasks.md` line appears after
  `bd sync --from-main` (if beads present) and before `wai reflect`

## Phase 2: Cross-tool tracking section

- [ ] 2.1 In `src/managed_block.rs`, add a conditional section after "Capturing
  Work" and before "Ending a Session", gated on `has_beads && has_openspec`:
  ```rust
  if has_beads && has_openspec {
      block.push_str(
          "\n## Tracking Work Across Tools\n\
           \n\
           When beads and openspec are both active, keep them in sync:\n\
           - When creating a beads ticket for an openspec task, include the task\n\
           \x20 reference in the description (format: `<change-id>:<phase>.<task>`,\n\
           \x20 e.g. `add-why-command:7.1`)\n\
           - When closing a beads ticket linked to a task, also check the box\n\
           \x20 (`[x]`) in the change's `tasks.md`\n",
      );
  }
  ```
- [ ] 2.2 Unit test: `wai_block_content(&["beads", "openspec"])` → output
  contains "Tracking Work Across Tools"
- [ ] 2.3 Unit test: `wai_block_content(&["beads"])` → no "Tracking Work" section
- [ ] 2.4 Unit test: `wai_block_content(&["openspec"])` → no "Tracking Work"
  section
- [ ] 2.5 Unit test: section appears between "Capturing Work" and "Ending a
  Session" headings

## Phase 3: Propagate to this repo

- [ ] 3.1 Run `wai reflect` (or manually update the WAI:START block in
  `CLAUDE.md` and `AGENTS.md`) to include the new steps
- [ ] 3.2 Verify `CLAUDE.md` WAI block now contains both the openspec checklist
  step and the "Tracking Work Across Tools" section
- [ ] 3.3 Commit the updated `CLAUDE.md` and `AGENTS.md`
