# Cartography — structural health check: wai `src/`

**Entry zoom level:** health check (additive to macro)
**Scope covered:** all 65 `.rs` files under `src/` (cycles, God Object, Shotgun Surgery, Feature Envy, Deep Inheritance, Pure/Impure Entanglement, Broken Composition)

**Baseline used:** n=65 files, median size **328 lines**, p90=1,101, max=2,975. Repo is small-to-medium; thresholds applied relative to this baseline.

## Findings

```
[SHOTGUN SURGERY] severity: medium
Evidence: projection-config concept modeled in 3 files, parsed in 2:
  - commands/sync.rs:14        (own ProjectionsConfig + serde_yml parse)
  - commands/doctor/checks_sync.rs:13-21  (own ProjectionsConfig/ProjectionEntry)
  - sync_core.rs:17            (pub(crate) Projection)
  sync.rs:212-215 and checks_sync.rs:130-140 both match the same closed
  strategy set (symlink|inline|reference|copy).
Quantity: 3 modeling sites / 2 parsers / 2 strategy matches
```

```
[GOD OBJECT] severity: medium (weak-baseline caveat: small repo)
Evidence + quantity (lines, ×median=328):
  - commands/way/mod.rs       2,975  (9.1× median; 22 inline check fns)
  - commands/doctor/mod.rs    1,959  (6.0×; ~10 inline check fns + entry/fix/render)
  - commands/pipeline/mod.rs  1,920  (5.9×; dispatcher + 2 inline commands)
  - commands/why/mod.rs       1,163  (3.5×)
Importers: these are command entries (fan-in from dispatcher only), so the
size signal is not compounded by fan-in — severity capped at medium.
```

```
[PURE/IMPURE ENTANGLEMENT] severity: low (idiomatic for CLI handlers)
Evidence: run() handlers mix parsing, I/O, and println rendering directly
  (e.g., commands/prime.rs:25-193, commands/sync.rs:94-233).
Counter-evidence: pure cores exist where tested — why/parsing.rs (pure),
  prime.rs:574-603 phase/pipeline matching (pure), resource/validation.rs.
Quantity: pervasive at entry level only; service layer (sync_core, llm, state)
  keeps I/O concentrated.
```

### Patterns checked, no findings

- **Circular dependencies** — mutual-import scan across all files: none. Dependency direction is strictly downward (core ← services ← commands).
- **Deep inheritance** — no `extends`-style chains; trait usage is shallow (`LlmClient` single-impl dispatch).
- **Broken composition (layer-skipping)** — no ↑ marks in the macro DSM; `cli` imported only by `commands/*` (13 files); no core module imports services/commands.
- **Feature Envy** — none exceeding 2× threshold; command files import several core modules roughly evenly (config/context/state/json), consistent with dispatcher-style handlers.
- **Orphan modules** — near-orphans (`freshness`, `guided_flows`, fan-in 1) noted in macro report; low severity, single-consumer by design.

## Notes

- The two God-Object candidates (`way/mod.rs`, `doctor/mod.rs`) share a shape: N inline check functions in the module's `mod.rs`. The 22-check inline list in `way/mod.rs:74-97` is the structural pattern's clearest instance.
- Findings are facts about structure; no fixes proposed per skill scope.
