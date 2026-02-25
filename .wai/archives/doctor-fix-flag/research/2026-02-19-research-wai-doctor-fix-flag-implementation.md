Research: wai doctor --fix flag implementation

## Question
How should a --fix flag be added to wai doctor following the bd (beads) pattern?

## wai doctor Current State

### Source: src/commands/doctor.rs (799 lines)

**Data types:**
- `Status` enum (`doctor.rs:20-26`): Pass, Warn, Fail
- `CheckResult` struct (`doctor.rs:28-35`):
  - name: String
  - status: Status
  - message: String
  - fix: Option<String> — currently only a human-readable hint, never executed
- `DoctorPayload` (`doctor.rs:37-41`): { checks: Vec<CheckResult>, summary: Summary }

**Seven checks (`doctor.rs:69-75`):**
1. `check_directories()` — checks .wai/ PARA subdirs exist
2. `check_config()` — validates config.toml parses
3. `check_plugin_tools()` — checks detected plugin CLIs are in PATH
4. `check_agent_config_sync()` — validates .projections.yml and projection targets
5. `check_project_state()` — validates each project's .state file
6. `check_custom_plugins()` — validates plugin YAML files
7. `check_agent_instructions()` — checks AGENTS.md has wai managed block

**CLI registration (`src/cli.rs:148`):**
`Doctor` is a unit variant with no arguments. Dispatch at `commands/mod.rs:52`:
`Some(Commands::Doctor) => doctor::run()`

**Global flags already available via `current_context()` (`src/context.rs`):**
- `context.yes` — from `--yes` global flag (`cli.rs:46-47`)
- `context.no_input` — from `--no-input` global flag (`cli.rs:42-43`)
- `context.json` — from `--json` global flag (`cli.rs:38-39`)
- `context.safe` — from `--safe` global flag (`cli.rs:50-51`)

**Fix hints (current):**
`CheckResult.fix` is displayed as `→ {fix}` in human output (`doctor.rs:114-116`),
included as `"fix"` in JSON output (`doctor.rs:33`). Never executed.

## bd (beads) Pattern

### Key design:
1. `DoctorCheck` has `Fix: string` field — non-empty = fixable
2. `--fix` flag triggers `applyFixes()` AFTER `runDiagnostics()`
3. `applyFixes()` filters: `status == warning || error AND Fix != ""`
4. Confirmation flow: prompt unless `--yes`; per-item if `--interactive`
5. Central `switch check.Name` dispatch to fix functions
6. Fix functions live in a separate package

### bd --fix flags:
- `--fix`: main flag
- `--yes` / `-y`: skip confirmation
- `--dry-run`: preview without changes

**Note on `--dry-run`:** bd's dry-run flag is out of scope for this implementation.
The wai pattern of "show fix hint in output, let user run it" already covers the
preview use case. Omitting dry-run keeps the implementation minimal.

## Which wai checks are Auto-Fixable?

| Check | Fixable? | Reason |
|---|---|---|
| `check_directories` | **Yes** | `fs::create_dir_all()` — safe, purely additive |
| `check_config` | No | `wai init` does more than just create config; not safe to automate |
| `check_plugin_tools` | No | Cannot install external binaries automatically |
| `check_agent_config_sync` | **Partial** — see below | Only stale-projection results; not missing/invalid config |
| `check_project_state` | **Warn only** | Missing `.state` → write default; corrupted `.state` → skip (data loss risk) |
| `check_custom_plugins` | No | Invalid YAML needs human judgment |
| `check_agent_instructions` | **Yes** | `inject_managed_block()` exists for exactly this purpose |

### check_agent_config_sync fixability detail

`check_agent_config_sync` produces four distinct result types — only one is auto-fixable:

| Condition | fix field | Auto-fixable? |
|---|---|---|
| `.projections.yml` missing | `Some("Run: wai init ...")` | No — requires full init |
| `.projections.yml` unreadable | `None` | No |
| `.projections.yml` invalid YAML | `Some("Fix the YAML syntax...")` | No |
| Projection target missing/stale | `Some("Run: wai sync")` | **Yes** |
| Projection source missing | `Some("Check .projections.yml sources")` | No |

The auto-fix applies only to `CheckResult` entries where `fix == Some("Run: wai sync")`.
In practice these are results named `"Projection → {target}"` from `check_symlink_strategy`,
`check_inline_strategy`, and `check_reference_strategy` (`doctor.rs:444-455`, `doctor.rs:496-499`,
`doctor.rs:538-541`).

### check_project_state fixability detail

Two failure modes, different treatment:
- `Status::Warn` + missing `.state` (`doctor.rs:653-660`): **auto-fixable** by writing
  `ProjectState::default()` via `state.save()` (`state.rs:98-126`)
- `Status::Fail` + invalid YAML (`doctor.rs:672-679`): **not auto-fixable** — overwriting
  a corrupt file would destroy any recoverable data

## Implementation Approach

### CLI changes (`src/cli.rs`)

Add only `--fix` to the `Doctor` variant. Do NOT add `--yes` — it is already a global
flag accessible via `current_context().yes`:

```rust
/// Diagnose workspace health
Doctor {
    /// Automatically fix issues where possible
    #[arg(long)]
    fix: bool,
},
```

Update dispatch in `commands/mod.rs`:
```rust
Some(Commands::Doctor { fix }) => doctor::run(fix),
```

### doctor.rs changes

1. Change `run()` to `run(fix: bool) -> Result<()>`
2. Add `fix_fn: Option<Box<dyn FnOnce(&Path) -> Result<()>>>` to `CheckResult`
   (annotated `#[serde(skip)]` so JSON output is unaffected)
3. Each check function that is fixable attaches a closure at construction time
4. After diagnosis, if `fix` is true: filter to results where `fix_fn.is_some()`,
   confirm (unless `context.yes`), apply in order

**Confirmation flow:**
```
if context.safe {
    // refuse: --fix is a write operation
    return Err(...)
}
if context.no_input || context.yes {
    // skip prompt
} else {
    // cliclack::confirm("Apply N fixes?")
}
```

**`--fix --json` behavior:**
Skip confirmation prompt entirely (non-interactive consumer). Output a structured
payload after fixes:
```json
{
  "fixes_applied": [{ "name": "...", "success": true }],
  "fixes_failed":  [{ "name": "...", "error": "..." }]
}
```

**Exit code:** Do not re-run diagnostics after fixing. Exit 0 if all fixes succeeded,
exit 1 if any fix failed. The fix hint output already tells the user to re-run doctor.

### Fix implementations

**1. Missing PARA directories (`check_directories`, `doctor.rs:135-178`)**
```rust
fix_fn: Some(Box::new(move |project_root| {
    for dir in &missing_dirs {
        fs::create_dir_all(wai_dir(project_root).join(dir))?;
    }
    Ok(())
}))
```

**2. Stale projections (`check_symlink_strategy` / `check_inline_strategy` / `check_reference_strategy`)**

`sync::run(false)` is not safely callable as a library function — it calls
`require_project()` internally, prints its own output, and has `require_safe_mode()`
guards (`sync.rs:79`, `102`, `131`, `173`).

**Design decision:** Extract the three strategy executor functions from `sync.rs`
(`execute_symlink`, `execute_inline`, `execute_reference`) into a new internal
module `src/sync_core.rs` with `pub(crate)` visibility. Both `sync::run()` and the
doctor fix can then call these without the guards or the human-output coupling.
This is a small refactor (move ~3 private functions, make them pub(crate)).

Alternatively: call `sync::run(false)` directly. This prints its own progress
lines to stdout (mixing with doctor output) and will be blocked by `--safe`.
Rejected as the cleaner option requires only a small refactor.

**3. Missing `.state` file (`check_project_state`, Warn case only)**
```rust
fix_fn: Some(Box::new(move |_project_root| {
    let state = ProjectState::default();
    state.save(&state_path)?;
    Ok(())
}))
```
The Fail case (invalid YAML) does not get a `fix_fn`.

**4. Missing/incomplete AGENTS.md managed block (`check_agent_instructions`)**
```rust
fix_fn: Some(Box::new(move |project_root| {
    let agents_md = project_root.join("AGENTS.md");
    let plugins = plugin::detect_plugins(project_root);
    let plugin_names: Vec<&str> = plugins.iter()
        .filter(|p| p.detected)
        .map(|p| p.def.name.as_str())
        .collect();
    managed_block::inject_managed_block(&agents_md, &plugin_names)?;
    Ok(())
}))
```
Note: `inject_managed_block` (`managed_block.rs:223`) requires a `&[&str]` of
detected plugin names — `detect_plugins()` must be called first.

## Global Flag Interactions

| Flag | Behavior with --fix |
|---|---|
| `--yes` | Skip confirmation prompt |
| `--no-input` | Skip confirmation prompt (same as --yes for fix) |
| `--json` | Skip prompt, output structured fix report |
| `--safe` | Refuse to apply fixes; print error |
| `--quiet` | Suppress fix progress output |

## Tests

Integration tests, following the existing `--json` pattern (`tests/integration.rs:1089+`):

- `doctor_fix_repairs_missing_directories` — remove `.wai/archives/`, run `--fix --yes`,
  assert directory recreated and output contains `"fixed"`
- `doctor_fix_skips_confirmation_with_yes` — `--fix --yes` applies without prompt
- `doctor_fix_no_fixable_issues` — healthy workspace, `--fix` reports nothing to fix
- `doctor_fix_repairs_agents_md_block` — delete managed block, `--fix --yes` re-injects it
- `doctor_fix_skips_corrupted_state` — invalid `.state` file, `--fix` reports it cannot
  be auto-fixed and leaves the file untouched
- `doctor_fix_blocked_by_safe_mode` — `--fix --safe` exits non-zero with an error message
