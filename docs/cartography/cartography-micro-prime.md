# Cartography — micro trace: `wai prime`

**Entry zoom level:** micro (functionality trace)
**Scope covered:** `src/commands/prime.rs` (621 lines) + services it calls

## Entry point

`pub fn run(project: Option<String>)` — `src/commands/prime.rs:25`

## Call chain

```
 1. prime::run(project)              commands/prime.rs:25
 2. require_project()                config (via commands::require_project)  [I/O: fs]
 3. list_projects / resolve_project  config                                  [I/O: fs]
 4. empty-state check                commands/prime.rs:30-51  [pure; early envelope return]
 5. ProjectState::load(state_path)   state.rs (Phase enum)                   [I/O: fs]
 6. check_pending_resume()           commands/prime.rs:305                   [I/O: fs, clock — 12h TTL]
 7. plugin::run_hooks("on_status")   plugin.rs                               [I/O: subprocess (beads, openspec hooks)]
 8. openspec::read_status()          openspec.rs                             [I/O: fs]
 9. ── branch: json ──  render_json()      commands/prime.rs:195
10. ── branch: tty  ──── date/phase/resume printing
11. find_latest_handoff()            commands/prime.rs:378                   [I/O: fs walk]
12. read_handoff_summary()           commands/prime.rs:401                   [I/O: fs; frontmatter parse]
13. extract_next_steps()             commands/prime.rs:352                   [pure on read string]
14. read_recent_plans()              commands/prime.rs:471                   [I/O: fs]
15. render_pipelines()               commands/prime.rs:518                   [I/O: fs; phase↔pipeline matching (pure, :594)]
16. render_health_summary()          commands/prime.rs:604 → doctor::health_summary  [I/O: full doctor check suite]
17. print_envelope(...)              output.rs → genesis envelope
```

## Data flow

```
optional project name arg → resolved project root+name
  → phase string (state file)
  → resume info (.pending-resume file, 12h validity, auto-deleted when stale)
  → plugin hook outputs (beads/openspec summaries)
  → handoff date+snippet+next steps (latest handoff doc)
  → recent plans, matching pipelines (by phase keywords)
  → doctor health summary
→ PrimePayload → genesis Envelope (JSON) or terminal rendering
```

## Structural observations

- Purity ends at hop 2 (`require_project`); prime is read-only — its only writes are the stale `.pending-resume` deletion (hop 6)
- Steps 5–8 run unconditionally for both output modes (comment at `prime.rs:55`); rendering branches after hop 8
- `render_health_summary` pulls the entire doctor check suite into prime — prime is doctor's heaviest external consumer
- Pipeline suggestion logic (`phase_keywords`, `pipeline_matches_phase`, `prime.rs:574-603`) is pure string matching over template TOML — an isolated, testable core inside the handler
- `read_pending_resume`/`extract_next_steps`/`find_latest_handoff`/`read_handoff_summary` are `pub` — unit-tested from outside the module (visible in `tests/` layout)

## Adjacent levels offered

Meso map of the whole `commands` district (`cartography-meso-commands`) or micro traces of `wai sync` / `wai why`.
