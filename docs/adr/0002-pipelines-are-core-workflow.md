# ADR 0002: Pipelines Are Core Workflow

## Status

Accepted (2026-10-19)

## Context

wai's product surface combines workflow management (pipelines, gates, run
state), documentation capture (way artifacts, decisions), session continuity
(prime, renew), repository auditing (doctor, way), and optional integrations
(plugins, genesis). A QA sweep (epic wai-fvhv) flagged the ambiguity:
**are pipelines core workflow, or an optional advanced feature?**

The ambiguity has concrete costs:

- AGENTS.md carries hand-written prose encoding the per-ticket discipline
  (`TDD → ro5u → fix → commit`) *in parallel* to the machine-enforced
  pipeline gates. The prose applies to every session; gates only fire
  during an active run — the two mechanisms were drifting apart.
- `wai doctor`'s `check_pipeline_utilization` had to heuristically detect
  "manual-cycle prose" and warn about it (GH #20) — a symptom of the same
  unresolved positioning.
- Scope drift risk: without a declared boundary, new features attach to
  whichever subsystem is nearest.

Evidence for pipelines being load-bearing, not optional:

- 83+ audited pipeline runs across 9 repos; epic-orchestrator shipped and
  battle-tested through multiple release trains (v2026.10.4–v2026.10.9).
- All builtin templates encode the TDD→red→green→refactor→review discipline
  as structural gates (`src/templates/tdd-ro5.toml`), tests-pass oracles,
  review requirements (`require_review`, `max_critical=0`), and approval
  gates — enforcement that prose alone cannot provide.
- The value proposition (context recovery ≤2 min) is delivered *through*
  structured workflow artifacts, which pipelines create and gates verify.

## Decision

**Pipelines are core workflow.** wai is, first and foremost, a workflow
manager. Every ticket follows the pipeline-encoded cycle
(`TDD → ro5u → fix → commit`) via `wai pipeline start tdd-ro5` (or the
appropriate specialization: `epic-autonomy-tdd-ro5`, `epic-orchestrator`,
`scientific-research`).

Consequences of this boundary:

- **Manual-cycle prose points to pipelines, it does not duplicate them.**
  AGENTS.md-style instructions must direct agents to
  `wai pipeline start`, never restate the cycle as bare prose (wai-2am8).
- **Gate enforcement is the single source of discipline.** Where prose and
  gates disagree, gates win. Prose that contradicts gate behavior is a bug.
- **Scope boundary:** documentation capture, session continuity, and
  repository auditing remain in scope as *supporting* surfaces; optional
  integrations (plugins) stay optional and must not accrete workflow-core
  behavior.
- **New workflow features must ship with a pipeline template and gates**, not
  prose instructions alone.

## Alternatives Considered

- **Pipelines as optional advanced feature** — rejected: the discipline
  (TDD, review, oracles) is the product's differentiator; making it optional
  would split every agent session into "disciplined" and "ad-hoc" modes with
  no enforcement in the latter.
- **Leave behavior unchanged, document boundaries only** — rejected: the
  prose-vs-gates duplication is the live defect (GH #20); documentation-only
  resolution leaves the drift in place.
- **Narrow the scope, remove pipelines** — rejected: contradicts observed
  usage and the shipped gate architecture.

## Follow-up Work

- wai-2am8 slice 2: redirect the manual-cycle prose in AGENTS.md to
  `wai pipeline start tdd-ro5` (in flight).
- Remaining scope questions of epic wai-fvhv proceed under this boundary:
  wai-fvhv.84 (workflow vs repo-hygiene auditing), wai-fvhv.93 (plugin
  scope vs maintenance budget), wai-fvhv.94 (tutorial audience).
