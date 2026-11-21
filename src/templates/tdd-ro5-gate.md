---
description: Route production-code edits through the tdd-ro5 TDD pipeline
scope: tool:edit(*.rs), tool:edit(*.py), tool:edit(*.go), tool:edit(*.ts), tool:edit(*.tsx), tool:edit(*.js), tool:edit(*.mjs), tool:edit(*.c), tool:edit(*.h), tool:edit(*.cc), tool:edit(*.cpp), tool:edit(*.hpp), tool:edit(*.ex), tool:edit(*.exs), tool:edit(*.nu), tool:edit(*.sh), tool:write(*.rs), tool:write(*.py), tool:write(*.go), tool:write(*.ts), tool:write(*.tsx), tool:write(*.js), tool:write(*.mjs), tool:write(*.c), tool:write(*.h), tool:write(*.cc), tool:write(*.cpp), tool:write(*.hpp), tool:write(*.ex), tool:write(*.exs), tool:write(*.nu), tool:write(*.sh)
condition: ".*"
interruptMode: never
repeatMode: after-gap
---
Production code changed — verify pipeline state. tdd-ro5 gate:

1. Run `wai pipeline current --json`. If a run is active, continue it (`wai pipeline next`) and record this edit in the current step.
2. If no run is active and this is a real implementation task (feature, bugfix, refactor), start one: `wai pipeline start tdd-ro5 --topic="<task>"`. RED comes first: write the failing test before production edits.
3. Only skip for a trivial typo/format change — state the skip in your reply in one line.

Note: docs and config files are exempt, but this rule also fires on test-file edits — if you are writing the failing test first (RED) inside an active run, this reminder is expected; proceed.
