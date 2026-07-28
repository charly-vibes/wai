# Suite Conventions

This document defines the canonical policies for the [charly-vibes](https://github.com/charly-vibes) tool suite. It
covers the infrastructure-level conventions (edition, license, versioning,
justfile setup) that every suite tool should follow.

Design principles, verb grammar, flag consistency, error patterns, and AIX
artefacts are documented in the genesis [tool-craft playbook](https://github.com/charly-vibes/genesis/blob/main/.wai/projects/genesis-foundation/research/tool-craft.md).

## Current State vs Policy

Unless a tool documents a deliberate deviation below, it SHOULD conform to the
canonical policy. Deviations are tracked and converging through the
[`wai-bdqw.6`](https://github.com/charly-vibes/wai/issues) task.

---

## 1. Rust Edition

**Canonical: `2024`** (the latest stable edition as of 2026).

| Tool | Current | Status |
|------|---------|--------|
| wai | `2024` | ✅ |
| genesis | `2024` | ✅ |
| dont | `2024` | ✅ |
| pretender | `2021` | 🎯 update to `2024` |
| espectacular | `2021` | 🎯 update to `2024` |
| testaruda | `2021` | 🎯 update to `2024` |
| vampiro / crua / livin | TBD | 🎯 start with `2024` |

**Rationale**: `2024` adds `unsafe` attributes in scope, `if let` chains
without nesting, and `impl Trait` in RPIT — all useful patterns in the
suite's codebase. The two-cycle lag policy means tools on `2021` should
migrate before the next edition.

---

## 2. License

**Canonical: `Apache-2.0`**.

| Tool | Current | Status |
|------|---------|--------|
| pretender | `Apache-2.0` | ✅ |
| dont | `Apache-2.0` | ✅ |
| espectacular | `Apache-2.0` | ✅ |
| testaruda | `Apache-2.0` | ✅ |
| genesis | `Apache-2.0` | ✅ |
| vampiro / crua / livin | `Apache-2.0` (vampiro) | ✅ planned |
| wai | `MIT` | ⚠️ deliberate deviation |

**Rationale**: Apache 2.0 is the suite standard — it provides express patent
grant and is the de-facto standard for Rust open-source CLI tools. Wai uses
MIT as a deliberate choice (permissive, no notice requirement in binary
distributions). A future suite MAY dual-license or converge on a single
license; this is not blocked on the current state.

---

## 3. Version Scheme

**Canonical: Semantic Versioning (SemVer) `0.x.y` or `MAJOR.MINOR.PATCH`**.

| Tool | Current | Status |
|------|---------|--------|
| pretender | `0.3.1` | ✅ SemVer |
| dont | `0.2.2` | ✅ SemVer |
| espectacular | `0.3.0` | ✅ SemVer |
| testaruda | `0.2.4` | ✅ SemVer |
| genesis | `0.2.0` | ✅ SemVer |
| vampiro / crua / livin | TBD | 🎯 start with `0.1.0` |
| wai | `2026.7.16` (CalVer) | ⚠️ deliberate deviation |

**Rationale**: crates.io requires SemVer for crate dependencies, and every
CLI tool in the suite ships as a crate. Wai uses CalVer because its releases
are coupled to the project timeline (weekly-ish), not to API stability
contracts — the `wai` binary is never depended on as a library. The suite
standard is SemVer; wai's CalVer is a documented outlier.

**Convention**: Tools at `0.x.y` MAY bump minor (`0.x+1.0`) for breaking
changes until `1.0.0`. Once stable, follow SemVer strictly.

---

## 4. Just Recipes

**Canonical recipe set** (every tool ships these four):

| Recipe | Purpose | Output |
|--------|---------|--------|
| `default` | Build + test (or `--list` if build/test not applicable) | `just` |
| `build` | Compile the tool | `just build` |
| `test` | Run the test suite | `just test` |
| `validate` | Full quality gate (clippy + test + fmt) | `just validate` |

Optional recipes (tool-specific):

| Recipe | Purpose | Examples |
|--------|---------|----------|
| `install` | Install the tool | `cargo install --path .` |
| `publish` | Release to crates.io | `cargo publish` |
| `docs` | Build documentation | `mdbook build docs` |
| `prime` / `status` | Orchestrator workflows | wai, dont, espectacular |

**Canonical recipe NOT to use**: `ah` (testaruda uses this for its test
runner — rename to `test` for consistency).

| Tool | `default` | `build` | `test` | `validate` | Status |
|------|-----------|---------|--------|------------|--------|
| wai | build+test | ✅ | ✅ | ✅ | ✅ |
| pretender | `--list` | ✅ | ✅ | ✅ | ✅ |
| dont | `--list` | ✅ | ✅ | ✅ | ✅ |
| espectacular | `--list` | ✅ | ✅ | ✅ | ✅ |
| testaruda | `--list` | ✅ | ✅ (as `ah`) | ❌ | 🎯 rename `ah` → `test`, add `validate` |
| vampiro / crua / livin | build+test | ✅ | ✅ | ✅ | 🎯 start aligned |

---

## 5. Justfile Shell

**Canonical**:

```just
set shell := ["bash", "-uc"]
```

The `-u` flag treats unset variables as errors, preventing silent failures in
recipe pipelines. The `-c` flag is required by `just` for shell execution.

| Tool | `set shell` | Status |
|------|-------------|--------|
| wai | `["bash", "-uc"]` | ✅ |
| genesis | `["bash", "-uc"]` | ✅ |
| vampiro | `["bash", "-uc"]` | ✅ planned |
| crua | `["bash", "-uc"]` | ✅ planned |
| livin | `["bash", "-uc"]` | ✅ planned |
| dont | `["bash", "-cu"]` | 🎯 add `-u` |
| pretender | absent | 🎯 add `set shell` |
| espectacular | absent | 🎯 add `set shell` |
| testaruda | absent | 🎯 add `set shell` |

---

## 6. Default Just Recipe

**Canonical: `just` runs `build` + `test`** (not `--list`).

A fresh clone should be buildable with a single `just` command. Using
`--list` as default means the developer must look up the recipe name before
building, which is friction for new contributors.

| Tool | Default | Status |
|------|---------|--------|
| wai | `build test` | ✅ |
| genesis | `build test` | ✅ |
| vampiro | `build` | 🎯 add `test` |
| crua | `build` | 🎯 add `test` |
| livin | `build` | 🎯 add `test` |
| pretender | `--list` | 🎯 change to `build test` |
| dont | `--list` | 🎯 change to `build test` |
| espectacular | `--list` | 🎯 change to `build test` |
| testaruda | `--list` | 🎯 change to `build test` |

---

## Summary of Required Changes

| Tool | Edition | License | Version | Just recipes | Shell | Default |
|------|---------|---------|---------|--------------|-------|---------|
| pretender | ⬆️ 2024 | ✅ | ✅ | ✅ | ➕ add | 🔁 → build test |
| dont | ✅ | ✅ | ✅ | ✅ | 🔧 `-cu`→`-uc` | 🔁 → build test |
| espectacular | ⬆️ 2024 | ✅ | ✅ | ✅ | ➕ add | 🔁 → build test |
| testaruda | ⬆️ 2024 | ✅ | ✅ | 🔧 `ah`→`test`, ➕ validate | ➕ add | 🔁 → build test |
| vampiro | 🆕 2024 | ✅ | 🆕 0.1.0 | ✅ | ✅ | 🔧 → build test |
| crua | 🆕 2024 | ✅ | 🆕 0.1.0 | ✅ | ✅ | 🔧 → build test |
| livin | 🆕 2024 | ✅ | 🆕 0.1.0 | ✅ | ✅ | 🔧 → build test |

✅ = conforms · 🎯 = target · ➕ = missing · 🔧 = needs change · ⬆️ = upgrade · 🆕 = initial setup

Each tool SHOULD implement its own changes as separate tickets. This document
is the suite-level source of truth that those tickets trace to.

---

## References

- [genesis tool-craft playbook](https://github.com/charly-vibes/genesis/blob/main/.wai/projects/genesis-foundation/research/tool-craft.md)
  — design principles, verb grammar, flag consistency, error handling, AIX
- [Toolchain Synergy](./toolchain.md) — full tool list and integration surfaces
- [`wai-bdqw.6`](https://github.com/charly-vibes/wai/issues) — tracking ticket