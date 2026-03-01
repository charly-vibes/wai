## 1. Data model (`src/commands/pipeline.rs`, `src/config.rs`)

- [x] 1.1 Add `PipelineStep { id: String, prompt: String }` struct
- [x] 1.2 Add `PipelineDefinition { name: String, description: Option<String>, steps: Vec<PipelineStep> }` (TOML format, replaces old struct)
- [x] 1.3 Update `PipelineRun`: rename `current_stage: usize` → `current_step: usize`; remove `stages: Vec<RunStage>`
- [x] 1.4 Add `last_run_path()` helper to `src/config.rs` returning `.wai/resources/pipelines/.last-run`
- [x] 1.5 Add `load_pipeline_toml()`: deserialize TOML, validate unique IDs and non-empty prompts
- [x] 1.6 Add `render_prompt()`: substitute `{topic}` in a prompt string with the given topic value

## 2. Command: `pipeline init` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [x] 2.1 Add `PipelineCommands::Init { name }` variant to `src/cli.rs`
- [x] 2.2 Implement `cmd_init`: create `.wai/resources/pipelines/` if absent, write a minimal two-step TOML template (thin prompt style per section 9 — see 9.1 for template content)
- [x] 2.3 Fail with clear error if `<name>.toml` already exists

## 3. Command: `pipeline start` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [x] 3.1 Add `PipelineCommands::Start { name, topic }` variant to `src/cli.rs`
- [x] 3.2 Implement `cmd_start`: load and validate TOML definition, generate run ID, write run state YAML
- [x] 3.3 Write run ID to `.last-run` pointer file
- [x] 3.4 Print env export line + first step prompt block (step 1/N header, rendered prompt)

## 4. Command: `pipeline next` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [x] 4.1 Add `PipelineCommands::Next` variant to `src/cli.rs` (no arguments)
- [x] 4.2 Implement `cmd_next`: resolve run ID from `WAI_PIPELINE_RUN` env var, then `.last-run`; error if neither
- [x] 4.3 Error if resolved run is already complete
- [x] 4.4 Mark current step complete (reuse existing tag-discovery logic for artifact path)
- [x] 4.5 Increment `current_step`, persist run state
- [x] 4.6 If more steps remain: print next step prompt block; if last step completed: print completion block with `wai close` suggestion

## 5. Command: `pipeline current` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [x] 5.1 Add `PipelineCommands::Current` variant to `src/cli.rs`
- [x] 5.2 Implement `cmd_current`: resolve run ID (env → `.last-run`), load run state and definition, print current step prompt block
- [x] 5.3 Error clearly when no active run found

## 6. Remove old commands (`src/cli.rs`, `src/commands/pipeline.rs`)

- [x] 6.1 Remove `PipelineCommands::Create`, `Run`, `Advance` variants and their implementations
- [x] 6.2 Remove `parse_stages()`, `validate_skill_exists()`, and `load_pipeline_definition()` (YAML loader)
- [x] 6.3 Remove `serde_yaml` usage from `src/commands/pipeline.rs` for definitions (keep for run state)

## 7. `wai status` pipeline integration (`src/commands/status.rs`)

- [ ] 7.1 Read `.last-run` pointer on status (fall back to `WAI_PIPELINE_RUN` env); if pointer exists but run file is missing, treat as no active run (stale pointer silently ignored)
- [ ] 7.2 If active run found: load run state, emit pipeline section ("⚡ PIPELINE ACTIVE: <name> step N/M")
- [ ] 7.3 Add `wai pipeline current` to suggestions block when active run is present
- [ ] 7.4 If no active run and pipelines directory has at least one `.toml` definition: emit "Available pipelines" section listing name, description, and step count; skip malformed TOML files with inline warning ("⚠ pipeline <name>: invalid TOML, skipped") rather than erroring
- [ ] 7.5 Add `wai pipeline suggest` to suggestions block when pipelines are present but no run is active

## 8. Command: `pipeline suggest` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [x] 8.1 Add `PipelineCommands::Suggest { description: Option<String> }` variant to `src/cli.rs`
- [x] 8.2 Implement `cmd_suggest`: scan `.wai/resources/pipelines/` for TOML definitions, load each, print name + description + step count
- [x] 8.3 If `description` provided and non-empty: score each pipeline by keyword overlap (case-insensitive; split description into words, count matches against pipeline name + description fields); sort by score descending; break ties alphabetically by name; if all score 0, sort alphabetically; treat empty string as absent (no scoring)
- [x] 8.4 Print `wai pipeline start <name> --topic=<your-topic>` hint for the top-ranked result (or first alphabetical result if no description given), using a concrete placeholder rather than `<slug>`
- [x] 8.5 If no pipelines found: print "No pipelines defined" with hint to run `wai pipeline init`

## 9. Step prompt convention (init template content)

- [ ] 9.1 Update `pipeline init` starter template to demonstrate thin prompt style: one-line task summary + optional skill hint + `wai add` + `wai pipeline next`
- [ ] 9.2 Add a comment in the generated template explaining the convention: "Step prompts are navigation hints. Instructions for HOW to do the work belong in skills."

## 10. Tests

- [ ] 10.1 Unit test: `render_prompt()` substitutes `{topic}` correctly; no panic on missing placeholder
- [ ] 10.2 Unit test: `load_pipeline_toml()` rejects duplicate step IDs with named error
- [ ] 10.3 Unit test: `load_pipeline_toml()` rejects empty prompts with named step ID in error
- [ ] 10.4 Integration test: `pipeline start` → `pipeline next` (mid-run) → `pipeline next` (last step) → completion block
- [ ] 10.5 Integration test: `.last-run` written on start; `pipeline next` resolves it when env var absent
- [ ] 10.6 Integration test: `pipeline current` re-prints prompt after env var cleared
- [ ] 10.7 Integration test: `pipeline next` on already-complete run errors clearly
- [ ] 10.8 Unit test: `pipeline suggest` with no description lists all pipelines sorted alphabetically
- [ ] 10.9 Unit test: `pipeline suggest "fix auth bug"` ranks pipeline with matching keywords above unrelated ones; deterministic order for equal-score pipelines (alphabetical)
- [ ] 10.10 Unit test: `pipeline suggest "xyz123"` with no matches returns all pipelines in alphabetical order
- [ ] 10.11 Integration test: `wai status` emits "Available pipelines" section when no run active and pipelines exist
- [ ] 10.12 Integration test: `wai status` falls back to idle state when `.last-run` points to a missing run file (stale pointer)

## 11. Spec update: `context-suggestions` (`openspec/specs/context-suggestions/`)

- [ ] 11.1 Update context-suggestions spec to reflect that the wai status suggestions block gains `wai pipeline suggest` (when idle with pipelines present) and `wai pipeline current` (when a pipeline run is active)
