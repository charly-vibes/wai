# Implementation Tasks

**CRITICAL: Use TDD (Test-Driven Development) and Tidy First principles for all
development tasks below.**

**Note**: When creating beads tickets from these tasks, add "CRITICAL: Use TDD
and Tidy First" at the bottom of each development ticket.

**Prerequisite**: Requires `src/llm.rs` merged (already implemented in
`add-why-command`; spec archiving not required).

## Phase 1: Infrastructure

- [ ] 1.1 Add `Reflect` subcommand to `src/cli.rs` with flags:
        `--project <name>`, `--conversation <file>`, `--output <target>`,
        `--dry-run`, `--yes`
        (target: "claude.md" | "agents.md" | "both"; default: auto-detect)
- [ ] 1.2 Create stub `src/commands/reflect.rs` that dispatches to unimplemented
        handler (returns Ok for now)
- [ ] 1.3 Register reflect module in `src/commands/mod.rs` and `src/main.rs`
        dispatch
- [ ] 1.4 Add REFLECT marker support to `src/managed_block.rs`:
        `has_reflect_block()`, `inject_reflect_block()`, `read_reflect_block()`
        for both CLAUDE.md and AGENTS.md
- [ ] 1.5 Create `.reflect-meta` read/write helpers in `src/commands/reflect.rs`:
        TOML with `last_reflected` (date string) and `session_count` (u32)

## Phase 2: Context Gathering

- [ ] 2.1 Implement conversation transcript reader: `--conversation <file>` reads
        plain text, capped at ~30K chars truncated from the top (oldest removed)
- [ ] 2.2 Implement handoff artifact reader — reads all
        `.wai/projects/*/handoffs/*.md` sorted by mtime (newest first), up to
        ~40K chars
- [ ] 2.3 Implement secondary artifact reader for research, design, plan files
        (reuse or extract the pattern from `src/commands/why.rs`), up to ~30K chars
        from remaining budget
- [ ] 2.4 Implement output target detection: scan repo root for CLAUDE.md and
        AGENTS.md; apply `--output` override; fail clearly if neither found
- [ ] 2.5 Read existing REFLECT block(s) from target file(s) to pass to LLM as
        "previously documented" context

## Phase 3: LLM Integration

- [ ] 3.1 Reuse `WhyConfig` and `detect_backend()` from `src/llm.rs` — no new
        config section required
- [ ] 3.2 Build reflection prompt:
        - Role: "You are synthesizing project-specific AI assistant guidance"
        - Input hierarchy explanation (conversation > handoffs > artifacts)
        - Existing REFLECT block content (if any): "already documented, do not repeat"
        - Conversation transcript (if provided), handoff files, other artifacts
          (each labeled with type and date)
        - Instruction to note artifact date when referencing patterns; flag items
          from artifacts older than 6 months as potentially stale
        - Output format: a WAI:REFLECT block in the format shown in design.md
          (LLM chooses which sections to include based on what it found)
        - Escape triple-backticks in all artifact content
- [ ] 3.3 Call LLM backend and capture response string
- [ ] 3.4 Extract the REFLECT block content from the LLM response (strip outer
        markdown fences if present; keep or add marker lines)

## Phase 4: Output and Confirmation

- [ ] 4.1 Compute unified diff between existing REFLECT block content (if any) and
        proposed new content; display with +/- markers
- [ ] 4.2 `--dry-run`: print diff and exit 0 without prompting or writing
- [ ] 4.3 If diff is empty (no changes): print "Reflect block is already up to date"
        and exit 0
- [ ] 4.4 Prompt user for confirmation using cliclack `confirm` widget showing
        which files will be written (dynamic: "Write to CLAUDE.md and AGENTS.md?")
- [ ] 4.5 `--yes` flag: skip confirmation prompt, write directly
- [ ] 4.6 On confirm: inject REFLECT block into each target file using
        `inject_reflect_block()` (create if absent, replace if exists)
- [ ] 4.7 Write updated `.reflect-meta` with current date and session count
- [ ] 4.8 Print success message with list of updated files and line counts

## Phase 5: Session Integration

- [ ] 5.1 In `src/commands/close.rs`: after creating handoff, read
        `.reflect-meta` for the current project (default last_reflected = epoch
        if file absent)
- [ ] 5.2 Count handoff files whose mtime is newer than `last_reflected`
- [ ] 5.3 If count ≥ 5 and not `--quiet`: append nudge suggestion:
        `→ N sessions since last reflect — run 'wai reflect' to update CLAUDE.md`
        (or AGENTS.md / both, matching what exists in the repo root)
- [ ] 5.4 Nudge is informational only — does NOT block close or change exit code

## Phase 6: Handoff Template Improvement

- [ ] 6.1 Extend the handoff template in `src/commands/handoff.rs` (or wherever
        it is defined) to include two new sections:
        `## Gotchas & Surprises` and `## What Took Longer Than Expected`
        with HTML comment prompts inside each section
- [ ] 6.2 Update the handoff-system spec delta to record this change
        (add delta in `specs/handoff-system/spec.md` under this change)

## Phase 7: Testing

- [ ] 7.1 Unit tests for `has_reflect_block()` and `inject_reflect_block()` —
        create, update, preserve surrounding content, CLAUDE.md and AGENTS.md
- [ ] 7.2 Unit tests for `.reflect-meta` read/write (missing file, valid file,
        corrupted file)
- [ ] 7.3 Unit tests for output target detection (only CLAUDE.md, only AGENTS.md,
        both, neither)
- [ ] 7.4 Unit tests for conversation transcript reader (missing file, empty,
        truncation at 30K chars)
- [ ] 7.5 Unit tests for context budget (handoff priority, dynamic allocation,
        truncation)
- [ ] 7.6 Unit tests for reflection prompt construction (input hierarchy labeling,
        escaping, existing block read-back)
- [ ] 7.7 Integration test with mock LLM: fixed response → verify CLAUDE.md update
        and .reflect-meta write
- [ ] 7.8 Integration test: `--dry-run` does NOT modify any file
- [ ] 7.9 Integration test: diff is shown when REFLECT block already exists
- [ ] 7.10 Integration test: empty diff → no write, "already up to date" message
- [ ] 7.11 Integration test: wai close nudge fires when 5+ handoffs newer than
        `.reflect-meta` date
- [ ] 7.12 Manual test: run `wai reflect` against this repo's actual handoffs and
        verify output quality

## Phase 8: Documentation

- [ ] 8.1 Update `managed_block.rs` `wai_block_content()` to add `wai reflect`
        to the "Ending a Session" checklist and Quick Reference section
- [ ] 8.2 Add `--help` text: example usage, conversation flag, output flag;
        note that LLM config is shared with `wai why`
- [ ] 8.3 Add `wai reflect` suggestion to `wai way` best-practices checks
        ("AI config has no REFLECT block after N+ handoffs")
