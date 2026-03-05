# Change: Flatten wai resource add skill to wai add skill

## Why

`wai resource add skill <name>` is the most common agent task in a wai project
(scaffolding a new skill file) but requires four command levels. The friction
research (`friction-analysis §1.2`) ranked this the #1 daily pain point:

> "Commands like `wai resource add skill` are four levels deep."

Every CI script, CLAUDE.md snippet, and spoken instruction has to say four words
before the actual argument. Agents invoking this repeatedly burn tokens on
cognitive overhead.

## Chosen Shape: `wai add skill`

Two candidates were evaluated:

| Shape | Assessment |
|-------|------------|
| `wai add skill <name>` | ✓ Fits existing verb-noun grammar (`wai add research`, `wai add plan`, `wai add design`) |
| `wai skill new <name>` | ✗ Introduces a new noun-first sub-hierarchy inconsistent with current verbs |

`wai add skill` extends the existing `wai add` verb directly. No new verb or
noun prefix is needed. `AddCommands` in `src/cli.rs` already enumerates
`Research`, `Plan`, and `Design` — adding `Skill` is a one-variant addition.

## What Changes

- **ADDED**: `wai add skill <name> [--template <template>]` — identical
  behaviour and flags to the current `wai resource add skill` variant.
- **DEPRECATED** (soft, not removed): `wai resource add skill` emits a
  one-line deprecation warning and then delegates to the same handler.
  The alias is preserved indefinitely until a v2 major release removes it.
- **UPDATED**: `cli-core` spec — the "Add artifacts" scenario gains
  `wai add skill`; a note documents the deprecated alias.
- **UPDATED**: `wai init` managed block template — `wai add skill` appears
  in the Quick Reference section alongside the other `add` commands.

## What Does NOT Change

- `wai resource list skills`, `wai resource install`, `wai resource export`,
  and `wai resource import` are **out of scope** for this change. They remain
  under `wai resource` and are not affected.
- Skill file format, templates, naming rules, and storage location are unchanged.
- Non-skill resource management is unchanged.

## Migration Path

| Old command | New command | Action |
|-------------|-------------|--------|
| `wai resource add skill <name>` | `wai add skill <name>` | Deprecated; still works |
| `wai resource add skill <n> --template tdd` | `wai add skill <n> --template tdd` | Deprecated; still works |

Deprecation warning format:
```
⚠ 'wai resource add skill' is deprecated. Use: wai add skill <name>
```

No migration script is required — the alias is transparent.

## Impact

- Affected specs: `cli-core` (modified)
- Affected code: `src/cli.rs` (add `Skill` variant to `AddCommands`),
  `src/commands/add.rs` or `src/commands/resource.rs` (deprecation warning +
  delegate), `src/commands/mod.rs` (`valid_patterns` list update),
  `wai init` managed block template
- CLAUDE.md/AGENTS.md: `wai init` will inject updated Quick Reference on next
  `wai init` run; existing files are not retroactively patched
