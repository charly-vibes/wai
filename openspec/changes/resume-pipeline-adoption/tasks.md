# Tasks: resume-pipeline-adoption

## 1. Test the prime suggestion (red)

- [x] 1.1 Write failing test: `wai prime` with active pipeline run outputs
      resume-the-orchestration line
- [x] 1.2 Implement prime suggestion
- [x] 1.3 Run `cargo test` — verify green

## 2. Test the sync breadcrumb (red)

- [ ] 2.1 Write failing test: `bd sync` output includes resume-the-orchestration
- [ ] 2.2 Implement sync breadcrumb
- [ ] 2.3 Run `cargo test` — verify green

## 3. Authoritative over epics flow (red)

- [ ] 3.1 Write failing test: prime output has no epics-flow suggestion when
      pipeline run is active
- [ ] 3.2 Implement the precedence
- [ ] 3.3 Run `cargo test` — verify green

## 4. Validate (green)

- [ ] 4.1 `openspec validate resume-pipeline-adoption --strict`
- [ ] 4.2 Manual verification on the wai repo itself (dogfood)
