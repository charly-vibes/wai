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

Place custom pipelines in `.wai/resources/pipelines/` and run `wai init --update` to refresh the managed block in CLAUDE.md.

## Pipeline Gates

Steps can optionally define **gates** — conditions that must be satisfied before advancing. Gates enforce validation at step boundaries and are checked when you run `wai pipeline next`.

### Gate tiers

Gates evaluate in order. The first failure blocks advancement — `wai pipeline next` prints the failing gate and what's missing, and blocks until the condition is satisfied.

| Tier | Type | Purpose |
|------|------|---------|
| 1 | **Structural** | Verify the step produced expected outputs (artifact count/type) |
| 2 | **Procedural** | Verify the validation process was followed (reviews exist, verdicts pass) |
| 3a | **Oracle** | Domain-specific machine-verifiable checks (user-written scripts) |
| 3b | **Approval** | Forced human checkpoint |

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

## Command reference

| Command | Description |
|---------|-------------|
| `wai pipeline list` | List all available pipelines |
| `wai pipeline show <name>` | View steps and gates for a pipeline |
| `wai pipeline start <name> --topic="..."` | Start a new run |
| `wai pipeline next` | Advance to the next step |
| `wai pipeline status <name>` | Show per-stage run status |
| `wai pipeline gates <name>` | Show gate requirements |
| `wai pipeline check` | Dry-run gate evaluation |
| `wai pipeline approve` | Human approval for current step |
| `wai pipeline validate <name>` | Validate pipeline TOML |

## See Also

- [Project Phases](./phases.md) — the phase system pipelines build on
- [Plugin System](./plugins.md#beads) — beads issue tracking used in task decomposition
- [Commands Reference](../commands.md) — full command documentation
