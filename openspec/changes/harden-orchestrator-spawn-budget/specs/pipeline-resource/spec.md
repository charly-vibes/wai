## MODIFIED Requirements

### Requirement: Orchestrator template encodes the canon loop

The `epic-orchestrator` template SHALL encode the orchestrator canon's spawn-safety
invariants as enforceable step prompts, not prose conventions.

#### Scenario: Spawn carries a wall-clock budget

- **WHEN** the epic-orchestrator template's spawn step is read after scaffolding
- **THEN** it requires the spawn command to be wrapped in a wall-clock budget
  (`timeout <budget>`) sized to the ticket and stated in the brief
- **AND** it forbids bare foreground `pi -p` calls that block until the harness
  timeout

#### Scenario: Interrupted-but-healthy spawn resumes, not re-prompts

- **WHEN** a spawn was interrupted by timeout or orchestrator crash with evidence of
  progress (commits or session record)
- **THEN** the retry path resumes the existing named session via
  `pi -c -p -n "<name>-resume"` instead of re-prompting the full brief from scratch
- **AND** re-prompting the full brief in that situation is stated as a violation of
  the step

#### Scenario: Beads DB is never staged

- **WHEN** the ship step prepares the commit and push
- **THEN** the step explicitly forbids `git add .beads` and committing the beads
  database, which the global gitignore already excludes

#### Scenario: Verify refuses record-less spawns

- **WHEN** the verify step cannot locate the spawn's log path or session record
- **THEN** the step marks the spawn as failed rather than passing verification
