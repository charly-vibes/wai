# Change: Add ubiquitous language guidance to the wai way

## Why

AI-assisted development works best when domain terms are explicit, stable, and easy to load on demand. Today wai recommends general AI context and reusable skills, but it does not yet recommend a canonical ubiquitous-language resource or provide a built-in workflow for maintaining one. A single giant glossary file would also bloat context windows, so the guidance needs a progressive-disclosure structure instead of a monolith.

## What Changes

- Add a new `wai way` recommendation for a canonical ubiquitous-language resource
- Define a progressive-disclosure resource tree under `.wai/resources/ubiquitous-language/`
- Add a built-in `ubiquitous-language` skill template to scaffold maintenance workflows
- Mention ubiquitous-language resources in the managed block so agents consult the root index first, then open only relevant bounded-context files before inventing or reusing domain terminology

## Impact

- Affected specs:
  - `repository-best-practices`
  - `agent-config-sync`
  - `cli-core`
  - `managed-block`
  - **new** `ubiquitous-language`
- Affected code:
  - `src/commands/way.rs`
  - `src/commands/resource.rs`
  - `src/managed_block.rs`
  - `tests/integration.rs`
