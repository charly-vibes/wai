# Workflow Protocols

Conventions for autonomous agent sessions working on this project.
Captures lessons from observed failure modes — each rule exists because something went wrong without it.

---

## 1. Phase Transition Quality Gate

**Rule:** Before claiming any issue from Phase N+1, `just check` must pass with zero errors.

**Why:** Pre-existing type errors and lint failures surface as confusing noise during feature implementation and block `just check` from being a reliable signal.

**Protocol for `impl:gather` Stage 1:**

1. After `bd ready`, before claiming any issue, run `just check`
2. If it fails → run `/issue:gather <describe the failures>` first, create fix tickets, implement those before feature work
3. Only claim feature work once `just check` is clean

---

## 2. Epic Lifecycle

**Rule:** Phase epics (type=feature, parent tickets) are closed by the orchestrator when the last sub-task closes. Epics must never appear in `bd ready`.

**Why:** `impl:gather` picks the highest-priority ready issue. An unclosed epic is indistinguishable from real work, causing the gather stage to claim it and waste a session orienting to already-done work.

**Protocol:**

- When closing the last sub-task of a phase, also close the phase epic in the same `bd close` call
- If an epic appears in `bd ready`: check whether all sub-tasks are closed, then close the epic — do not run `impl:run` on it
- Sub-task completion is the trigger; do not wait for a separate review pass

---

## 3. `impl:gather` Pre-Claim Checklist

Before claiming an issue (expand Stage 1 Step 2):

```
[ ] just check passes (0 errors) — if not, gather fix tickets first
[ ] Top result is not an epic — if it is, close it and re-run bd ready
[ ] Issue is not already implemented — read the files listed before claiming
```

---

## 4. Commit Discipline

- Close tickets before committing (so commit message can reference closure)
- One commit per logical unit — don't bundle mypy fixes with feature code
- Run `bd sync --from-main` after every commit session (even if it fails for missing remote — it's a no-op then)
