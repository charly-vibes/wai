## 1. Artifact Locking
- [ ] 1.1 Add `lock` field to `PipelineStep` TOML schema
- [ ] 1.2 Implement SHA-256 hash computation for artifact files (LF-normalized)
- [ ] 1.3 Implement `.lock` sidecar file write (TOML format, run-ID in filename)
- [ ] 1.4 Implement `wai pipeline lock` command (lock current step's artifacts)
- [ ] 1.5 Integrate locking into `wai pipeline next` when `lock = true`
- [ ] 1.6 Error when locking a step with zero artifacts
- [ ] 1.7 Write tests: lock creation, hash correctness, duplicate lock prevention, zero-artifact error

## 2. Artifact Verification
- [ ] 2.1 Implement `wai pipeline verify` command (recompute LF-normalized hashes, report mismatches)
- [ ] 2.2 Integrate verification into `wai doctor` pipeline checks
- [ ] 2.3 Write tests: valid lock passes, tampered artifact fails, missing lock warns

## 3. Addenda Support
- [ ] 3.1 Add `--corrects=<path>` flag to `wai add` for creating addenda
- [ ] 3.2 Auto-tag addenda with `pipeline-addendum:<step-id>` and record `corrects` in frontmatter
- [ ] 3.3 Warn when `--corrects` targets an unlocked artifact (suggest editing directly)
- [ ] 3.4 Include addenda in `wai pipeline status` output for affected steps
- [ ] 3.5 Write tests: addendum creation, tag resolution, unlocked-target warning, status display

## 4. Coverage Gate
- [ ] 4.1 Add `CoverageGate` struct to `StepGate` (new tier between procedural and oracle)
- [ ] 4.2 Add `[steps.gate.coverage]` TOML parsing with `require_input_manifest` field
- [ ] 4.3 Implement coverage check: verify agent produced coverage manifest artifact (type=review, tag=coverage-manifest:<step-id>)
- [ ] 4.4 Block `wai pipeline next` when coverage gate unsatisfied
- [ ] 4.5 Write tests: gate blocks without manifest, passes with manifest

## 5. Pipeline TOML Schema Update
- [ ] 5.1 Update pipeline TOML parsing for new fields (lock on PipelineStep, coverage on StepGate)
- [ ] 5.2 Update `wai pipeline validate` to check new fields
- [ ] 5.3 Update documentation / managed block with new gate options
