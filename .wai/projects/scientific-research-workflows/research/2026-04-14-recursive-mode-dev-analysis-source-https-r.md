# recursive-mode.dev Analysis

## Source
https://recursive-mode.dev/ — "A structured, file-backed workflow for coding agents"
Open-source skill package by try-works. Predates Factory.ai Missions.

## What It Is
A file-backed workflow that replaces conversational context with repository-anchored artifacts. Each task becomes a "run" stored in `/.recursive/run/<run-id>/` with 8 sequential phases, each producing a locked markdown artifact.

## Phase Architecture

| Phase | Artifact | Audited | Purpose |
|-------|----------|---------|---------|
| 0a | 00-worktree.md | No | Git worktree isolation, baseline diff |
| 0b | 00-requirements.md | No | In-scope requirements (R1, R2...) |
| 1 | 01-as-is.md | Yes | Ground current state against requirements |
| 1.5 | 01.5-root-cause.md | Yes | Debug-only root cause analysis |
| 2 | 02-to-be-plan.md | Yes | Implementation strategy |
| 3 | 03-implementation-summary.md | Yes | Execute plan, TDD evidence |
| 3.5 | 03.5-code-review.md | Yes | Optional multi-agent code review |
| 4 | 04-test-summary.md | Yes | Test suite results |
| 5 | 05-manual-qa.md | No | Human/agent/hybrid QA |
| 6 | 06-decisions-update.md | Yes | Append to global DECISIONS.md |
| 7 | 07-state-update.md | Yes | Update global STATE.md |
| 8 | 08-memory-impact.md | Yes | Update memory plane |

## Key Concepts

### Artifact Locking
- Every artifact has a lifecycle: DRAFT → Audit → Lock
- Locking writes SHA-256 hash (`LockHash`), timestamp (`LockedAt`), and `Status: LOCKED`
- Locked artifacts are immutable — verified by recomputing hashes
- Must use `recursive-lock` script; manual edits break integrity

### Audit Gates (Coverage + Approval)
Audited phases require two gates before locking:
- **Coverage Gate**: proves artifact addresses ALL relevant inputs including addenda
- **Approval Gate**: verifies readiness criteria (consistency, no missing sections, no blockers)
- Neither passes without prior audit; order is: Audit PASS → Coverage PASS → Approval PASS → Lock

### Addenda System
When later phases discover gaps in locked earlier phases:
- You do NOT edit the locked phase
- You create an addendum file in `run/<run-id>/addenda/`
- Downstream phases must list addenda as inputs and reconcile them
- Two types: stage-local (extends current) and upstream-gap (compensates for prior phase gaps)

### Memory Plane
Structured persistent knowledge with freshness tracking:
- Categories: domains/, patterns/, incidents/, episodes/, skills/
- Status lifecycle: CURRENT → SUSPECT → STALE → DEPRECATED
- Each doc declares `Owns-Paths` and `Watch-Paths` (code paths it's authoritative for)
- When a run touches watched paths, relevant docs auto-downgrade to SUSPECT
- Phase 8 (Memory Impact) is responsible for updating the memory plane

### Global Ledgers
- `STATE.md` — current codebase truth (updated Phase 7)
- `DECISIONS.md` — cumulative decision records (updated Phase 6)
- Both read at start of every run, creating a feedback loop

### TDD Subskill
Enforces RED-GREEN-REFACTOR with evidence artifacts:
- RED: must document failure output before writing production code
- GREEN: minimal code to pass, no speculative features
- REFACTOR: never introduce new behavior
- Pragmatic mode requires explicit declaration and compensating evidence
- Phase 3 artifact must include TDD Compliance Log

## Comparison with wai

### Overlapping Concerns
| Concern | wai | recursive-mode |
|---------|-----|---------------|
| Why decisions were made | wai artifacts (research/design/plan) | DECISIONS.md global ledger |
| Session continuity | wai close/prime handoff | File-backed runs, read fresh |
| Pipeline phases | wai pipeline (configurable) | Fixed 8-phase sequence |
| Memory | beads: `bd remember`; wai: resources + reflections | Structured memory plane with freshness |
| Phase gates | Pipeline gate checks | Formal Coverage + Approval gates |
| TDD | CLAUDE.md principle | Enforced subskill with evidence |

### What recursive-mode has that wai lacks
1. **Artifact integrity** — SHA-256 locking prevents post-hoc edits
2. **Formal audit gates** — Coverage + Approval before advancement
3. **Addenda system** — Forward-only corrections preserving history
4. **Memory freshness** — CURRENT/SUSPECT/STALE tied to code paths
5. **Global state file** — Single STATE.md truth document
6. **TDD evidence enforcement** — RED artifacts required before GREEN

### What wai has that recursive-mode lacks
1. **Issue tracking** (beads) — dependencies, priorities, blocking
2. **Change proposals** (openspec) — spec-driven development
3. **LLM oracle** (wai why) — explain decisions from context
4. **Configurable pipelines** — not locked to 8 phases
5. **Plugin architecture** — extensible ecosystem
6. **PARA organization** — structured knowledge management
7. **Search** — cross-artifact search with tags

## Integration Opportunities (ranked by ROI)

### High ROI
1. **Artifact locking for pipeline steps** — `wai pipeline lock` + `wai pipeline verify` commands. SHA-256 hash pipeline step outputs, mark immutable. Cheap to build, high auditability gain.
2. **Audit gates on pipeline steps** — Extend pipeline step config with optional coverage_gate and approval_gate. Steps with gates can't advance until agent proves coverage + readiness.

### Medium ROI
3. **Memory freshness model** — Add Owns-Paths/Watch-Paths to wai resources. When code changes touch watched paths, flag resources for review.
4. **Addenda for pipeline artifacts** — Forward-only corrections instead of editing past outputs. Especially valuable for scientific-research pipeline.
5. **STATE.md + DECISIONS.md consolidation** — Merge reflections + handoffs into single ground-truth files.

### Lower ROI (wai already covers differently)
6. **TDD evidence in pipeline steps** — Specialized step type requiring failure artifacts. wai already has TDD in CLAUDE.md but doesn't enforce evidence.
7. **Fixed phase templates** — Offer recursive-mode's 8 phases as a built-in pipeline formula. Easy to do but wai's flexibility may be preferred.

## Sources Consulted
- https://recursive-mode.dev/ — landing page
- https://recursive-mode.dev/llms.txt — documentation index
- https://recursive-mode.dev/concepts/phases.md — phase reference
- https://recursive-mode.dev/concepts/artifacts.md — artifact lifecycle and locking
- https://recursive-mode.dev/concepts/memory.md — memory plane architecture
- https://recursive-mode.dev/concepts/workflow-overview.md — run model and feedback loops
- https://recursive-mode.dev/installation.md — scaffold structure
- https://recursive-mode.dev/guides/auditing-phases.md — audit gates and addenda
- https://recursive-mode.dev/subskills/recursive-tdd.md — TDD enforcement subskill
- https://recursive-mode.dev/scripts/lock-and-verify.md — locking and verification scripts
