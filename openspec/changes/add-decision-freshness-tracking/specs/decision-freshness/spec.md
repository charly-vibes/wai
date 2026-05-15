# Capability: decision-freshness

## Purpose

Detect when code described by a wai decision artifact has changed since that
artifact was last evaluated, and surface that staleness to agents and humans.

## ADDED Requirements

### Requirement: Artifact tracks frontmatter field

Decision artifacts of type `research`, `design`, and `plan` SHALL support an
optional `tracks` frontmatter field listing repo-relative file paths or glob
patterns that the artifact describes. Artifacts without `tracks` MUST be ignored
by all freshness checks. An optional `decision_point` slug groups related
artifacts for future rollup phases but SHALL be ignored in v1 processing.

#### Scenario: Author adds tracks field

- **GIVEN** a research artifact with `tracks: [src/commands/status.rs]`
- **WHEN** the freshness scanner processes the artifact
- **THEN** `src/commands/status.rs` is resolved relative to the repo root
- **AND** included in the comparison set

#### Scenario: Artifact without tracks is ignored

- **GIVEN** a research artifact with no `tracks` field
- **WHEN** the freshness scanner runs
- **THEN** the artifact produces no stale signal

### Requirement: Freshness sidecar format

For each decision artifact with `tracks`, wai SHALL store a sidecar file
`<artifact>.fresh.lock` adjacent to the artifact in TOML format:

```toml
artifact = "<artifact-filename>"
verified_at = "<ISO-8601 timestamp>"
[[tracked]]
path = "src/commands/status.rs"
mtime = 1747060000
hash = "sha256:<hex>"
```

The sidecar is written when the artifact is created with `--tracks` or when
a future `wai artifacts refresh` is run. Comparison: expand globs → compare
mtime (cheap prefilter) → if mtime differs, recompute SHA-256 → if hash
changed, mark stale. A `tracks` path absent on disk counts as stale.

#### Scenario: Tracked file is unchanged

- **GIVEN** a sidecar records mtime=T and hash=H for `src/foo.rs`
- **WHEN** the scanner runs and `src/foo.rs` still has mtime=T
- **THEN** the artifact is NOT stale

#### Scenario: Tracked file mtime changed but content identical

- **GIVEN** a sidecar records mtime=T and hash=H for `src/foo.rs`
- **WHEN** the scanner runs and `src/foo.rs` has mtime=T+1 but same content
- **THEN** the scanner recomputes the hash
- **AND** the artifact is NOT stale (hash matches)

#### Scenario: Tracked file content changed

- **GIVEN** a sidecar records mtime=T and hash=H for `src/foo.rs`
- **WHEN** the scanner runs and `src/foo.rs` has different content
- **THEN** the artifact IS stale

#### Scenario: Tracked file is missing

- **GIVEN** a sidecar records `src/bar.rs`
- **WHEN** the scanner runs and `src/bar.rs` does not exist
- **THEN** the artifact IS stale

### Requirement: Freshness scanner scope

The freshness scanner SHALL search `.wai/projects/**/{research,design,plans}/`
for artifacts with `tracks` frontmatter. It MUST NOT scan `reviews/`,
`handoffs/`, or `corrections/`.

#### Scenario: Scanner finds artifacts in correct directories

- **GIVEN** a research artifact in `.wai/projects/my-proj/research/` with `tracks`
- **WHEN** the scanner runs
- **THEN** the artifact is included in the staleness check

#### Scenario: Handoff artifacts are excluded

- **GIVEN** an artifact in `.wai/projects/my-proj/handoffs/` with `tracks`
- **WHEN** the scanner runs
- **THEN** the artifact is NOT included in the staleness check

### Requirement: Machine-readable stale-artifact report

`wai artifacts stale` SHALL emit a list of stale artifacts. With `--json` the
output MUST be a JSON object conforming to:

```json
{
  "stale": [
    {
      "artifact": ".wai/projects/my-proj/research/2026-05-12-status-design.md",
      "decision_point": "status-freshness",
      "changed_paths": ["src/commands/status.rs"]
    }
  ],
  "untracked": [],
  "clean": 5,
  "stale_count": 1
}
```

Without `--json`, a human-readable summary lists each stale artifact and its
changed paths. Exit code is 0 in all cases (stale is advisory, not an error).

#### Scenario: No stale artifacts

- **GIVEN** no tracked file has changed since its sidecar was written
- **WHEN** `wai artifacts stale` runs
- **THEN** exit code is 0
- **AND** output indicates all artifacts are current

#### Scenario: One stale artifact

- **GIVEN** one tracked file has changed
- **WHEN** `wai artifacts stale` runs
- **THEN** the artifact path and changed paths appear in output
- **AND** exit code is 0

#### Scenario: JSON output is valid

- **GIVEN** any workspace state
- **WHEN** `wai artifacts stale --json` runs
- **THEN** output is valid JSON conforming to the schema above

### Requirement: Artifacts with tracks but no sidecar are untracked not stale

If an artifact has `tracks` but no `.fresh.lock` sidecar exists, it SHALL
appear in an `"untracked"` field in JSON output and MUST NOT appear in the
`"stale"` list. This distinguishes "never evaluated" from "evaluated and
now outdated."

#### Scenario: New artifact with tracks but no sidecar

- **GIVEN** a research artifact with `tracks` and no `.fresh.lock`
- **WHEN** `wai artifacts stale` runs
- **THEN** the artifact appears in the `"untracked"` list
- **AND** does NOT appear in the `"stale"` list
