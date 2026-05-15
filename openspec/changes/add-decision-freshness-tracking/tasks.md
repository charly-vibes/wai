## 1. Specification
- [ ] 1.1 Add new `decision-freshness` capability spec covering frontmatter fields, sidecar format, drift detection algorithm, and stale-artifact reporting
- [ ] 1.2 Modify `cli-core` spec to add `wai artifacts stale` subcommand and `wai add --tracks` flag
- [ ] 1.3 Modify `research-management` spec to define `tracks` and `decision_point` frontmatter fields and their semantics
- [ ] 1.4 Modify `context-suggestions` spec to add stale-artifact surfacing in `wai status`

## 2. Design
- [ ] 2.1 Define sidecar file format (`<artifact>.fresh.lock`) and comparison algorithm (mtime prefilter → SHA-256)
- [ ] 2.2 Define glob expansion rules and scope of freshness scan (research, design, plan artifact types only)
- [ ] 2.3 Define machine-readable output schema for `wai artifacts stale --json`

## 3. Implementation
- [ ] 3.1 Add `src/freshness.rs`: sidecar read/write helpers, mtime+hash comparison, glob expansion
- [ ] 3.2 Add `src/commands/artifacts/` module with `stale` subcommand (human + JSON output)
- [ ] 3.3 Extend `src/commands/add.rs` to accept `--tracks` flag and write `tracks` into frontmatter
- [ ] 3.4 Extend `src/commands/status.rs` to call freshness scanner and surface stale artifacts in suggestions
- [ ] 3.5 Register `wai artifacts` in the CLI dispatcher

## 4. Tests (TDD — write failing tests first)
- [ ] 4.1 `tests/freshness_test.rs`: unchanged file → no stale signal; changed file → stale signal; missing tracked path → stale signal
- [ ] 4.2 `tests/artifacts_test.rs`: `wai artifacts stale` on clean workspace → empty output; after tracked-file change → artifact listed; `--json` output is valid JSON
- [ ] 4.3 `tests/add_test.rs`: `wai add research --tracks src/foo.rs` writes `tracks` field to artifact frontmatter

## 5. Validation
- [ ] 5.1 Run `openspec validate add-decision-freshness-tracking --strict`
- [ ] 5.2 Resolve all validation issues before requesting approval
