## Dependencies
Phases 1-2 are prerequisites for phase 3 (gate engine needs step tags and review type).
Phase 3 is prerequisite for phase 5 (commands wrap the gate engine).
Phases 4, 6, and 7 can proceed in parallel after phase 3.

## 1. Step-level artifact tagging
- [ ] 1.1 Read `current_step` from run state in `wai add` and inject `pipeline-step:<step-id>` tag
- [ ] 1.2 Add tests for step-level tag injection alongside existing run-level tag tests

## 2. Review artifact type
- [ ] 2.1 Add `review` artifact type with `REVIEWS_DIR` constant and directory creation in workspace
- [ ] 2.2 Implement `wai add review "..." --reviews <artifact-filename>` command
- [ ] 2.3 Parse and validate review frontmatter: `reviews`, `verdict`, `severity`, `produced_by`
- [ ] 2.4 Include review artifacts in `wai search` and `wai timeline`
- [ ] 2.5 Add tests for review creation, frontmatter validation, and search integration

## 3. Gate protocol engine
- [ ] 3.1 Define gate TOML schema in pipeline parser (structural, procedural, oracle, approval sections)
- [ ] 3.2 Implement structural gate: query artifacts by step + run tags, check count and types
- [ ] 3.3 Implement procedural gate: find review artifacts matching each step artifact, check verdict/severity
- [ ] 3.4 Implement oracle gate: resolve command, execute with artifact path, handle exit code and timeout
- [ ] 3.5 Implement approval gate: check run state for approval timestamp
- [ ] 3.6 Integrate gate evaluation into `wai pipeline next` — block on first failure with reason
- [ ] 3.7 Add tests for each gate tier individually and combined evaluation order

## 4. Oracle system
- [ ] 4.1 Implement oracle name resolution: `.wai/resources/oracles/<name>` with extension probing
- [ ] 4.2 Support explicit `command` override in TOML
- [ ] 4.3 Execute oracles with configurable timeout (default 30s)
- [ ] 4.4 Create `.wai/resources/oracles/` directory in workspace setup
- [ ] 4.5 Scaffold `README.md` and `example-check.sh` when `wai pipeline init` creates a gated pipeline
- [ ] 4.6 Add tests for resolution, execution, timeout, and failure reporting

## 5. New pipeline commands
- [ ] 5.1 `wai pipeline show <name>` — detailed view with steps, gates, oracles
- [ ] 5.2 `wai pipeline gates [pipeline-name] [--step=<id>]` — gate requirements and live status
- [ ] 5.3 `wai pipeline check [--oracle=<name>]` — dry-run gate evaluation without advancing
- [ ] 5.4 `wai pipeline approve` — set approval timestamp in run state
- [ ] 5.5 `wai pipeline validate [name]` — validate TOML structure, gate config, oracle paths, metadata
- [ ] 5.6 Add tests for each new command

## 6. Pipeline metadata and managed block
- [ ] 6.1 Add `[pipeline.metadata]` parsing to pipeline TOML schema (`when`, `skills` fields)
- [ ] 6.2 Update `wai_block_content()` to read installed pipelines and generate "Available Pipelines" table
- [ ] 6.3 Implement managed block staleness detection in `wai doctor` (diff generated vs actual)
- [ ] 6.4 Run `wai pipeline validate` as part of `wai doctor`
- [ ] 6.5 Validate before `wai pipeline start` — block on errors, warn on warnings
- [ ] 6.6 Add tests for metadata parsing, block generation, staleness detection

## 7. Pipeline init scaffolding
- [ ] 7.1 Update `wai pipeline init` to accept built-in template names (e.g., `scientific-research`)
- [ ] 7.2 Ship `scientific-research` as a built-in template
- [ ] 7.3 Scaffold oracle directory and example files for gated pipelines
- [ ] 7.4 Add tests for template scaffolding
