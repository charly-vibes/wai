## Context

Pipeline runs produce artifacts (research, plans, designs) as dated markdown
files. Currently nothing prevents an agent from silently editing a prior step's
output after moving to a later step. For scientific research workflows this is
a reliability problem — conclusions must trace back to unaltered evidence.

recursive-mode.dev solves this with SHA-256 locked artifacts and an addenda
system for forward-only corrections. We adapt their approach to wai's flexible
pipeline model rather than adopting their fixed 8-phase sequence.

## Goals / Non-Goals

- **Goal**: tamper-evident pipeline artifacts via SHA-256 hashing
- **Goal**: addenda as the correction mechanism for locked artifacts
- **Goal**: optional coverage and approval gates per pipeline step
- **Goal**: verification integrated into `wai doctor`
- **Non-goal**: mandatory locking for all pipelines (opt-in only)
- **Non-goal**: recursive-mode's full phase sequence (wai pipelines are flexible)
- **Non-goal**: memory freshness tracking (separate future work)

## Decisions

### Lock file format
Use a `.lock` sidecar file (TOML) alongside each locked artifact rather than
embedding lock fields in the markdown. This avoids modifying the artifact
content and simplifies hash verification.

```toml
# .wai/projects/<project>/research/2026-04-14-hypothesis.md.lock
artifact = "2026-04-14-hypothesis.md"
locked_at = "2026-04-14T15:30:00Z"
lock_hash = "sha256:abc123..."
pipeline_run = "scientific-research-2026-04-14-topic"
pipeline_step = "hypothesis"
```

**Why sidecar over inline**: recursive-mode embeds lock fields in the markdown
header, which means the hash must exclude those fields during verification
(content normalization). A sidecar avoids this — the hash covers the entire
file byte-for-byte. Simpler to implement, harder to break.

### Addenda as tagged artifacts
Addenda are regular wai artifacts with a `pipeline-addendum:<step-id>` tag
and a `corrects: <original-artifact-path>` field. They live alongside other
artifacts in the project directory, not in a separate addenda folder.

**Why**: keeps wai's existing artifact storage model. `wai search --tag
pipeline-addendum` finds them. Downstream steps reference them naturally.

### Gate definitions in pipeline TOML
The existing gate system uses a tiered struct (`StepGate`) with four tiers
evaluated in order: structural → procedural → oracle → approval. Coverage
is a new tier that slots between procedural and oracle:

```toml
[[steps]]
id = "hypothesis"
prompt = "..."
lock = true              # lock artifacts on advancement

[steps.gate.coverage]
require_input_manifest = true   # agent must list all inputs addressed
```

A **coverage manifest** is a wai artifact of type `review` tagged with
`coverage-manifest:<step-id>`. It lists each input artifact path (prior step
outputs + any addenda) with a one-line disposition: `addressed`, `deferred`,
or `N/A`.

The new `CoverageGate` struct fits into `StepGate` alongside the existing
tiers:

```rust
pub struct StepGate {
    pub structural: Option<StructuralGate>,
    pub procedural: Option<ProceduralGate>,
    pub coverage: Option<CoverageGate>,    // NEW
    pub oracles: Vec<OracleGate>,
    pub approval: Option<ApprovalGate>,
}
```

**Why a new tier**: the existing `ProceduralGate` checks *review* coverage
(does each artifact have a review?). The new `CoverageGate` checks *input*
coverage (did you address all prior step outputs + addenda?). These are
distinct concerns — review coverage is about quality, input coverage is
about completeness.

**Why not touch approval or oracle**: oracle gates and approval gates already
exist and work correctly. No changes needed.

### Addenda creation flow
Addenda are created via a `--corrects` flag on `wai add`:

```bash
wai add research --corrects=".wai/projects/my-proj/research/2026-04-14-hypothesis.md" \
  "Correction: boundary condition was wrong because..."
```

The CLI resolves the corrected artifact's pipeline step from its tags,
applies the `pipeline-addendum:<step-id>` tag, and records the `corrects`
path in artifact frontmatter. If the corrected artifact is not locked, the
CLI warns that the original can be edited directly instead.

### Lock timing
Locking happens as part of `wai pipeline next` when the step has
`lock = true` in its TOML config. The CLI computes hashes for all artifacts
tagged with the current step, writes sidecar `.lock` files, and then advances.
If the step has zero artifacts, locking fails with an error: "Cannot lock
step with no artifacts."

Alternatively, `wai pipeline lock` can be called manually at any time to lock
the current step's artifacts without advancing.

### Lock file paths
Lock sidecar files include the pipeline run ID to prevent collisions when
multiple runs produce same-named artifacts:

```
<artifact-path>.<run-id>.lock
```

For example: `2026-04-14-hypothesis.md.scientific-research-2026-04-14-topic.lock`

## Risks / Trade-offs

- **Sidecar clutter**: `.lock` files alongside artifacts. Mitigation: they're
  small (< 200 bytes) and `.gitignore`-able if the team prefers.
- **Hash brittleness**: line-ending normalization (CRLF vs LF) could break
  hashes. Mitigation: normalize to LF before hashing, both at lock time and
  verification time (same as recursive-mode's recommendation).
- **Addenda overhead**: agents must create new files instead of quick edits.
  This is the point — but it adds friction. Mitigation: only locked steps
  require addenda; unlocked steps allow normal edits.

## Open Questions

- Should `wai pipeline verify` be a standalone command or only run via
  `wai doctor`? Leaning toward both.
- Should lock files be committed to git? Probably yes for auditability, but
  some teams may prefer `.gitignore`.
