# Project status brief

## Purpose / current state

none

## Focus

none

## In flight

- wai-0awg — Invalidate hook approval when repository identity or script contents change (status: open; priority: 1; age: 40d)
- wai-4rn7 — Triage ah check: 426 structural spec-test findings (381 no-tests-declared, 39 no-toml, 4 orphan-toml, 2 slug-collision) (status: open; priority: 1; age: 7d)
- wai-4wp8 — Research: Hegel property-based testing plan (antithesis-research) (status: open; priority: 1; age: 2d)
- wai-fped — CI red on main: clippy -D warnings — 6 errors (items_after_test_module + 5 pre-existing) (status: open; priority: 1; age: 7d)
- wai-fvhv — QA round: 100+ wai CLI findings across usability, docs, scope, code quality, and test coverage (status: open; priority: 1; age: 189d)
- wai-fvhv.84 — Scope: Define product boundary between workflow management and repository-hygiene auditing (status: open; priority: 1; age: 189d)
- wai-fvhv.93 — Scope: Evaluate plugin-system scope against maintenance budget (status: open; priority: 1; age: 189d)
- wai-fvhv.94 — Scope: Clarify tutorial target audience and success criteria (status: open; priority: 1; age: 189d)
- wai-g2iv — lefthook spk-lint gate calls wrong linter for openspec tree (status: open; priority: 1; age: 7d)
- wai-jzhn — Implement Hegel property tests (status: open; priority: 1; age: 2d; blocked by: wai-4wp8)
- wai-y3p7 — docs: apply README front-door standard v0.4 (§5.1) (status: open; priority: 1; age: 2d)
- wai-0apx — fold orchestrator-subagents pattern doc projection to reference wai pipeline spawn wrapper flags (wai-tdnv task 3.5) (status: open; priority: 2; age: 30d)
- wai-0b95 — wai add: error messages must state the fix (filename vs path, missing --reviews) (status: open; priority: 2; age: 40d)
- wai-0v2e — investigate: pretender gate (--mode gate) is stateful — passes with cmd_init red staged, blocks with command_help red staged, flaps across runs with identical output (status: open; priority: 2; age: 41d)
- wai-1s8i — ddl doctor repo.gates false-pass: lefthook.yml checked but hooks activation never verified (status: open; priority: 2; age: 7d)
- wai-42ig — Write ADR: command taxonomy and admission criteria (status: open; priority: 2; age: 187d)
- wai-4rn7.1 — Contract-bind spec: cli-core (63 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.10 — Contract-bind spec: managed-block (15 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.11 — Contract-bind spec: project-state-machine (13 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.12 — Contract-bind spec: plugin-system (12 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.13 — Contract-bind spec: orchestration-integrity (10 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.14 — Contract-bind spec: error-recovery (9 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.15 — Contract-bind spec: help-system (9 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.16 — Contract-bind spec: handoff-system (8 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.17 — Contract-bind spec: skill-sharing (8 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.18 — Contract-bind spec: timeline-search (8 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.19 — Contract-bind spec: para-structure (7 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.2 — Contract-bind spec: pipeline-resource (55 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.20 — Contract-bind spec: onboarding (6 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.21 — Contract-bind spec: agnostic-capabilities (4 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.3 — Contract-bind spec: repository-best-practices (44 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.4 — Contract-bind spec: reasoning-oracle (34 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.5 — Contract-bind spec: project-reflection (31 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.6 — Contract-bind spec: agent-config-sync (26 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.7 — Contract-bind spec: context-suggestions (23 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.8 — Contract-bind spec: decision-matrix (20 findings → 0) (status: open; priority: 2; age: 7d)
- wai-4rn7.9 — Contract-bind spec: doctor-auto-fix (17 findings → 0) (status: open; priority: 2; age: 7d)
- wai-80x5 — refactor(src/help.rs): split command_help (598 lines) and render_main_help (86 lines, abc 60) under function_lines_max=60 (status: open; priority: 2; age: 41d)
- wai-bvn7 — way: detect Nix (flake.nix/devenv.nix) in the Reproducible environments check (status: open; priority: 2; age: 41d)
- wai-ekwq — Detail docs IA restructuring work (status: open; priority: 2; age: 187d)
- wai-hfye — openspec 2.1.0 validator rejects wai's specodelic-native corpus (9 items) (status: open; priority: 2; age: 0d)
- wai-of8s — Flaky test: gather_git_file_context_returns_history_for_tracked_file fails intermittently in full cargo test runs (passed 5/5 in isolation and in 2 consecutive full-suite reruns). Git-race/env-dependent in src/commands/why/context.rs — needs investigation (git config/env leakage between concurrent tests suspected). Untouched by wai-lqdr. (status: open; priority: 2; age: 41d)
- wai-opk8 — pipeline templates: adopt shared-block mechanism for remaining convention boilerplate (status: open; priority: 2; age: 41d)
- wai-sojk — wai way/doctor: staleness notification for .wai artifacts (status: open; priority: 2; age: 62d)
- wai-t3qa — wai pipeline next exits 0 when a gate blocks advancement (status: open; priority: 2; age: 41d)
- wai-wra0 — Design: absorb sync into doctor --fix (status: open; priority: 2; age: 187d)
- wai-yp2l — Failure census over real run history before any journal/receipt protocol work (dr-workflow-execution-verdict decisive experiment) (status: open; priority: 2; age: 29d)
- wai-35zh — docs: v2026.10.3 versioned snapshot missing — tag ran broken docs.yml before fixes (status: open; priority: 3; age: 46d)
- wai-4z3q — Add --dry-run flag to wai import (status: open; priority: 3; age: 187d)
- wai-5w57 — wai add still stamps never-advanced abandoned runs from .last-run pointer (status: open; priority: 3; age: 35d)
- wai-9gzj — Docs workflow fails on tag pushes (pre-existing since v2026.10.21) (status: open; priority: 3; age: 7d)
- wai-anez — Strengthen tutorial exit CTA (status: open; priority: 3; age: 187d)
- wai-c71i — Surface harness-local memories (e.g. Claude Code memory dir) into .wai or beads (status: open; priority: 3; age: 62d)
- wai-d78z — openspec 1.x upgrade blocked: 18 validation failures under @fission-ai/openspec@1.14.2 (status: open; priority: 3; age: 33d)
- wai-db7n — pipeline spawn checkpoint save is non-atomic (ro5u L3) (status: open; priority: 3; age: 30d)
- wai-fvhv.100 — Scope: Audit examples and docs for Pi-specific scope drift (status: open; priority: 3; age: 189d)
- wai-fvhv.110 — Docs: strengthen guidance for `wai plugin` management and passthrough behavior (status: open; priority: 3; age: 189d; blocked by: wai-fvhv.76, wai-fvhv.93)
- wai-fvhv.22 — Docs: strengthen user-facing guidance for `wai tutorial` (status: open; priority: 3; age: 189d; blocked by: wai-fvhv.94)
- wai-fvhv.55 — Code quality: modularize src/managed_block.rs (status: open; priority: 3; age: 189d)
- wai-fvhv.56 — Code quality: modularize src/cli.rs (status: open; priority: 3; age: 189d)
- wai-fvhv.57 — Code quality: modularize src/sync_core.rs (status: open; priority: 3; age: 189d)
- wai-fvhv.58 — Code quality: modularize src/llm.rs (status: open; priority: 3; age: 189d)
- wai-fvhv.60 — Code quality: modularize src/help.rs (status: open; priority: 3; age: 189d)
- wai-fvhv.61 — Code quality: modularize src/workflows.rs (status: open; priority: 3; age: 189d)
- wai-fvhv.62 — Code quality: modularize src/plugin.rs (status: open; priority: 3; age: 189d)
- wai-fvhv.63 — Code quality: modularize src/suggestions.rs (status: open; priority: 3; age: 189d)
- wai-fvhv.67 — Usability review: Review `add` command discoverability across artifact types (status: open; priority: 3; age: 189d)
- wai-fvhv.68 — Usability review: Review `resource` namespace cognitive load (status: open; priority: 3; age: 189d)
- wai-fvhv.71 — Usability review: Improve error-message guidance for outside-workspace usage (status: open; priority: 3; age: 189d)
- wai-fvhv.75 — Usability review: Audit `ls` workspace discovery ergonomics (status: open; priority: 3; age: 189d)
- wai-fvhv.76 — Usability review: Review plugin passthrough UX (status: open; priority: 3; age: 189d)
- wai-fvhv.78 — Usability review: Improve handoff discoverability (status: open; priority: 3; age: 189d)
- wai-fvhv.8 — Docs: strengthen user-facing guidance for `wai ls` (status: open; priority: 3; age: 189d; blocked by: wai-fvhv.75)
- wai-fvhv.80 — Usability review: Clarify move/category semantics (status: open; priority: 3; age: 189d)
- wai-fvhv.82 — Usability review: Audit verbosity ladder consistency (status: open; priority: 3; age: 189d)
- wai-fvhv.85 — Scope: Define product boundary between session continuity features and general project management (status: open; priority: 3; age: 189d)
- wai-fvhv.87 — Scope: Decide whether the Pi extension package is a core product concern or separate integration (status: open; priority: 3; age: 189d)
- wai-fvhv.90 — Scope: Create a feature map tying commands to primary user journeys (status: open; priority: 3; age: 189d)
- wai-fvhv.95 — Scope: Write a deprecation policy for commands, flags, and output formats (status: open; priority: 3; age: 189d)
- wai-fvhv.98 — Scope: Define boundaries for global config vs workspace-local config (status: open; priority: 3; age: 189d)
- wai-fvhv.99 — Scope: Clarify the contract between `.wai/` artifacts and external tools like beads/openspec (status: open; priority: 3; age: 189d)
- wai-hpn1 — pretender: tighten tiered advisory lease — way/ debt fully greened, lease may be removable (status: open; priority: 3; age: 46d)
- wai-z25x — flaky test: execute_hook_no_deadlock_on_fast_command fails under parallel load (status: open; priority: 3; age: 54d)

## Blocked / waiting

- wai-jzhn — Implement Hegel property tests (status: open; priority: 1; age: 2d; blocked by: wai-4wp8)
- wai-fvhv.110 — Docs: strengthen guidance for `wai plugin` management and passthrough behavior (status: open; priority: 3; age: 189d; blocked by: wai-fvhv.76, wai-fvhv.93)
- wai-fvhv.22 — Docs: strengthen user-facing guidance for `wai tutorial` (status: open; priority: 3; age: 189d; blocked by: wai-fvhv.94)
- wai-fvhv.8 — Docs: strengthen user-facing guidance for `wai ls` (status: open; priority: 3; age: 189d; blocked by: wai-fvhv.75)

## Next / ready

- wai-0awg — Invalidate hook approval when repository identity or script contents change (status: open; priority: 1; age: 40d)
- wai-4rn7 — Triage ah check: 426 structural spec-test findings (381 no-tests-declared, 39 no-toml, 4 orphan-toml, 2 slug-collision) (status: open; priority: 1; age: 7d)
- wai-4wp8 — Research: Hegel property-based testing plan (antithesis-research) (status: open; priority: 1; age: 2d)
- wai-fped — CI red on main: clippy -D warnings — 6 errors (items_after_test_module + 5 pre-existing) (status: open; priority: 1; age: 7d)
- wai-fvhv — QA round: 100+ wai CLI findings across usability, docs, scope, code quality, and test coverage (status: open; priority: 1; age: 189d)
(74 more omitted)

## Parked

none

### Needs re-triage

none

## Recent

- 9d95cd5e761feeed887f538e20f0c9c4fb2a9416 chore(gates): render the managed gate blocks from .ddl/config.toml (2026-11-18T16:03:39Z)
- 21b4e0b4b2ef7d1d6a0bf3b65482f97619dad37d spec: make the corpus valid under openspec 2.1.0 (2026-11-18T15:46:50Z)
- 632aab2da537a86a95f5f735a73dc3e644c93cb8 chore(skills): retire the pre-2.1 openspec workflows (2026-11-18T15:21:20Z)
- 936512c7898aea0f5908bdf7b665524bd9975c1a chore(openspec): migrate project.md into config.yaml; pin 2.1.0 (2026-11-18T15:03:22Z)
- 9c4422b054a08f53ceb8d637e464d4e828178934 chore(openspec): adopt the 2.1 artifact layout (2026-11-18T15:03:14Z)
(195 more omitted)

## Health + confidence

Health: **amber** — basis: signals — owner: none — reason: orphan_commits: 68 commits in the 14-day window link no tracker id or open work-item path: 01b44d833622a923674404235932be4fa7663b7a (release: 2026.11.17 — layout-aware openspec pointer; green pre-commit); 107e67156ab23b9392cda474894c386a9db77201 (bd: file baseline-drift bug (pre-existing test failure)); 1c5ded10bada1e91cd6f389f887e3b293ebfffd3 (refactor(tests): table-drive way_check and help scenarios to clear lazy_cluster in doctor suite); 1dd83898ae9f8853317437942bcbbe0fb9dbbf9c (refactor(plugin): split plugin.rs into plugin/ submodules); 21b4e0b4b2ef7d1d6a0bf3b65482f97619dad37d (spec: make the corpus valid under openspec 2.1.0); 224096f91f8e95fd4fdf0ef7ea66ae0e8aee181b (refactor(src): split detect_patterns into per-pattern push helpers); 2428e0c955a4dd1a5f1a675368dd50e39303796b (refactor(tests): flatten walk dir traversal to reduce nesting); 2a8317b0c6770004d62727e76e2c2f7e7d84b9ba (chore(scripts): drop unused SESSIONS_DIR, inline token comment); 46ce48de86177a0804600f95d8433fdad57e5acc (docs: file README front-door and release tickets); 493abc501cf51f3195f15254c3b5ead65bfe61ab (refactor(src): decompose doctor checks_sync strategy checks; reduce mutable bindings); 5a9a1358350df5ba392158066fba7315c5192321 (fix(pipeline): terminate the bail macro so the newer-stable lint passes); 5f7c4921e0599ecfda7e6d32c494304361857bec (refactor(src): split ls run and discover_workspaces into helpers); 632aab2da537a86a95f5f735a73dc3e644c93cb8 (chore(skills): retire the pre-2.1 openspec workflows); 65f60c58d7a06c120f5950a3e8f46d0fdbfb48a3 (refactor(tests): dedupe matrix/plugin-list/tdd-ro5 lazy test clusters via tables and helpers); 6986cc6e20f7e6cd7c0843033d7f24a9c967ef2f (refactor(src): data-driven help tables; split render_main_help sections); 6d3c95609eb1b832b91cabc7e5879f2c1b6bb43d (refactor(tests): split epic-orchestrator walk test into phase helpers); 70b8f6cb4298a367b84692182c567e5397c3c70f (chore: remove superseded matrix.rs/plugin.rs/sync_core.rs; fmt touch-ups); 78b47385f34c81370f6182a688b0d36f8ab79a77 (refactor(src): decompose pipeline queries into per-phase helpers); 7f32379417f5fe15932016dc7c8178840b92e733 (refactor(tests): vary pipeline TOML helper structure to break lazy_cluster in common); 80ad7b0f983ab4233c5061c2c11722fd358f7dc3 (refactor(init): extract per-path handlers from run() dispatcher); 85b67dfcc714e01ae1f5e2d5708a9fb66055072b (refactor(scripts): split session-friction main into select/fetch/signals/aggregate helpers); 86e05f91aa82c465745153dd73cb0576376e18af (docs: apply issue-review fixes to README/release tickets); 86ea659d8b83869d7157f4d0b15ad0cf199e25f5 (refactor(src): split build_prompt into sections; remove mutable bindings in why/prompt.rs); 8845c882acb38d8ac26e286cf59779cd7364b442 (style: cargo fmt across pipeline/add modules and test suites); 936512c7898aea0f5908bdf7b665524bd9975c1a (chore(openspec): migrate project.md into config.yaml; pin 2.1.0); 93c282fbba95f2fa729697eed37b3374e1a9a4f0 (refactor(src): split close run into summary/remember helpers; tidy ls); 990cb7972e1426c21afe8e1e7b83059b2895fede (test: assert claim sha in walk test body for min_assertions); 9b9183d601dc2d9b2d090c512885292fd6773d14 (refactor(src): extract external-command dispatch helper in commands/mod.rs); 9c4422b054a08f53ceb8d637e464d4e828178934 (chore(openspec): adopt the 2.1 artifact layout); 9d6d6722e37e357d2ebe2cc1f75aef1dcabe3f5b (refactor(pipeline): extract model.rs and verify.rs from mod.rs; split tests into files); 9d95cd5e761feeed887f538e20f0c9c4fb2a9416 (chore(gates): render the managed gate blocks from .ddl/config.toml); a170021b433a504145701eaf1b866cd07a39fb55 (docs(brief): pretender debt fix brief (wai) — orchestrator artifact); a2776313bc4ee7f241b4e7fd0ae239a21e0aada5 (refactor(pipeline): split orchestration.rs into orchestration/ submodules); a31e6e83cc871388848b3aff0f87121e45dffa45 (refactor(src): split timeline run into collect/sort/print helpers); a38f071a820b67f1206284541ac61cd0642d24cb (refactor(sync): extract projection execution and from-main sync helpers); a53d51b31d07b5584c90b5b0f9fab99cd798795f (test(archive): extract make_archive/import_archive helpers from round-trip tests); a91c8167eb3082c79cf8c6cd608c2239b939e6ce (refactor(add): split add.rs into add/ module dir); ab26fa45d171e15b8235fcc49ff48ff44b389f63 (refactor(src): extract shared walk/budget helpers in reflect/context.rs); ad59b564212461a2600265bfe5a0d4e429749868 (fix(tests): replace unwrap/expect with panic-context alternatives); b035e7f10afb01be139b4bae2a170f8f241f7409 (refactor(plugin): extract per-subcommand handlers from run()); b600d07d286c30caf5cc3efbac69a24d405a5920 (refactor(tests): merge near-duplicate matrix lint tests below file_lines limit); b65e523f88e10c6feb1862fed8fb8feb9d3da847 (refactor(sync): flatten claude_code_needs_sync into iterator chains); b8b3892b6074129078164a7707f300edcb756a6a (refactor(src): extract plugin-context and content-template helpers in handoff.rs); bc2fad24b8feaa25f9d9523264c1111f17b99e03 (refactor(src): split tutorial run into step-runner and summary helpers); bd4a02bac28b998507b10b6ee5e0d4f49fb4a122 (refactor(src): decompose doctor checks_basic into per-concern helpers); be652db3db6a32ec6fac6d6f29728d81bb00a978 (style: cargo fmt freshness.rs); bf8181d7793dc080193ed70068351f440a207e0e (refactor(matrix): split matrix.rs into matrix/ submodules); c224ad2c6688aae47133c20a23362a7976903dea (refactor(tests): extract shared script/pipeline-file helpers in common); c269c677a280648236297881de1276d1084a8f74 (docs(scripts): trim session-friction docstring under file-line limit); c7c03bd31265cd8a651f3e7008fa25035bf8d9b6 (refactor(src): extract type/tag/line-match filters in search collect_matches); cb95f6a496e968081b6d23f2cc2dfb2fe9af0543 (refactor(src): split new run into per-kind handlers); d0409f675944aeaeb4f1404feb8fa11e1dc2f75c (refactor(src): split ensure_workspace_current into phase helpers); d39966055b7d6a95b29c892160d9fba6e97146f3 (refactor(sync_core): split tests out; decompose long projection fns); d820576934f2fff850d85f668d7a4e8a2fa29522 (refactor(tests): extract spawn/status runners to cut abc in liveness test); dc26d0225401e1c965cda725ddb6d5cc8250a48c (refactor(src): decompose search run and match collection into helpers); dd23d17a734ad687d5ccf7903713659b62b83383 (refactor(examples): hoist llm.txt body into const to cut abc); e1e7383809877443efad1137c3a38bdac9ab9b07 (refactor(add): remove superseded add.rs); e22c6a52441de0d7b888e681f380877ae747334c (refactor(status): decompose run() into per-section renderers); e42601ddddf50a7e88ee36662cedc235cdbc6c0d (refactor(src): split matrix run into per-subcommand handlers); e5a4f9f93bc319c9f5d76b54930e95b1388f5837 (refactor(src): split llm.rs into module dir (claude_api/claude_cli/ollama/agent/tests)); e7a58b1aad1182df7878f98719d94873af5ff5f1 (refactor(src): extract artifact-dir scan helpers in freshness.rs); ea5b07ac47f6723e5bccd3d2e360186e596f1f2d (style(tests): cargo fmt workflows_test); ef101ee3842fe59f43932001ba9f7ff1e83c07d4 (refactor(tests): merge near-duplicate workflow tests to clear lazy_cluster); f4987edceb4de0b970b8ec051df104c0c2db7055 (refactor(tests): keep matrix_test under file limit with fmt-stable helpers); f713539cd3d7cbc95c062d25be3b22205b6ad08f (docs: apply issue-review fixes to README/release tickets); f8ddfb355635491a193a94a1b0fc57f8c29bc00a (refactor(src): split config run into per-subcommand handlers); fa165c6306704203d1c92b8aac4b0c31a9d8ce52 (refactor(src): extract plugin-context and content-template helpers in handoff.rs); fb70c2e7b3109d27fa460534f124dcaf332254be (refactor(tests): extract run_pipeline helper to cut abc in bypass telemetry test); spec_change_without_bead: openspec changes link no tracker id: `add-cross-tool-trace-ingestion` changed 0 days ago, `add-plugin-trust-model` changed 0 days ago, `adopt-genesis` changed 0 days ago
- signal: orphan commits — 68 commits in the 14-day window link no tracker id or open work-item path: 01b44d833622a923674404235932be4fa7663b7a (release: 2026.11.17 — layout-aware openspec pointer; green pre-commit); 107e67156ab23b9392cda474894c386a9db77201 (bd: file baseline-drift bug (pre-existing test failure)); 1c5ded10bada1e91cd6f389f887e3b293ebfffd3 (refactor(tests): table-drive way_check and help scenarios to clear lazy_cluster in doctor suite); 1dd83898ae9f8853317437942bcbbe0fb9dbbf9c (refactor(plugin): split plugin.rs into plugin/ submodules); 21b4e0b4b2ef7d1d6a0bf3b65482f97619dad37d (spec: make the corpus valid under openspec 2.1.0); 224096f91f8e95fd4fdf0ef7ea66ae0e8aee181b (refactor(src): split detect_patterns into per-pattern push helpers); 2428e0c955a4dd1a5f1a675368dd50e39303796b (refactor(tests): flatten walk dir traversal to reduce nesting); 2a8317b0c6770004d62727e76e2c2f7e7d84b9ba (chore(scripts): drop unused SESSIONS_DIR, inline token comment); 46ce48de86177a0804600f95d8433fdad57e5acc (docs: file README front-door and release tickets); 493abc501cf51f3195f15254c3b5ead65bfe61ab (refactor(src): decompose doctor checks_sync strategy checks; reduce mutable bindings); 5a9a1358350df5ba392158066fba7315c5192321 (fix(pipeline): terminate the bail macro so the newer-stable lint passes); 5f7c4921e0599ecfda7e6d32c494304361857bec (refactor(src): split ls run and discover_workspaces into helpers); 632aab2da537a86a95f5f735a73dc3e644c93cb8 (chore(skills): retire the pre-2.1 openspec workflows); 65f60c58d7a06c120f5950a3e8f46d0fdbfb48a3 (refactor(tests): dedupe matrix/plugin-list/tdd-ro5 lazy test clusters via tables and helpers); 6986cc6e20f7e6cd7c0843033d7f24a9c967ef2f (refactor(src): data-driven help tables; split render_main_help sections); 6d3c95609eb1b832b91cabc7e5879f2c1b6bb43d (refactor(tests): split epic-orchestrator walk test into phase helpers); 70b8f6cb4298a367b84692182c567e5397c3c70f (chore: remove superseded matrix.rs/plugin.rs/sync_core.rs; fmt touch-ups); 78b47385f34c81370f6182a688b0d36f8ab79a77 (refactor(src): decompose pipeline queries into per-phase helpers); 7f32379417f5fe15932016dc7c8178840b92e733 (refactor(tests): vary pipeline TOML helper structure to break lazy_cluster in common); 80ad7b0f983ab4233c5061c2c11722fd358f7dc3 (refactor(init): extract per-path handlers from run() dispatcher); 85b67dfcc714e01ae1f5e2d5708a9fb66055072b (refactor(scripts): split session-friction main into select/fetch/signals/aggregate helpers); 86e05f91aa82c465745153dd73cb0576376e18af (docs: apply issue-review fixes to README/release tickets); 86ea659d8b83869d7157f4d0b15ad0cf199e25f5 (refactor(src): split build_prompt into sections; remove mutable bindings in why/prompt.rs); 8845c882acb38d8ac26e286cf59779cd7364b442 (style: cargo fmt across pipeline/add modules and test suites); 936512c7898aea0f5908bdf7b665524bd9975c1a (chore(openspec): migrate project.md into config.yaml; pin 2.1.0); 93c282fbba95f2fa729697eed37b3374e1a9a4f0 (refactor(src): split close run into summary/remember helpers; tidy ls); 990cb7972e1426c21afe8e1e7b83059b2895fede (test: assert claim sha in walk test body for min_assertions); 9b9183d601dc2d9b2d090c512885292fd6773d14 (refactor(src): extract external-command dispatch helper in commands/mod.rs); 9c4422b054a08f53ceb8d637e464d4e828178934 (chore(openspec): adopt the 2.1 artifact layout); 9d6d6722e37e357d2ebe2cc1f75aef1dcabe3f5b (refactor(pipeline): extract model.rs and verify.rs from mod.rs; split tests into files); 9d95cd5e761feeed887f538e20f0c9c4fb2a9416 (chore(gates): render the managed gate blocks from .ddl/config.toml); a170021b433a504145701eaf1b866cd07a39fb55 (docs(brief): pretender debt fix brief (wai) — orchestrator artifact); a2776313bc4ee7f241b4e7fd0ae239a21e0aada5 (refactor(pipeline): split orchestration.rs into orchestration/ submodules); a31e6e83cc871388848b3aff0f87121e45dffa45 (refactor(src): split timeline run into collect/sort/print helpers); a38f071a820b67f1206284541ac61cd0642d24cb (refactor(sync): extract projection execution and from-main sync helpers); a53d51b31d07b5584c90b5b0f9fab99cd798795f (test(archive): extract make_archive/import_archive helpers from round-trip tests); a91c8167eb3082c79cf8c6cd608c2239b939e6ce (refactor(add): split add.rs into add/ module dir); ab26fa45d171e15b8235fcc49ff48ff44b389f63 (refactor(src): extract shared walk/budget helpers in reflect/context.rs); ad59b564212461a2600265bfe5a0d4e429749868 (fix(tests): replace unwrap/expect with panic-context alternatives); b035e7f10afb01be139b4bae2a170f8f241f7409 (refactor(plugin): extract per-subcommand handlers from run()); b600d07d286c30caf5cc3efbac69a24d405a5920 (refactor(tests): merge near-duplicate matrix lint tests below file_lines limit); b65e523f88e10c6feb1862fed8fb8feb9d3da847 (refactor(sync): flatten claude_code_needs_sync into iterator chains); b8b3892b6074129078164a7707f300edcb756a6a (refactor(src): extract plugin-context and content-template helpers in handoff.rs); bc2fad24b8feaa25f9d9523264c1111f17b99e03 (refactor(src): split tutorial run into step-runner and summary helpers); bd4a02bac28b998507b10b6ee5e0d4f49fb4a122 (refactor(src): decompose doctor checks_basic into per-concern helpers); be652db3db6a32ec6fac6d6f29728d81bb00a978 (style: cargo fmt freshness.rs); bf8181d7793dc080193ed70068351f440a207e0e (refactor(matrix): split matrix.rs into matrix/ submodules); c224ad2c6688aae47133c20a23362a7976903dea (refactor(tests): extract shared script/pipeline-file helpers in common); c269c677a280648236297881de1276d1084a8f74 (docs(scripts): trim session-friction docstring under file-line limit); c7c03bd31265cd8a651f3e7008fa25035bf8d9b6 (refactor(src): extract type/tag/line-match filters in search collect_matches); cb95f6a496e968081b6d23f2cc2dfb2fe9af0543 (refactor(src): split new run into per-kind handlers); d0409f675944aeaeb4f1404feb8fa11e1dc2f75c (refactor(src): split ensure_workspace_current into phase helpers); d39966055b7d6a95b29c892160d9fba6e97146f3 (refactor(sync_core): split tests out; decompose long projection fns); d820576934f2fff850d85f668d7a4e8a2fa29522 (refactor(tests): extract spawn/status runners to cut abc in liveness test); dc26d0225401e1c965cda725ddb6d5cc8250a48c (refactor(src): decompose search run and match collection into helpers); dd23d17a734ad687d5ccf7903713659b62b83383 (refactor(examples): hoist llm.txt body into const to cut abc); e1e7383809877443efad1137c3a38bdac9ab9b07 (refactor(add): remove superseded add.rs); e22c6a52441de0d7b888e681f380877ae747334c (refactor(status): decompose run() into per-section renderers); e42601ddddf50a7e88ee36662cedc235cdbc6c0d (refactor(src): split matrix run into per-subcommand handlers); e5a4f9f93bc319c9f5d76b54930e95b1388f5837 (refactor(src): split llm.rs into module dir (claude_api/claude_cli/ollama/agent/tests)); e7a58b1aad1182df7878f98719d94873af5ff5f1 (refactor(src): extract artifact-dir scan helpers in freshness.rs); ea5b07ac47f6723e5bccd3d2e360186e596f1f2d (style(tests): cargo fmt workflows_test); ef101ee3842fe59f43932001ba9f7ff1e83c07d4 (refactor(tests): merge near-duplicate workflow tests to clear lazy_cluster); f4987edceb4de0b970b8ec051df104c0c2db7055 (refactor(tests): keep matrix_test under file limit with fmt-stable helpers); f713539cd3d7cbc95c062d25be3b22205b6ad08f (docs: apply issue-review fixes to README/release tickets); f8ddfb355635491a193a94a1b0fc57f8c29bc00a (refactor(src): split config run into per-subcommand handlers); fa165c6306704203d1c92b8aac4b0c31a9d8ce52 (refactor(src): extract plugin-context and content-template helpers in handoff.rs); fb70c2e7b3109d27fa460534f124dcaf332254be (refactor(tests): extract run_pipeline helper to cut abc in bypass telemetry test)
- signal: spec change without bead — openspec changes link no tracker id: `add-cross-tool-trace-ingestion` changed 0 days ago, `add-plugin-trust-model` changed 0 days ago, `adopt-genesis` changed 0 days ago

## Sources & freshness

| source | status | observed_at | staleness | detail |
| --- | --- | --- | --- | --- |
| beads | present | 2026-11-18T14:59:37Z | fresh | 44 malformed line(s) skipped in .beads/issues.jsonl |
| git | present | 2026-11-18T16:03:39Z | fresh | none |
| openspec | present | 2026-11-18T15:46:23Z | fresh | none |
| whisper | present | 2026-10-19T21:24:50Z | stale | none |
| curation | present | 2026-11-18T21:12:08Z | fresh | none |

Curation: updated_at 2026-11-18T21:12:08Z; confirmed_at none.

Freshness: generated_at 2026-11-18T21:12:08Z; per-source observed_at and staleness are listed above.
