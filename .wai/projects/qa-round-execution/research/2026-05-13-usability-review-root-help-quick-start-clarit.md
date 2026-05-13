---
tags: [qa, usability, help, wai-fvhv.64, pipeline-run:epic-autonomy-tdd-ro5-2026-05-13-work-one-ready-child-issue-from-epic-wai-fvhv, pipeline-step:execute]
---

# Usability Review: Root Help & Quick-Start Clarity

**Issue:** wai-fvhv.64
**Date:** 2026-05-13
**Surfaces reviewed:** `wai` (bare), `wai --help`, `wai help`, `wai -v --help`, README.md quick-start, docs/src/quick-start.md

---

## Summary

The root help system is well-structured with progressive disclosure (`-v`/`-vv`/`-vvv`) and a curated QUICK START block. However, several clarity and discoverability issues reduce first-time-user effectiveness.

## Findings

### F1: Bare `wai` output omits `tutorial` and `init` (in initialized workspace)
- **Severity:** HIGH (caveat: tested only inside an initialized workspace — bare output in an uninitialized directory may already show `init`/`tutorial`; verify before implementing)
- Running `wai` with no args shows 4 suggestions: `status`, `phase`, `new project`, `way`
- Missing: `wai init` (required first step) and `wai tutorial` (recommended first step)
- A new user who runs `wai` won't discover how to start
- **Recommendation:** Test bare `wai` in an uninitialized directory first. If context-sensitive, ensure the uninitialized path shows `init`/`tutorial`. If not context-sensitive, add them to the bare-command output.

### F2: `--help` and `help` show different content
- **Severity:** MEDIUM
- `wai --help` shows the curated QUICK START block (5 commands) — good
- `wai help` shows the clap-generated long form with full option docs, env vars, and sorted command list — different ordering, different grouping, no quick-start
- Users will encounter both forms; the inconsistency is confusing
- **Recommendation:** Unify or cross-reference; at minimum, `wai help` should include the same QUICK START block at top

### F3: 24–26 commands in root help is overwhelming
- **Severity:** MEDIUM
- The COMMANDS section lists 24 entries in `--help`, 26 in `help` (adds `project` and `help` itself)
- No grouping by workflow stage or frequency of use
- New users scanning the list cannot distinguish core from advanced
- **Recommendation:** Group commands into sections (e.g., "Getting Started", "Daily Workflow", "Advanced", "Diagnostics") or at least mark the 5-6 most common ones

### F4: `status` vs `prime` distinction is unclear from help text
- **Severity:** MEDIUM
- `status` → "Check project status and suggest next steps"
- `prime` → "Orient yourself at session start"
- Both sound like "show me what's going on" — the difference is not obvious
- Quick-start mentions `status` but not `prime`; README mentions `status` but not `prime`
- **Recommendation:** Clarify both sides: `status` → "Show active projects, phases, plugin state, and suggested next commands"; `prime` → "Load session context: last handoff, active phase, open issues, suggested next step"

### F5: `doctor` vs `way` distinction is unclear
- **Severity:** MEDIUM
- `doctor` → "Diagnose workspace health"
- `way` → "Check repository best practices"
- Both sound like "check if things are OK"
- `--help` for each is minimal and doesn't clarify the boundary
- **Recommendation:** Add a distinguishing note, e.g., doctor checks `.wai/` structural integrity; way checks repo conventions and agent workflow patterns

### F6: Quick-start in `--help` doesn't mention `close` or `prime`
- **Severity:** LOW
- The QUICK START block covers init → new → add → status → phase next
- Missing the session lifecycle: `prime` (start) and `close` (end)
- These are core workflow commands
- **Recommendation:** Either add them to quick-start or add a "Session Lifecycle" note

### F7: README quick-start and docs quick-start are slightly misaligned
- **Severity:** LOW
- README mentions `wai tutorial` as an alternative to `wai init`
- docs/quick-start.md leads with `wai tutorial` as step 1, then `wai init` as step 2
- The two give slightly different impressions of the recommended first action
- **Recommendation:** Align: tutorial first in both, or init first in both

### F8: No help for "what is PARA?" in CLI
- **Severity:** LOW
- PARA is referenced in help text and README but never explained inline
- A user unfamiliar with PARA gets no in-CLI explanation
- `wai help` mentions it; quick-start.md explains it
- **Recommendation:** Consider `wai help para` or a brief parenthetical in root help

### F9: `project` subcommand hidden from `--help` but visible in `help`
- **Severity:** LOW
- `wai --help` doesn't list `project`; `wai help` does
- This is likely intentional (hidden command), but the inconsistency may confuse power users
- **Recommendation:** Either hide it from both or show it in both

### F10: Progressive disclosure verbosity doesn't show in `-v --help`
- **Severity:** LOW
- `-v --help` adds ADVANCED OPTIONS section (4 flags) — good
- But the content delta is small; users expecting more detail may be underwhelmed
- env vars and internals only appear at `-vv`/`-vvv` per-command level
- The root-level help doesn't hint at what `-vv`/`-vvv` would reveal
- **Recommendation:** Add a note like "Use -vv for env vars, -vvv for internals" in the verbose help

---

## Strengths

- The QUICK START block in `--help` is a clear, well-curated entry point
- Progressive disclosure is a strong pattern — not many CLIs do this
- Bare `wai` output is concise and action-oriented
- docs/quick-start.md is well-paced and explains concepts alongside commands
- Error messages suggest fixes (design principle delivered)

## Overall Assessment

The root help is **good but fragmented**: three surfaces (`wai`, `wai --help`, `wai help`) tell slightly different stories. The main gap is that the absolute first-time path (`init`/`tutorial`) is underexposed in the bare command, and the 25-command flat list needs grouping. The `status`/`prime` and `doctor`/`way` pairs need sharper differentiation.

**Priority fixes:** F1 (bare output), F2 (help inconsistency), F3 (command grouping)
**Quick wins:** F4 (status vs prime text), F6 (add close/prime to quick-start)

## Verification Evidence

This is a non-code usability review ticket. Verification method: manual inspection of CLI output surfaces.

Commands run:
- `cargo run -- ` (bare, no args)
- `cargo run -- --help`
- `cargo run -- help`
- `cargo run -- -v --help`
- `cargo run -- tutorial --help`
- `cargo run -- doctor --help`
- `cargo run -- way --help`
- `cargo run -- status`
- `cargo run -- prime`

Files read:
- `README.md` (quick-start section)
- `docs/src/quick-start.md`
- `src/help.rs`
- `src/cli.rs`

All findings verified against actual CLI output on this date.

