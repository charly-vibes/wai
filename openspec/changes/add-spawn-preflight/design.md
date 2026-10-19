# Design: add-spawn-preflight

## Command vs justfile recipe

The preflight is a `wai pipeline preflight` subcommand, not a justfile recipe:
it must be testable in Rust (regression test in `tests/`), executable from any
spawned worktree without repo-specific just targets, and exit-code structured
for the template to gate on. A justfile recipe would be per-repo boilerplate in
~50 repos — the wrong layer.

## Check set (initial)

1. **lefthook executable present** — the week's data shows `npx lefthook run
   pre-commit` failing exit 2 with no output when lefthook is not installed;
   the check is `lefthook version` (or binary presence) with a message pointing
   at the install path.
2. **`bd list` exits 0** — catches broken beads state / wrong-project invocation
   (11 wrong-ID-prefix failures observed).
3. **testaruda store initialized** — `testaruda` store present or auto-init on
   first use (the DDL test-runner standard already specifies first-use init;
   preflight surfaces a clear message if init fails rather than failing
   mid-run on a stale store).

Each check is independent: report ALL failures in one pass (one message per
missing tool), not fail-on-first — a spawn should not need three runs to learn
about three problems.

## Extensibility

The check list is data, not control flow: adding a future check (e.g. gate
tooling versions) is one branch in the check set. No plugin mechanism now —
minimal implementation first.

## What preflight is not

- No auto-install, no network calls (fail-fast only, per proposal).
- Not a gate tier: it runs at spawn, before any step advances; it does not
  participate in `pipeline next` gate evaluation.
