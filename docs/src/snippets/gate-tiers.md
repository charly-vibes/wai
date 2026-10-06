<!-- Shared snippet: pipeline gate tiers. Included by docs/src/concepts/pipelines.md
     and available for reuse (e.g. commands docs via wai-fvhv.105). Keep this
     table in sync with src/commands/pipeline/gates.rs gate evaluation order. -->

| Tier | Type | Purpose |
|------|------|---------|
| 1 | **Structural** | Verify the step produced expected outputs (artifact count/type) |
| 2 | **Procedural** | Verify the validation process was followed (reviews exist, verdicts pass) |
| 3a | **Oracle** | Domain-specific machine-verifiable checks (user-written scripts) |
| 3b | **Approval** | Forced human checkpoint |
