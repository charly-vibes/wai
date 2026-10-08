# Pipelines

> **Why pipelines?** Some tasks are too complex or error-prone for a single pass. When an AI agent tries to do research, validation, and synthesis all at once, errors compound — context drifts, assumptions go unchecked, and the final output is built on shaky foundations. Pipelines break these tasks into discrete steps with validation gates between them, so each step is verified before the next one begins.

Pipelines are ordered, multi-step workflows that guide you through a structured process. Each step provides a prompt with deliverables and instructions, and you advance through steps sequentially.

Pipelines are useful when a task is too large or too error-prone for a single pass — for example, research that spans many subtasks, or a process where intermediate validation prevents compounding errors. If your work fits naturally into wai's [project phases](./phases.md) without extra structure, you probably don't need a pipeline.

**Prerequisite:** an initialized wai workspace (`wai init`). See [Quick Start](../quick-start.md).

## Overview

```bash
wai pipeline list                                # List available pipelines
wai pipeline show <name>                         # View steps and gates
wai pipeline start <name> --topic="<objective>"  # Start a run
wai pipeline next                                # Mark current step done, move to next
```

Pipelines are defined as TOML files in `.wai/resources/pipelines/`. When you run `wai init`, built-in templates are installed into this directory. You can modify your local copy without affecting the built-in defaults.

## Creating Custom Pipelines

Pipelines are TOML files with a simple structure:

```toml
[pipeline]
name = "my-workflow"
description = "What this pipeline does"

[pipeline.metadata]
when = "When to use this pipeline"
skills = ["skill-a", "skill-b"]    # Optional: skills used by steps

[[steps]]
id = "step-one"
prompt = """
{topic}: Instructions for this step.

Deliverables:
- What to produce

Record findings: `wai add research "..."`
Advance: `wai pipeline next`
"""

[[steps]]
id = "step-two"
prompt = """
{topic}: Instructions for this step.
"""
```

The `{topic}` placeholder is replaced with the `--topic` value when the run starts.

Place custom pipelines in `.wai/resources/pipelines/` and run `wai init` to refresh the managed block in CLAUDE.md.

## Pipeline Gates

Steps can optionally define **gates** — conditions that must be satisfied before advancing. Gates enforce validation at step boundaries and are checked when you run `wai pipeline next`.

### Gate tiers

Gates evaluate in order. The first failure blocks advancement — `wai pipeline next` prints the failing gate and what's missing, and blocks until the condition is satisfied.

{{#include ../snippets/gate-tiers.md}}

### Configuring gates in TOML

Gate sections go inside the `[[steps]]` entry they belong to. Here is a complete step with all four gate tiers:

```toml
[[steps]]
id = "generate-validate-accrue"
prompt = """
{topic}: Execute subtasks with intermediate validation.
...
"""

[steps.gate.structural]
min_artifacts = 1
types = ["research"]

[steps.gate.procedural]
require_review = true
review_verdict = "pass"
max_critical = 0
max_high = 0

[[steps.gate.oracles]]
name = "dimensional-analysis"
description = "Verify dimensional consistency"
timeout = 300

[steps.gate.approval]
required = true
message = "Review all accrued artifacts before advancing"
```

### Oracle scripts

Oracles are user-written scripts that perform domain-specific checks:

- Location: `.wai/resources/oracles/<name>.sh` or `.wai/resources/oracles/<name>.py`
- Contract: receives the artifact path as an argument, exits 0 on pass, writes failure reason to stderr
- Timeout: default 30 seconds, configurable per oracle

### Gate commands

```bash
wai pipeline gates <name>              # Show gate requirements per step
wai pipeline check                     # Dry-run gates without advancing
wai pipeline approve                   # Human approval for current step
wai pipeline validate <name>           # Validate pipeline TOML and gate config
```

## Built-in: Scientific Research Pipeline

The `scientific-research` pipeline is designed for frontier-level theoretical or computational research where an LLM generates mathematical derivations, proofs, or data analysis across many subtasks and a human supervisor needs systematic validation.

It addresses three common failure modes in AI-assisted research:

- **Context drift** — the research gradually shifts away from the original problem
- **Sycophancy** — the model confirms what it thinks you want rather than what's true
- **Hallucinated data smoothing** — gaps are filled with plausible but unverified claims

### Starting a run

```bash
wai pipeline start scientific-research --topic="Your research objective here"
```

This creates a pipeline run and places you at step 1. Each step provides a detailed prompt explaining what to do, what deliverables to produce, and how to record them.

### Steps

The pipeline has five framing steps (Describe, Diagnose, Delimit, Direction, Decompose), a core execution loop (Generate-Validate-Accrue), and two closing steps (Synthesize, Final Review).

Every step ends with recording artifacts and advancing:

```bash
wai add research "findings..."   # or design/plan as appropriate
wai pipeline next
```

#### 1. Describe

Capture the research problem **neutrally**. Focus on observed symptoms, signals, metrics, boundary conditions, and known constraints. Do not propose solutions or hypotheses yet.

**Deliverables:** neutral problem statement, known constraints, data sources, success criteria.

#### 2. Diagnose

Generate and test **competing hypotheses** — at least three. For each, state its predictions and identify confirming/refuting evidence. Use negative progress (ruling out false leads) rather than converging too early.

**Deliverables:** hypothesis map, evidence for/against each, eliminated paths.

#### 3. Delimit

Define explicit **boundaries and scope**. This is the primary defense against context drift. Document what is in scope, what is explicitly out, constraints, and assumptions taken as given.

**Deliverables:** scope document (referenced throughout remaining steps).

#### 4. Direction

Select the research approach using a **decision matrix**. Compare surviving hypotheses against feasibility, rigor, falsifiability, and compute cost. Include a null hypothesis baseline. Document what would cause you to revisit the decision.

**Deliverables:** decision matrix, selected approach with justification.

#### 5. Decompose

Break the selected approach into a **master task tree** of discrete, independently verifiable subtasks. Each subtask gets a unique ID, input dependencies, expected output format, and validation criteria.

**Deliverables:** task tree (created as [beads](./plugins.md#beads) issues with dependencies).

```bash
bd create --title="<subtask>" --description="<details>" --type=task
bd dep add <blocked> <blocker>
wai add plan "task decomposition..."
```

---

#### 6. Generate-Validate-Accrue (core loop)

Execute each subtask from the task tree. For **every** subtask:

1. **Generate** — produce the derivation, code, analysis, or argument
2. **Validate** — run `/ro5` (Rule of 5, a wai review skill) on the artifact:
   - *Accuracy* — verify facts, citations, mathematical correctness
   - *Completeness* — check for skipped steps, missing edge cases
   - *Clarity* — ensure unambiguous terminology and notation
   - *Actionability* — confirm output is usable by the next subtask
   - *Integration* — check consistency with all previously accrued artifacts
3. **Accrue** — if validation passes, commit the artifact:
   ```bash
   wai add research "<subtask findings>"
   bd close <subtask-id>
   ```

**Critical rules:**
- Do not skip validation. Do not accrue unvalidated artifacts.
- If a subtask contradicts a previously accrued artifact, **halt** and surface the contradiction for human review.
- Fix and re-validate before accruing if validation fails.

---

#### 7. Synthesize

Assemble the final output from all accrued artifacts. Check for logical flow, notation consistency, gap detection, and whether boundary conditions from the delimit step are satisfied.

**Deliverables:** coherent final output (paper, report, derivation).

#### 8. Final Review

Run `/ro5` on the complete synthesized output. Verify:
- The result satisfies success criteria from step 1
- The scope document from step 3 was respected
- The decision matrix from step 4 is still valid
- All beads subtasks are closed

If critical issues are found, route back to step 6 for the specific subtask that introduced the error.

### Visual overview

```
┌─────────┐   ┌───────────┐   ┌─────────┐   ┌───────────┐   ┌───────────┐
│ Describe │──▶│ Diagnose  │──▶│ Delimit │──▶│ Direction │──▶│ Decompose │
│          │   │           │   │         │   │           │   │           │
│ Problem  │   │ Hypotheses│   │ Scope   │   │ Decision  │   │ Task tree │
└─────────┘   └───────────┘   └─────────┘   └───────────┘   └─────┬─────┘
                                                                   │
                                                                   ▼
┌──────────────┐   ┌─────────────┐   ┌─────────────────────────────────┐
│ Final Review │◀──│ Synthesize  │◀──│ Generate → Validate → Accrue    │
│              │   │             │   │         (repeat per subtask)     │
│ Quality gate │   │ Assembly    │   └─────────────────────────────────┘
└──────────────┘   └─────────────┘
```

## Built-in: TDD + Rule of 5 Pipeline

The `tdd-ro5` pipeline is designed for autonomous implementation work where an agent should keep moving without routine confirmations, but must prove correctness before shipping.

Start it with:

```bash
wai pipeline start tdd-ro5 --topic="Your feature or bug fix"
```

The pipeline uses nine gated steps:

1. **Orient** — run `wai prime`, `wai status`, available work discovery, and `wai search` before editing.
2. **Plan** — record desired behavior, out of scope, test cases, and verification commands.
3. **Red** — write failing tests and record expected failure evidence.
4. **Green** — implement the minimum code and pass relevant tests.
5. **Refactor** — tidy safely while tests remain green.
6. **RO5U Review** — run Universal Rule of 5 review and record findings without fixing yet.
7. **Fix Review** — address review findings and keep tests green.
8. **Quality Ledger** — record Changed, Verified, Review, Risks, and Next.
9. **Ship Close** — inspect the diff, update linked workflow tools, commit atomically, and run `wai close`.

Review and fixing are intentionally separate steps: the review step observes and records evidence; the fix step applies bounded remediation. Push/release/deploy remains outside the default pipeline unless explicitly authorized by project policy or user instruction.

## Built-in: Epic-Orchestrator Pipeline

The `epic-orchestrator` pipeline is designed for epic execution where a **lead** agent orchestrates only — one ready child ticket per run — and **subagents** do the implementation. It encodes the orchestrator + subagents pattern as enforceable steps instead of hoping agents follow prose: the lead's job is claim → gates → brief → spawn → verify → ship → STOP, never inline implementation.

### Starting a run

```bash
wai pipeline start epic-orchestrator --topic="<child ticket id>"
```

Each run handles exactly one child ticket. The next ticket starts a fresh run — the loop ends in an explicit STOP at the ticket boundary and never chains past it.

### Steps

1. **Claim** — orient (`wai prime`, `wai status`, `bd ready`) and claim exactly one ready, unblocked child issue atomically (`bd update <id> --claim`). Write the per-ticket state file (see below).
2. **Gates** — run the project's pre-work quality gates and establish a green test-suite baseline so later failures are attributable to this change only. No implementation yet.
3. **Brief** — write the subagent brief as a **committed repo artifact** at `.wai/projects/<project>/briefs/<slug>.md`, with mandatory `## Why` (delegation rationale), `## Completion criteria` (runnable commands only — exit 0 = done), and `## Spawn model` (model plus a context ceiling expressed as `≤ N%` of the actual model window; fixed token counts are invalid). Commit the brief before spawning so the subagent starts from a clean tree.
4. **Spawn** — spawn a named, resumable subagent session for the brief, streaming output to a known log path the orchestrator can tail. Spawn is a named session (retry is a fresh `-retry`-suffixed session, never an in-place resume).
5. **Verify** — verify before trust. With a second model family configured, a read-only verifier session (`subagent:<ticket>:verify`) cross-checks the implementor's report against reality: commits vs `git log`, report claims vs ticket-tracker state.
6. **Ship** — inspect the diff, commit atomically, update the ticket tracker and openspec tasks. Push only when project policy or user instruction explicitly authorizes it.
7. **STOP** — hard stop at the ticket boundary. Record the final checkpoint and start the next ticket as a new run.

Every step ends with recording an artifact (`wai add`), appending a `step id + sha` checkpoint to the state file, and `wai pipeline next`.

### Per-ticket state file

The claim step creates a durable state file at `.wai/projects/<project>/runs/<ticket>.state`, e.g.:

```yaml
ticket: wai-hr0w
branch: main
brief_path: .wai/projects/<project>/briefs/<slug>.md
step_history: []
- claim @ 52c6de8604ad86f87c4a6a3defe4800a23b3b1bf
- gates @ 52c6de8604ad86f87c4a6a3defe4800a23b3b1bf
- brief @ 939dae26e2b7aefa0e4200d09a41c4259029d4b4
```

It records the ticket id, the branch, the brief path (empty until the brief step), and a step history of `step id + sha` checkpoints appended before each advance.

**Retry and drift:** if the state file already exists at claim, the retry reads it and runs a drift check before resuming — it refuses to resume when HEAD advanced without a completed step accounting for the advance. A mid-run advance is expected (the implementor commits) and is never flagged as drift. Retries resume from the last completed gate, not from the beginning.

**Refusal conditions:**

- Spawn refuses to run without a **committed brief** at its recorded path.
- Spawn refuses to run without the **state file** — the spawn must be resumable through it.
- The retry-entry **drift check** refuses to resume a state file whose HEAD advanced without an accounting step.

### Single-model degradation

The primary verify path spawns a verifier from a *different model family*. Single-model setups degrade to **lead-side checks** instead of a spawned verifier: record the branch head at verify entry, diff `git log`/`git diff` against the report, run the repo's test suite, and check the ticket tracker. This degradation is **non-blocking** — the run may advance once the lead-side checks pass. A contradiction between the report and reality always blocks the run, degraded or not.

### Oracle requirement

The verify and ship steps carry a `tests-pass` oracle gate. `wai pipeline init epic-orchestrator` ships a generic `tests-pass` script (see [Oracle scripts](#oracle-scripts)) to `.wai/resources/oracles/`, so the gates resolve out of the box; if you remove it, advancing past those steps fails.

> Provenance: the evidence base and design proposal for this pipeline live in the orchestrator-tooling project (`~/.wai/projects/orchestrator-tooling/`); the openspec change is `openspec/changes/add-epic-orchestrator-template/`.

## Artifact Locking

Steps can declare `lock = true` to freeze their artifacts with SHA-256 hashes when you advance past them. This prevents accidental modification of validated work — once a step's artifacts are locked, any change will be caught by `wai pipeline verify` or `wai doctor`.

### How it works

When a step with `lock = true` completes (via `wai pipeline next`), wai computes a SHA-256 hash of each artifact tagged with that step and writes a `.lock` sidecar file alongside it. The sidecar records the hash, run ID, step ID, and timestamp.

You can also lock manually at any time:

```bash
wai pipeline lock      # Lock current step's artifacts now
```

### Verifying integrity

```bash
wai pipeline verify    # Check all locked artifacts across the workspace
```

Verify reports:
- **Missing artifacts** — a `.lock` file exists but the artifact was deleted
- **Hash mismatches** — an artifact was modified after locking

`wai doctor` also checks artifact lock integrity as part of its health checks.

### Correcting locked artifacts

Locked artifacts should not be edited directly. Instead, create an **addendum** — a new artifact that references the original:

```bash
wai add research --corrects=<path-to-locked-artifact> "correction details"
```

This preserves the audit trail: the original artifact stays intact and the addendum records what changed and why.

### Configuring lock in pipeline TOML

Add `lock = true` to any step definition:

```toml
[[steps]]
id = "synthesize"
lock = true
prompt = """
{topic}: Assemble findings into a coherent output.
"""
```

## Command reference

| Command | Description |
|---------|-------------|
| `wai pipeline list` | List all available pipelines |
| `wai pipeline show <name>` | View steps and gates for a pipeline |
| `wai pipeline start <name> --topic="..."` | Start a new run |
| `wai pipeline next` | Advance to the next step |
| `wai pipeline current` | Reprint the active step |
| `wai pipeline current --json` | Emit machine-readable active run context |
| `wai pipeline status` | Alias for machine-readable active run context |
| `wai pipeline gates <name>` | Show gate requirements |
| `wai pipeline check` | Dry-run gate evaluation |
| `wai pipeline approve` | Human approval for current step |
| `wai pipeline validate <name>` | Validate pipeline TOML |
| `wai pipeline lock` | Lock current step's artifacts (SHA-256) |
| `wai pipeline verify` | Verify integrity of all locked artifacts |

## See Also

- [Project Phases](./phases.md) — the phase system pipelines build on
- [Plugin System](./plugins.md#beads) — beads issue tracking used in task decomposition
- [Commands Reference](../commands.md) — full command documentation
