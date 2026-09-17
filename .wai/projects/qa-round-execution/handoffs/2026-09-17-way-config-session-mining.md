# Session Handoff — 2026-09-17: way configuration rounds (pi session mining)

## What Happened

User asked to "do rounds of work for wai way configuration — investigate all pi
sessions, see what worked and what can be improved."

1. **Mined all 30 pi sessions** (complete pi record, Apr 18–Sep 17 2026, ~4,400
   tool calls). Research doc:
   `.wai/projects/qa-round-execution/research/2026-09-17-pi-session-mining-way-configuration-what-worked-and-gaps.md`
   Key findings: the 2026-07-29 investigation doc was EMPTY (findings evaporated,
   work re-run 7 weeks later); user's July questions about prime-surfacing and
   staleness notification were never built; 294 tool errors (6.7%); Claude Code
   keeps harness-local memories invisible to pi agents (wai-c71i).
2. **Rule-of-5 review** (user requested) validated the research; fixed metric
   wording (close/prime = exposure not verified execution), added coverage
   section, widened stub-scan scope to all 5 artifact dirs, added wai-c71i.
3. **Implemented two tickets** on branch `feat/way-stub-artifact-check` (pushed):
   - `438b1f3` (wai-vdg6, closed): `wai way` "Project artifact completeness" check
     — scans research/handoffs/designs/plans/reviews for stub docs (<5 body lines)
     except structured records (UPPERCASE-KEY: convention). 7 tests.
   - `d02aff5` (wai-xt9o, closed): `wai prime` "Prior context" section (5 newest
     research/design/review docs, filename-date sorted, JSON parity) + fixed Plans
     rendering (was printing frontmatter `---`). 2 integration tests.
   All gates pass (fmt, clippy -D warnings, full test suite).

## Open Tickets Created

- wai-c71i (P3): surface Claude Code harness memories into .wai/beads
- wai-2am8 (P3): encode TDD pipeline in pipeline gates vs AGENTS.md prose
- wai-sojk (P2, blocked): now depends on `add-decision-freshness-tracking` epic —
  its `artifacts stale` command is the right mechanism; don't build a parallel one.

## Critical Files

- `src/commands/way/mod.rs` — check_artifact_stubs, is_structured_record,
  markdown_body_lines, ARTIFACT_SCAN_DIRS
- `src/commands/prime.rs` — read_prior_context, strip_frontmatter, PRIOR_CONTEXT_DIRS
- `tests/prime_test.rs` — 2 new integration tests

## Next Steps (priority order)

1. **Open PR** for `feat/way-stub-artifact-check` → main (2 commits, ~450 lines,
   under the 400-line limit per commit; PR body via /describe-pr skill)
2. **Execute `add-decision-freshness-tracking` epic** (17 tasks, openspec
   proposal ready at openspec/changes/add-decision-freshness-tracking/) —
   fulfills wai-sojk. Follow TDD→ro5u→fix→commit per ticket.
3. wai-2am8 (P3), wai-c71i (P3) after.

## Gotchas

- `.beads` is gitignored — `git add -f .beads/issues.jsonl` needed for sync commits
- `bd dolt push` FAILS: local/remote Dolt histories diverged (no common ancestor).
  Needs user decision: `bd dolt push --force` (keep local) or `bd bootstrap`
  (keep remote). NOT resolved — do not force-push without user approval.
- pretender emits ADVISORY complexity warnings on way/mod.rs and prime.rs —
  advisory only, pre-existing pattern in that file
- pi session JSONL format: `{"type":"message","message":{role,content[]}}`,
  toolResults carry `isError`; ~30 sessions in
  `~/.pi/agent/sessions/--var-home-sasha-para-areas-dev-gh-charly-wai--/`
