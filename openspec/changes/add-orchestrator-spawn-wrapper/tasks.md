# Tasks: add-orchestrator-spawn-wrapper

Sequencing: implement after `add-run-wait-primitive` landed (both touch
`src/commands/pipeline/orchestration.rs` and the epic-orchestrator template —
EXCL-001). Rebase template expectations onto current template state.

## 1. Regression tests (red-first)

- [x] 1.1 `orchestrator_template_requires_spawn_wrapper`: template scan asserts
      the epic-orchestrator spawn step names `wai pipeline spawn` as the
      required path and names improvised manual `pi -p ... &` incantations as
      a violation (pattern: suite_pipeline_wait 1.1)
- [x] 1.2 `spawn_wrapper_writes_checkpoint`: hermetic integration test —
      PATH-stubbed `pi` (sleeping stub), `wai pipeline spawn <name> <prompt>`
      returns immediately, checkpoint `.wai/pipeline-runs/spawns/<name>.json`
      exists with session name, pid, log path, prompt path, budget; stub `pi`
      received the named-session invocation (`pi -p -n <name> <prompt>`) and
      output lands in the log file
- [x] 1.3 `spawn_status_classifies_liveness`: with the stub running,
      `wai pipeline spawn status <name>` prints one line naming running state
      and the log path; after the stub's pid dies, status classifies exited
      with a log-tail probe line; pure `pipeline_spawn::classify` unit tests
      pin running/exited/budget-exhausted mapping
- [x] 1.4 `spawn_resume_continues_named_session`: killed run + checkpoint →
      `wai pipeline spawn resume <name>` launches `pi -c -p -n "<name>-resume"`
      (stub records argv), writes fresh checkpoint `<name>-resume.json`, and
      does not re-prompt the original full prompt

## 2. Implementation

- [x] 2.1 `src/pipeline_spawn.rs` pure module (pattern: pipeline_wait):
      `SpawnCheckpoint` (JSON ser/de), `classify` (pid liveness + log probe →
      running/exited/budget-exhausted), `status_line` (one line per spawn),
      `resume_invocation` (canon Invariant 6 pi -c shape); declared in both
      lib and bin
- [x] 2.2 `wai pipeline spawn` verb group in orchestration.rs:
      `cmd_spawn` (detached launch under `timeout` budget, log redirection,
      checkpoint write), `cmd_spawn_status` (checkpoint read + probe),
      `cmd_spawn_resume` (verify checkpoint, detached resume launch, fresh
      checkpoint); CLI wiring in subcommands.rs + pipeline/mod.rs
- [x] 2.3 epic-orchestrator template (builtin `src/templates/`): spawn step
      requires the wrapper, names manual incantations as violation

## 3. Validation

- [x] 3.1 `just test` green (1.1–1.4 included)
- [x] 3.2 `just ci` green
- [x] 3.3 `openspec validate add-orchestrator-spawn-wrapper --strict`
- [x] 3.4 Live validation: `wai pipeline spawn` a real short pi session,
      status probe answers, budget enforcement observable
- [ ] 3.5 Follow-up: fold the pattern doc's invocation-shape projection
      (`~/.wai/resources/patterns/orchestrator-subagents.md`, chezmoi-managed)
      to reference the wrapper flags — separate ticket after this ships
