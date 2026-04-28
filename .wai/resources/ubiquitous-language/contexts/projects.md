# Projects & Phases Context

## Project

**Definition:** An active body of work with a defined goal and an end date. Tracked under `.wai/projects/` with a TOML descriptor. Moves through phases from research to archive.

**Anti-terms:** Do not use "task" or "ticket" — those are beads issues, not wai projects.

**Related:** Phase, PARA, Artifact

---

## Phase

**Definition:** One of six lifecycle stages a project moves through: `research → design → plan → implement → review → archive`. Each phase shapes what artifacts are expected and what work is appropriate.

**Anti-terms:** Do not use "stage" or "step" — "phase" is the canonical term.

**Related:** Project, Artifact, Pipeline

---

## PARA

**Definition:** An organizational system with four categories: **P**rojects (active, goal-bound), **A**reas (ongoing responsibilities), **R**esources (reference material), **A**rchives (completed items). Wai maps its own directory layout to PARA.

**Anti-terms:** Do not abbreviate individual categories — say "Projects area" not "the P".

**Related:** Project

---

## Archive (phase)

**Definition:** The final phase of a project lifecycle. Completed projects move to `.wai/archives/` via `wai archive`. Distinct from the PARA "Archives" category, which is the broader concept.

**Anti-terms:** Do not say "closed" for a project — "archived" is the correct term. "Closed" applies to beads issues, not wai projects.

**Related:** Phase, PARA
