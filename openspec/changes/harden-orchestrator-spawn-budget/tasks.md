## 1. Template edit (TDD: red → green)

- [x] 1.1 Write failing template-content test first: scaffold `epic-orchestrator` into
      a temp repo (mirror `pipeline_init_tdd_ro5_uses_autonomous_ro5u_template`), assert
      the scaffolded TOML contains: (a) `timeout <budget>` budget requirement in the
      spawn step, (b) `pi -c -p -n "<name>-resume"` resume exception, (c) a `git add .beads`
      prohibition in the ship step. Test must fail before the edit.
- [x] 1.2 Edit `src/templates/epic-orchestrator.toml`: spawn step gains the budget
      requirement + resume exception; verify step gains record-less-spawn refusal; ship
      step gains the `.beads` prohibition; header citation extended to canon invariants
      6, 8, 9. Make the test green.
- [x] 1.3 Run the full template test suite + `ah check --run-tests`; all green.

## 2. Trace

- [x] 2.1 `bd update wai-tdnv --append-notes "template layer: openspec change
      harden-orchestrator-spawn-budget applied"` and reference the change in the ticket.
