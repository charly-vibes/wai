## Phase 1: Session-close checklist step

- [x] 1.1 In `src/managed_block.rs`, add a conditional `push_str` inside the
  "Ending a Session" block, after the beads steps and before `wai reflect`:
  ```rust
  if has_openspec {
      block.push_str(
          "[ ] openspec tasks.md — mark completed tasks [x]\n",
      );
  }
  ```
- [x] 1.2 Unit test: `wai_block_content(&["openspec"])` → output contains
  `openspec tasks.md`
- [x] 1.3 Unit test: `wai_block_content(&[])` → output does NOT contain
  `openspec tasks.md`
- [x] 1.4 Unit test: ordering — `openspec tasks.md` line appears after
  `bd sync --from-main` (if beads present) and before `wai reflect`

## Phase 2: Cross-tool tracking section

- [x] 2.1 In `src/managed_block.rs`, add a conditional section after "Capturing
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
- [x] 2.2 Unit test: `wai_block_content(&["beads", "openspec"])` → output
  contains "Tracking Work Across Tools"
- [x] 2.3 Unit test: `wai_block_content(&["beads"])` → no "Tracking Work" section
- [x] 2.4 Unit test: `wai_block_content(&["openspec"])` → no "Tracking Work"
  section
- [x] 2.5 Unit test: section appears between "Capturing Work" and "Ending a
  Session" headings

## Phase 3: Pre-claim implementation check

- [x] 3.1 In `src/managed_block.rs`, add a note after the `bd ready` step in
  "Starting a Session", gated on `has_beads`:
  ```rust
  if has_beads {
      block.push_str(
          "   Before claiming: read the relevant source files to confirm\n\
           \x20  the issue is not already implemented.\n",
      );
  }
  ```
- [x] 3.2 Unit test: `wai_block_content(&["beads"])` → output contains
  "already implemented" near the `bd ready` line
- [x] 3.3 Unit test: `wai_block_content(&[])` → no "already implemented" text

## Phase 4: Epic closure reminder

- [x] 4.1 In `src/managed_block.rs`, update the `bd close <id>` checklist line
  (already gated on `has_beads`) to add a trailing comment:
  ```rust
  "[ ] bd close <id>                  # close completed issues; also close parent epic if last sub-task\n"
  ```
- [x] 4.2 Unit test: `wai_block_content(&["beads"])` → `bd close` line contains
  "epic" or "parent"
- [x] 4.3 Unit test: `wai_block_content(&[])` → no `bd close` line at all

## Phase 5: Propagate to this repo

- [x] 5.1 Run `wai reflect` (or manually update the WAI:START block in
  `CLAUDE.md` and `AGENTS.md`) to include all new steps
- [x] 5.2 Verify `CLAUDE.md` WAI block contains: openspec checklist step,
  "Tracking Work Across Tools" section, pre-claim note, and epic reminder
- [x] 5.3 Commit the updated `CLAUDE.md` and `AGENTS.md`
