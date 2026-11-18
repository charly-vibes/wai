# managed-block spec delta: adopt genesis injector

## MODIFIED Requirements

### Requirement: Cross-tool tracking convention

The `<!-- …:START -->`/`<!-- …:END -->` injector mechanics SHALL be sourced
from `genesis::managed_block`. wai retains ownership of its block *content*
(plugin-aware, slim Layer-1 progressive-disclosure body) but delegates the
read/inject/replace mechanics to genesis.

#### Scenario: sync still refreshes the WAI block

- **WHEN** `wai sync` is run after adopting genesis
- **THEN** the `<!-- WAI:START -->` … `<!-- WAI:END -->` block in `AGENTS.md`
  SHALL be injected/refreshed via `genesis::managed_block`
- **AND** the block body SHALL remain wai's plugin-aware slim content
- **AND** no local injector code SHALL remain in `src/managed_block.rs`.

#### Scenario: Both plugins detected
- **WHEN** `wai init` or `wai reflect` runs
- **AND** both beads and openspec plugins are detected
- **THEN** the generated WAI block includes a "Tracking Work Across Tools"
  section with the cross-reference format and sync rule

#### Scenario: Only one plugin detected
- **WHEN** `wai init` or `wai reflect` runs
- **AND** only one of beads or openspec is detected (not both)
- **THEN** the "Tracking Work Across Tools" section is NOT included