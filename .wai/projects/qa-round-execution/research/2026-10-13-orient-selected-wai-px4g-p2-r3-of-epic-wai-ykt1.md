---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-13-wai-px4g, pipeline-step:orient-and-claim]
---

ORIENT: selected wai-px4g (P2, R3 of epic wai-ykt1); why: next ready child; depends only on spec, independent of R2 (design sequencing says parallelizable). Mechanism per design.md Decision 3: evidence contract enforced by the verify step's oracle gate — the template ships an evidence-check oracle on the review artifact. wai-979g verifier convention produces review artifacts. Files: src/templates/oracles/verify-evidence.sh (new), src/commands/pipeline/setup.rs (bundled_oracles_for), src/templates/epic-orchestrator.toml (verify step oracle + prompt evidence rules + brief step evidence requirements).
