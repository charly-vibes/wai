## Prerequisites

This change requires `add-hierarchical-skills` and `add-artifact-tags` to be
implemented first (pipeline runs tag artifacts with run IDs).

## 1. Define CLI surface

- [x] 1.1 Add `Pipeline` subcommand to `src/cli.rs` with:
        `create`, `run`, `advance`, `status`, `list` sub-subcommands
- [x] 1.2 Create `src/commands/pipeline.rs` and wire it in `src/commands/mod.rs`
- [x] 1.3 Add `pipelines/` directory to the resources path in `src/config.rs`

## 2. Pipeline creation

- [x] 2.1 Implement `wai pipeline create <name> --stages="skill:artifact,..."` —
        parse and validate the stages string; reject unknown skill names
- [x] 2.2 Persist the pipeline definition as YAML in
        `.wai/resources/pipelines/<name>.yml`
- [x] 2.3 Add unit tests for stage string parsing (valid and malformed inputs)

## 3. Pipeline run

- [x] 3.1 Implement `wai pipeline run <name> --topic=<slug>` — generate a run ID
        (e.g., `<name>-<date>-<topic>`), persist run state in
        `.wai/resources/pipelines/<name>/runs/<id>.yml`
- [x] 3.2 Output the run ID and the stage 1 skill invocation hint to stdout
- [x] 3.3 Implement `wai pipeline advance <run-id>` — marks the current stage complete,
        records the artifact path (via `pipeline-run:<run-id>` tag lookup), and outputs
        a hint to invoke the next stage skill; error if run ID unknown or all stages done
- [x] 3.4 Tag artifacts automatically with `pipeline-run:<run-id>` when a skill
        uses any `wai add <type>` command during a run (detect via env var `WAI_PIPELINE_RUN`)

## 4. Pipeline status

- [x] 4.1 Implement `wai pipeline status <name>` — list all runs with per-stage
        completion status and artifact paths
- [x] 4.2 Add `--run <id>` filter to show detail for a single run

## 5. Tests

- [x] 5.1 Integration test: create pipeline, run it, advance through stages, verify
        status output shows correct per-stage state
- [x] 5.2 Test error paths: `advance` with unknown run ID, `advance` past final stage

## 6. Documentation

- [x] 6.1 Update `--help` strings for all new subcommands (`pipeline create`, `run`,
        `advance`, `status`, `list`)
- [x] 6.2 Document the `WAI_PIPELINE_RUN` env var in `wai pipeline run --help` output,
        including the `export WAI_PIPELINE_RUN=<id>` hint
