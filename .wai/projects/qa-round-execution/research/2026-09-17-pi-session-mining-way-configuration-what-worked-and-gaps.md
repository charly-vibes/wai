# Pi Session Mining for `wai way` Configuration — What Worked & Gaps

**Date:** 2026-09-17
**Method:** Mined all 30 pi sessions for this repo (`~/.pi/agent/sessions/--var-home-sasha-para-areas-dev-gh-charly-wai--/`, Apr 18 – Sep 17, 2026, ~30MB, ~4,400 tool calls).

## Coverage

- **30 sessions is the complete pi record.** Verified against all 68 project dirs in `~/.pi/agent/sessions/` (714MB): no session outside the wai dir references the wai cwd. Pi was installed ≈ 2026-04-18, which matches the first session date.
- **Cross-harness gap discovered:** Claude Code history for this repo (`~/.claude/projects/-...-wai/`) contains 1 real session (61 empty shells) plus a **harness-local memory dir** with user feedback ("always push after close"). This feedback is invisible to pi agents — instructions fragment across harness-local stores instead of living in `.wai/`.

## Dataset

| Metric | Value |
|---|---|
| Sessions | 30 |
| Tool calls | bash 2940 · read 892 · edit 487 · write 84 |
| Tool errors | 294 (6.7%) — bash 241, edit 49, read 4 |
| Error profile | "Command exited with code 1" (exploration), edit text-mismatch diffs, rg regex errors from piping ANSI-colored `wai` output into `rg` |

## What Worked

1. **Session lifecycle discipline is strong.** `wai close` / `wai prime` are *referenced* in 29/30 sessions (string mentions; skill instructions also echo these strings, so this measures exposure, not verified execution); handoff docs and research artifacts were used to resume work across weeks (May → Jul → Sep gaps).
2. **Autonomy from artifacts.** Terse user prompts ("work", "ok can you work on all") succeeded — agents recovered context from `.wai/` artifacts instead of requiring re-explanation.
3. **Low user-correction rate.** Only 1 explicit frustration message across 30 sessions ("why do I have to repeat tdd→ro5u→fix→commit every time"). Agents self-recovered from tool errors without abandonment.

## Gaps (way configuration opportunities)

1. **Findings evaporate (critical, self-demonstrating).** The 2026-07-29 research doc
   `2026-07-29-pipeline-utilization-investigation-findings-from.md` is **empty** — title
   only. The same session-mining investigation had to be re-run today, 7 weeks later.
   This is exactly the context-recovery cost wai exists to eliminate.
   → `wai way`/doctor needs a **stub/empty-artifact check** (frontmatter + <5 body lines).
2. **`wai prime` doesn't surface prior investigations.** User asked on 2026-07-29:
   "shouldn't this [investigation] come up on wai prime???" — never addressed. Prime
   should inject pointers to existing research docs relevant to the session topic.
3. **No staleness notification.** User asked on 2026-07-29: "do we have something that
   notifies that things are outdated??? so it can happen automatically" — nothing exists.
   Relates to open epic `add-decision-freshness-tracking` (0/17).
4. **Pipeline discipline lives in prose only.** TDD→ro5u→fix→commit repetition suggests
   it belongs in pipeline gates (epic-autonomy-tdd-ro5 has gates), not AGENTS.md reminders.
5. **Machine-unfriendly CLI output.** rg regex errors caused by parsing ANSI-colored
   `wai status` output — argues for stable `--json`/no-color output on more commands.
6. **Cross-harness memory fragmentation.** Claude Code stores repo-specific user feedback in its own memory dir (`~/.claude/projects/.../memory/`), unreachable from pi sessions. User feedback and corrections should live in `.wai/` (or beads) where any harness can see them.

## Recommended Rounds

| Round | Work | Type |
|---|---|---|
| 1 | `wai way` check: empty/stub research & handoff artifacts | Tool |
| 2 | `wai prime`: surface prior research/handoff docs for active project | Tool |
| 3 | Staleness notification (doctor/way), aligned with freshness-tracking epic | Tool |
| 4 | Encode per-ticket pipeline in pipeline gates; slim AGENTS.md prose | Repo/tool |

## Prior Art

- 2026-05-13 usability review (root help, doctor-vs-way distinction)
- genesis-adoption-plan.md
- Open ticket wai-fvhv.23: strengthen user-facing guidance for `wai way`
