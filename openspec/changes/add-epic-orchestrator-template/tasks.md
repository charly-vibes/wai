## 1. Implementation (TDD: red → green)

- [x] 1.1 Write failing template test first — `epic-orchestrator` scaffold test mirroring
      `tests/suite_pipeline_init_tdd_ro5_ships_release_oracle_script.rs` and
      `tests/suite_project_use_fish_shell_syntax.rs::pipeline_init_tdd_ro5_uses_autonomous_ro5u_template`:
      run `wai pipeline init epic-orchestrator` in a temp repo, assert
      `.wai/resources/pipelines/epic-orchestrator.toml` exists with the orchestrator
      loop steps (claim → gates → brief → spawn → verify → ship → STOP) and the canon
      header citation. Test must fail before implementation.
- [x] 1.2 Add `src/templates/epic-orchestrator.toml` — orchestrator loop as ordered steps
      (claim, gates, brief, spawn, verify, ship, STOP); header cites canon
      `orchestrator-subagents.md` + Invariant 8; template metadata follows the tdd-ro5
      TOML shape (`name`, `description`, step ordering, skill/artifact declarations).
- [x] 1.3 Register in `BUILTIN_PIPELINE_TEMPLATES` (`src/config.rs`) — add
      `"epic-orchestrator"` to the const and update `BUILTIN_PIPELINE_TEMPLATES_HELP`;
      verify `wai pipeline init epic-orchestrator` scaffolds (setup.rs list-driven gate)
      and `wai pipeline list` offers it.
- [x] 1.4 Update template listing/help text — `src/cli/subcommands.rs`,
      `src/cli/mod.rs`, `src/help.rs` mention `epic-orchestrator` alongside `tdd-ro5`;
      update any dry-run/list test expectations. Green = template test passes,
      `cargo test` fully green.

## 2. Validation

- [x] 2.1 `openspec validate add-epic-orchestrator-template --strict` passes
- [ ] 2.2 `cargo test` fully green; pre-push hook passes

## 3. Distribution (wai-hr0w)

- [x] 3.1 Integration test `tests/suite_pipeline_epic_orchestrator_ticket_state.rs` —
      scaffolds the template in a temp repo, walks a simulated ticket through
      claim → gates → brief, asserts the per-ticket state-file shape
      (`.wai/projects/<project>/runs/<ticket>.state`: ticket id, branch, brief
      path, `step id + sha` step-history entries) plus the refusal conditions
      and single-model degradation declared by the template.
- [x] 3.2 Document the epic-orchestrator pipeline in `docs/src/concepts/pipelines.md`
      alongside the tdd-ro5 section: usage, loop steps, per-ticket state file
      (path, shape, retry/drift semantics), refusal conditions, single-model
      degradation, and the `tests-pass` oracle requirement.
- [x] 3.3 Distribution note linking provenance (`~/.wai/projects/orchestrator-tooling/`)
      in the docs section and this change's proposal.
