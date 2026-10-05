---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-05-wai-vx02-3-epic-run-tree, pipeline-step:execute, ticket:wai-vx02.3]
---

# Design: epic run tree (wai-vx02.3)

Supersedes the failed first GREEN loop (catastrophic RED violation recorded
2026-10-05 15:32): its 4 tests targeted an imaginary fixture API
(`setup_workspace_env`, `create_beads_issue`, `create_in_progress_run_fixture`)
and did not compile. This design re-shapes the tests against the real fixture
API (`init_workspace`, `write_pipeline_toml`, `install_fake_bd*` + PATH
injection, hand-written run-state YAML) and scopes to exactly the plan
artifact's four features — no prime changes, no close changes (close refusal of
any mid-flight run via `.last-run` is already covered by wai-csgb).

## Feature 1 — `pipeline start <pipeline> --epic=<id>` (discovery + parent run)

- New clap arg `epic: Option<String>` on `PipelineCommands::Start`.
- In `cmd_start` when `--epic` is set:
  - Invoke `bd ready --json` via `std::process::Command` in project root.
    Parse a JSON array; collect `id` of entries whose `parent` == epic id.
    bd failure or non-array output → bail "epic start requires bd".
  - No matching ready children → print "Epic `<id>` has no ready children —
    no parent run created", exit success, create nothing.
  - Ready children found → create a parent run: run_id =
    `<pipeline>-<date>-<epic-slug>-parent` (`--topic` defaults to epic id),
    then write run state and `.last-run` as usual, print discovered child ids.
  - Idempotent: if a parent run yml for the same epic already exists in
    `.wai/pipeline-runs/` (scan for `epic: <id>` field), reuse it — print
    "Epic parent run already active: <run-id>" and do not rewrite state.

## Feature 2 — child-run ids recorded on parent state

- `PipelineRun` gains three `#[serde(default)]` fields:
  `epic: Option<String>`, `child_issues: Vec<String>`, `child_runs: Vec<String>`.
- When a *child* run is started normally (`--topic=<child-issue-id>`) and some
  existing parent run has that topic in its `child_issues`, append the new
  run_id to that parent's `child_runs` (read-modify-write of the parent yml).

## Feature 3 — parent advance blocked while mid-flight children

- `cmd_next`: when the active run has `epic` set, load each run yml named in
  `child_runs`; if any has `current_step < <steps of its pipeline def>` →
  bail "Epic parent run cannot advance while child runs are mid-flight: <ids>"
  before advancing. Parent state untouched on refusal.

## Feature 4 — `pipeline current --json` renders the tree

- `PipelineCurrentPayload` gains `#[serde(skip_serializing_if = "Option::is_none")]
  epic: Option<EpicTreePayload>` where
  `EpicTreePayload { epic: String, children: Vec<ChildRunNode> }`,
  `ChildRunNode { issue: String, run_id: Option<String>, mid_flight: bool }`.
  Populated only when the active run is an epic parent: children resolved from
  `child_issues` ∪ run ymls in `pipeline-runs/` matching those topics.

## Test shape (RED first; tests/integration.rs, real fixture API)

Helper `install_fake_bd_ready_json(dir, issues_json)` — stub `bd` responding
only to `ready --json` (exit 0, print payload). Helper
`write_epic_run_fixture(dir, run_id, epic, child_issues, child_runs, step)`
writes parent run yml + `.last-run` pointer (style of close_test.rs
`write_pipeline_run`).

1. `pipeline_start_epic_discovers_children_creates_parent_run` — stub returns
   children of `epic-1` and `epic-2`; start `--epic=epic-1` → one yml named
   `...-parent`, containing `epic: epic-1`, `child-a`/`child-b` in
   `child_issues`, not `other`. Second start → still exactly one run yml
   (idempotent reuse).
2. `pipeline_start_epic_skips_when_no_ready_children` — stub returns only
   `epic-2` children; start `--epic=epic-1` → success, stdout says "no ready
   children", zero run ymls, no `.last-run`.
3. `pipeline_start_child_appends_run_to_epic_parent` — parent fixture with
   `child_issues: [child-a]`; `pipeline start my-pipe --topic=child-a` →
   parent yml's `child_runs` contains the new run id.
4. `pipeline_next_epic_parent_blocked_while_children_midflight` — parent
   fixture (child_runs: [child-run]) + mid-flight child run yml; `wai pipeline
   next` → failure, stderr mentions mid-flight child run, parent step
   unchanged.
5. `pipeline_current_json_renders_epic_tree` — same fixtures; `wai pipeline
   current --json` → JSON contains `"epic"`, the child node's `run_id`, and
   `"mid_flight":true`.

RED = all five compile and fail on missing behavior. GREEN = features 1-4 in
`src/commands/pipeline/{subcommands via cli.rs, orchestration.rs, mod.rs}` +
`src/json.rs`. `bd ready --json` parent-field parsing is a recorded
contract assumption: beads issue JSON carries `id` and `parent` string fields.

## Verification (GREEN evidence, commands run 2026-10-05)

- `cargo test --test integration epic_` → 5 passed, 0 failed (RED first:
  5/5 failed before implementation)
- `cargo test --test integration` → 382 passed, 0 failed
- `cargo test` → 498 passed, 1 failed = known timing-flake
  `plugin::tests::execute_hook_no_deadlock_on_fast_command` (wai-z25x);
  passes in isolation re-run
- `cargo fmt --all` → clean; `cargo clippy --all-targets` → 0 warnings
- Implementation files: `src/cli/subcommands.rs` (Start `--epic`),
  `src/commands/pipeline/mod.rs` (PipelineRun epic/child_issues/child_runs
  serde-default fields), `src/commands/pipeline/orchestration.rs`
  (cmd_start_epic, discover_ready_children, find_epic_parent_run,
  record_child_run_on_epic_parent, next mid-flight block, epic tree),
  `src/json.rs` (EpicTreePayload, ChildRunNode), `tests/integration.rs`
  (5 tests + 2 real-API fixtures).
