# Pipelines Context

## Pipeline

**Definition:** An ordered, multi-step workflow defined in TOML under `.wai/resources/pipelines/`. Pipelines structure complex tasks (like research) into sequential steps with validation gates between them.

**Anti-terms:** Do not use "workflow" or "process" — "pipeline" is the canonical term.

**Related:** Pipeline step, Pipeline gate, Pipeline run

---

## Pipeline step

**Definition:** A single unit within a pipeline. Each step has a name, description, and optional gate conditions that must pass before advancing. Steps are completed in order.

**Anti-terms:** Do not call it a "phase" — phases belong to projects; steps belong to pipelines.

**Related:** Pipeline, Pipeline gate

---

## Pipeline gate

**Definition:** A condition that must be satisfied before advancing to the next pipeline step. Gate tiers: `structural` (artifact counts), `procedural` (review verdicts), `oracle` (oracle scripts), `approval` (human checkpoints).

**Anti-terms:** Do not use "checkpoint" — "gate" is the canonical term.

**Related:** Pipeline step, Oracle script, Review artifact

---

## Pipeline run

**Definition:** An active execution of a pipeline, scoped to a topic. Created with `wai pipeline start` and advanced with `wai pipeline next`. State is tracked so runs can resume across sessions.

**Anti-terms:** Do not say "pipeline instance" or "pipeline execution" — "pipeline run" is the canonical term.

**Related:** Pipeline, Session
