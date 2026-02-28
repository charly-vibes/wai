## 1. Data model (`src/commands/pipeline.rs`, `src/config.rs`)

- [ ] 1.1 Add `PipelineStep { id: String, prompt: String }` struct
- [ ] 1.2 Add `PipelineDefinition { name: String, description: Option<String>, steps: Vec<PipelineStep> }` (TOML format, replaces old struct)
- [ ] 1.3 Update `PipelineRun`: rename `current_stage: usize` → `current_step: usize`; remove `stages: Vec<RunStage>`
- [ ] 1.4 Add `last_run_path()` helper to `src/config.rs` returning `.wai/resources/pipelines/.last-run`
- [ ] 1.5 Add `load_pipeline_toml()`: deserialize TOML, validate unique IDs and non-empty prompts
- [ ] 1.6 Add `render_prompt()`: substitute `{topic}` in a prompt string with the given topic value

## 2. Command: `pipeline init` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [ ] 2.1 Add `PipelineCommands::Init { name }` variant to `src/cli.rs`
- [ ] 2.2 Implement `cmd_init`: create `.wai/resources/pipelines/` if absent, write a minimal two-step TOML template
- [ ] 2.3 Fail with clear error if `<name>.toml` already exists

## 3. Command: `pipeline start` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [ ] 3.1 Add `PipelineCommands::Start { name, topic }` variant to `src/cli.rs`
- [ ] 3.2 Implement `cmd_start`: load and validate TOML definition, generate run ID, write run state YAML
- [ ] 3.3 Write run ID to `.last-run` pointer file
- [ ] 3.4 Print env export line + first step prompt block (step 1/N header, rendered prompt)

## 4. Command: `pipeline next` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [ ] 4.1 Add `PipelineCommands::Next` variant to `src/cli.rs` (no arguments)
- [ ] 4.2 Implement `cmd_next`: resolve run ID from `WAI_PIPELINE_RUN` env var, then `.last-run`; error if neither
- [ ] 4.3 Error if resolved run is already complete
- [ ] 4.4 Mark current step complete (reuse existing tag-discovery logic for artifact path)
- [ ] 4.5 Increment `current_step`, persist run state
- [ ] 4.6 If more steps remain: print next step prompt block; if last step completed: print completion block with `wai close` suggestion

## 5. Command: `pipeline current` (`src/cli.rs`, `src/commands/pipeline.rs`)

- [ ] 5.1 Add `PipelineCommands::Current` variant to `src/cli.rs`
- [ ] 5.2 Implement `cmd_current`: resolve run ID (env → `.last-run`), load run state and definition, print current step prompt block
- [ ] 5.3 Error clearly when no active run found

## 6. Remove old commands (`src/cli.rs`, `src/commands/pipeline.rs`)

- [ ] 6.1 Remove `PipelineCommands::Create`, `Run`, `Advance` variants and their implementations
- [ ] 6.2 Remove `parse_stages()`, `validate_skill_exists()`, and `load_pipeline_definition()` (YAML loader)
- [ ] 6.3 Remove `serde_yaml` usage from `src/commands/pipeline.rs` for definitions (keep for run state)

## 7. `wai status` pipeline integration (`src/commands/status.rs`)

- [ ] 7.1 Read `.last-run` pointer on status (fall back to `WAI_PIPELINE_RUN` env)
- [ ] 7.2 If active run found: load run state, emit pipeline section ("⚡ PIPELINE ACTIVE: <name> step N/M")
- [ ] 7.3 Add `wai pipeline current` to suggestions block when active run is present

## 8. Tests

- [ ] 8.1 Unit test: `render_prompt()` substitutes `{topic}` correctly; no panic on missing placeholder
- [ ] 8.2 Unit test: `load_pipeline_toml()` rejects duplicate step IDs with named error
- [ ] 8.3 Unit test: `load_pipeline_toml()` rejects empty prompts with named step ID in error
- [ ] 8.4 Integration test: `pipeline start` → `pipeline next` (mid-run) → `pipeline next` (last step) → completion block
- [ ] 8.5 Integration test: `.last-run` written on start; `pipeline next` resolves it when env var absent
- [ ] 8.6 Integration test: `pipeline current` re-prints prompt after env var cleared
- [ ] 8.7 Integration test: `pipeline next` on already-complete run errors clearly
