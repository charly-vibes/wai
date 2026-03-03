## Context

Skills created in one project are generically useful but trapped in that project's
`.wai/`. The copy-paste workflow breaks discoverability and means skills diverge.
This design adds the minimum infrastructure for skill reuse without a registry server.

## Goals / Non-Goals

- Goals:
  - Global install path that project skills shadow
  - Install from local repo (common case: copy across your own projects)
  - Export/import for sharing bundles outside git
- Non-Goals:
  - Remote registry or package server
  - Versioning or dependency resolution
  - Automatic update of globally installed skills

## Decisions

- Decision: Global path is `~/.wai/resources/skills/`, not `~/.wai/skills/`
  - Mirrors the local structure `.wai/resources/agent-config/skills/` — same
    hierarchy, different root
  - Alternative: `~/.config/wai/skills/` — `~/.wai/` is more consistent with
    wai's existing conventions

- Decision: Resolution order is local-first, then global
  - When both exist, the local skill silently takes priority
  - No warning on shadow — the user explicitly created the local version
  - Alternative: warn on shadow — rejected; noisy in normal workflows

- Decision: Portability validation is a warning, not a block
  - Detecting every hardcoded project name would require heuristics prone to false
    positives (e.g., the word "main" is not necessarily a project name)
  - A warning draws attention; blocking installation is too aggressive
  - Template system (`add-skill-templates`) is the positive path to portability

- Decision: Export/import uses tar.gz, not zip
  - tar.gz is the standard in Unix toolchains; Rust has good `tar` + `flate2` crates
  - The directory structure inside the archive mirrors the skills hierarchy exactly,
    making it inspectable with `tar -tz`

## Risks / Trade-offs

- Global skills are shared across all projects on the machine. A destructive edit
  to a globally installed skill affects every project. Mitigated by the fact that
  install is always an explicit `--global` operation; no auto-install.
- Import overwrites require confirmation to prevent silent data loss.

## Open Questions

- Should `wai resource list skills` show the source (local vs global) in all
  output modes, or only when `--verbose` is passed? Lean toward always showing it
  to aid debugging, but keep it visually minimal.
