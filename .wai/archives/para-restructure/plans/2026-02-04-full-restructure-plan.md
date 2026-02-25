# Plan: Align Repo with Conversation's Full Vision

## Summary

Restructure `wai` from a bead-centric work tracker to a **PARA-based artifact organizer** with project state machines, agent config projections, handoff generation, plugin hooks, and cross-artifact search/timeline. Beads (`.beads/`) remains a separate external tool that wai detects and references but does not manage.

---

## Key Architectural Shifts

| Aspect | Current Repo | Conversation's Vision |
|--------|-------------|----------------------|
| `.wai/` structure | `beads/`, `research/`, `plugins/` | `projects/`, `areas/`, `resources/`, `archives/`, `plugins/` |
| Core unit | Bead (draft→ready→in-progress→done) | Project with PARA category + state machine |
| Beads relationship | Wai manages bead lifecycle internally | Beads is external (`.beads/`), wai detects/references |
| Agent configs | Not addressed | Single source of truth with projections to `.claude/`, `.cursorrules`, etc. |
| Handoffs | Not addressed | First-class artifact type with templates and plugin enrichment |
| Plugin system | manifest.toml, basic install/remove | YAML configs with hooks, commands, detector patterns |
| CLI commands | `new`, `add`, `show`, `move` (bead-focused) | `new`, `add`, `show`, `move`, `phase`, `sync`, `config`, `handoff`, `search`, `timeline` |

---

## Implementation Phases

### Phase 1: Update OpenSpec Specs ✅ DONE

Update specifications to match the conversation's architecture before writing code.

**1a. Archive `bead-lifecycle` spec** ✅
- `openspec/specs/bead-lifecycle/spec.md` → archived
- Wai no longer manages bead lifecycle; that's `bd`'s job

**1b. Update `cli-core` spec** ✅
- Removed bead-centric verbs (`new bead`, `move bead`, `show beads`)
- Added: `new project`, `new area`, `new resource`
- Added: `phase` command group (`phase`, `phase next`, `phase set`, `phase back`)
- Added: `sync`, `config`, `handoff`, `search`, `timeline` commands
- Updated `init` to create PARA structure
- Updated `status` to show project phase + plugin status

**1c. Create new spec: `para-structure`** ✅
**1d. Create new spec: `project-state-machine`** ✅
**1e. Create new spec: `agent-config-sync`** ✅
**1f. Create new spec: `handoff-system`** ✅
**1g. Update `plugin-system` spec** ✅
**1h. Update `research-management` spec** ✅
**1i. Create new spec: `timeline-search`** ✅
**1j. Update `onboarding`, `help-system`, `error-recovery`, `context-suggestions` specs** ✅

### Phase 2: Core Architecture (Rust Code) ✅ DONE

**2a. Restructure config and directory layout** ✅

Files modified:
- `src/config.rs` — New PARA structure constants and path helpers
- `src/commands/init.rs` — Create PARA directories on init

New `.wai/` structure:
```
.wai/
├── config.toml
├── projects/
├── areas/
├── resources/
│   ├── agent-config/
│   │   ├── skills/
│   │   ├── rules/
│   │   ├── context/
│   │   └── .projections.yml
│   ├── patterns/
│   └── templates/
├── archives/
└── plugins/
```

**2b. Update CLI command structure** ✅

File: `src/cli.rs`
- Replaced `NewCommands::Bead` with `NewCommands::Area`, `NewCommands::Resource`
- Replaced `MoveCommands::Bead` with `MoveArgs { item, target }`
- Replaced `ShowCommands::Beads/Phase` with `Show { name: Option<String> }`
- Added `Commands::Phase(PhaseCommands)` — `next`, `set`, `back`, `show`
- Added `Commands::Sync` — with `--status` flag
- Added `Commands::Config(ConfigCommands)` — `add`, `list`
- Added `Commands::Handoff(HandoffCommands)` — `create`
- Added `Commands::Search { query, type_filter, project }`
- Added `Commands::Timeline { project }`
- Added `Commands::Plugin(PluginCommands)` — `list`, `enable`, `disable`
- Added `Commands::Import { path }`

**2c. Implement PARA commands** ✅

New files:
- `src/commands/new.rs` — `wai new project`, `wai new area`, `wai new resource`
- `src/commands/show.rs` — `wai show [name]` with PARA overview
- `src/commands/move_cmd.rs` — `wai move <item> <category>`
- `src/commands/add.rs` — `wai add research`, `wai add plan`, `wai add design`

**2d. Implement project state machine** ✅

New files:
- `src/state.rs` — Phase enum, state file load/save, transition validation
- `src/commands/phase.rs` — `wai phase show/next/set/back`

**2e. Update error types** ✅

File: `src/error.rs`
- Removed `BeadNotFound`
- Added `ProjectNotFound`, `AreaNotFound`, `ResourceNotFound`
- Added `NoProjectContext`, `ConfigSyncError`, `HandoffError`, `Yaml`

### Phase 3: Agent Config Projections — REMAINING

**3a. Implement projections config** — Code exists in `src/commands/sync.rs`

The sync command already:
- Parses `.projections.yml`
- Supports three strategies: symlink, inline, reference
- Shows sync status with `--status` flag

Remaining work:
- End-to-end testing of each projection strategy
- Handle edge cases (broken symlinks, permission errors)
- Add `--dry-run` option

**3b. Implement config management** — Code exists in `src/commands/config_cmd.rs`

Already implemented:
- `wai config add skill|rule|context <file>` — copies to agent-config subdirectory
- `wai config list` — lists all config files

Remaining work:
- `wai config edit <path>` — open in `$EDITOR` (not in plan to avoid blocking agents)

**3c. Implement import command** — Code exists in `src/commands/import.rs`

Already implemented:
- `wai import <path>` — imports directory or file
- Auto-categorizes by filename patterns (rule, skill, context)

Remaining work:
- Smarter categorization (parse file content, not just filename)
- Claude-specific detection (`.claude/` structure awareness)
- Cursor-specific detection (`.cursorrules` parsing)

### Phase 4: Handoff System — REMAINING

**4a. Implement handoff templates** — Code exists in `src/commands/handoff.rs`

Already implemented:
- `wai handoff create <project>` — generates handoff with frontmatter
- Git status enrichment (calls `git status --short`)
- Beads enrichment (calls `bd list --status=open`)
- Date-collision handling (appends counter suffix)

Remaining work:
- Configurable templates (load from `.wai/resources/templates/`)
- Plugin hook abstraction (currently hardcoded git/beads calls)
- Template variable substitution

### Phase 5: Plugin Architecture — REMAINING

**5a. Implement plugin system**

Already implemented:
- `wai plugin list` — shows built-in plugins with auto-detection
- Custom plugin YAML detection in `.wai/plugins/`
- Plugin enable/disable stubs

Remaining work:
- New file: `src/plugin.rs` — Plugin trait, YAML config parsing, hook execution engine
- New files: `src/plugins/{mod,beads,openspec,git}.rs` — Built-in plugin implementations
- Hook execution pipeline wired through `new`, `phase`, `handoff`, `status` commands
- Command pass-through routing (e.g., `wai beads list` → `bd list --json`)

### Phase 6: Extended Features — REMAINING (code exists)

**6a. Timeline** — `src/commands/timeline.rs` is implemented
- Scans date-prefixed files, sorts chronologically, displays with type labels
- May want: date range filtering, `--reverse` flag

**6b. Search** — `src/commands/search.rs` is implemented
- Walks `.wai/` tree, grep-style matching with highlighted output
- Filters by `--type` and `--in` project
- May want: regex support, result count limiting

### Phase 7: Update Existing Code — REMAINING

**7a. Update `src/commands/status.rs`** ✅ Already updated
- Shows project phase instead of bead counts
- Shows plugin status summaries
- Shows phase-based suggestions

**7b. Update `src/commands/init.rs`** ✅ Already updated
- Creates PARA directory structure
- Auto-detects plugins
- Creates default `.projections.yml`

**7c. Update `README.md`** ✅ Already updated

**7d. Address change proposals in `openspec/changes/`** — REMAINING
- `add-self-healing-errors` — needs revision for new error types
- `add-progressive-disclosure` — needs revision for phase-based output
- `add-context-suggestions` — needs revision for phase-based suggestions
- `add-first-run-experience` — needs revision for PARA onboarding

### Phase 8: Dependencies & Build — REMAINING

**8a. Update `Cargo.toml`** ✅ Already updated
- Added `serde_yaml`, `chrono`, `walkdir`, `slug`, `tempfile`

**8b. Build verification** ✅ Build is clean, clippy is clean

**8c. Integration tests** — REMAINING
- Test `wai init` creates correct PARA structure
- Test `wai new project` creates project with `.state`
- Test `wai phase next/back/set` transitions work
- Test `wai add research` creates date-prefixed files
- Test `wai move` relocates between categories
- Test `wai search` finds content
- Test `wai timeline` shows chronological entries

---

## Files Summary

### Files Modified (Phase 1-2) ✅
| File | Changes |
|------|---------|
| `src/cli.rs` | Major restructure — new commands, removed bead verbs |
| `src/config.rs` | PARA constants, path helpers, `&Path` signatures |
| `src/error.rs` | New error variants, removed bead errors |
| `src/commands/mod.rs` | New command dispatch |
| `src/commands/init.rs` | PARA directory creation, plugin detection |
| `src/commands/status.rs` | Phase display, plugin status |
| `src/main.rs` | Added `mod state` |
| `Cargo.toml` | New dependencies |
| `Cargo.lock` | Updated lockfile |
| `README.md` | Updated architecture and examples |
| `openspec/project.md` | Updated domain context |
| `openspec/specs/cli-core/spec.md` | Major update |
| `openspec/specs/research-management/spec.md` | PARA context |
| `openspec/specs/plugin-system/spec.md` | YAML configs, hooks |
| `openspec/specs/onboarding/spec.md` | New commands |
| `openspec/specs/help-system/spec.md` | Minor adjustment |
| `openspec/specs/error-recovery/spec.md` | New error types |
| `openspec/specs/context-suggestions/spec.md` | Phase-based |

### Files Created (Phase 1-2) ✅
| File | Purpose |
|------|---------|
| `src/state.rs` | Phase state machine |
| `src/commands/new.rs` | New project/area/resource |
| `src/commands/add.rs` | Add artifacts |
| `src/commands/show.rs` | Show details |
| `src/commands/move_cmd.rs` | Move between categories |
| `src/commands/phase.rs` | Phase command |
| `src/commands/sync.rs` | Sync command |
| `src/commands/config_cmd.rs` | Config command |
| `src/commands/handoff.rs` | Handoff command |
| `src/commands/search.rs` | Search command |
| `src/commands/timeline.rs` | Timeline command |
| `src/commands/plugin.rs` | Plugin command |
| `src/commands/import.rs` | Import command |
| `openspec/specs/para-structure/spec.md` | PARA spec |
| `openspec/specs/project-state-machine/spec.md` | State machine spec |
| `openspec/specs/agent-config-sync/spec.md` | Projections spec |
| `openspec/specs/handoff-system/spec.md` | Handoff spec |
| `openspec/specs/timeline-search/spec.md` | Timeline/search spec |

### Files to Create (Remaining)
| File | Purpose |
|------|---------|
| `src/plugin.rs` | Plugin trait and YAML parsing |
| `src/plugins/mod.rs` | Plugin module |
| `src/plugins/beads.rs` | Beads plugin |
| `src/plugins/openspec.rs` | OpenSpec plugin |
| `src/plugins/git.rs` | Git plugin |
| `tests/integration.rs` | Integration tests |

### Files Archived
| File | Reason |
|------|--------|
| `openspec/specs/bead-lifecycle/spec.md` → `.archived` | Wai no longer manages beads |

---

## Execution Order for Remaining Work

1. **Phase 3** — Test and harden sync/config/import (low effort, code exists)
2. **Phase 5** — Plugin abstraction layer (`src/plugin.rs`, `src/plugins/`) — this is the biggest remaining piece
3. **Phase 4** — Wire handoffs through plugin hooks (depends on Phase 5)
4. **Phase 6** — Polish timeline/search (code exists, needs edge cases)
5. **Phase 7** — Revise openspec change proposals
6. **Phase 8** — Integration tests
