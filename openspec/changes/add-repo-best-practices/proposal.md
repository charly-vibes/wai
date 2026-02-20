# Change: Add Repository Best Practices Recommendations (`wai way`)

## Why

Modern software repositories benefit from standardized tooling and conventions (justfile, git hook managers like prek, devcontainers, EditorConfig, conventional commits, etc.) that reduce cognitive load, improve onboarding, and maintain code quality. Currently, wai checks its own internal health via `wai doctor`, but doesn't help users discover and adopt repository-level best practices.

Introducing a dedicated `wai way` command provides opinionated best practice recommendations without bloating the doctor command:
- Guides users toward proven patterns from 2026 industry standards
- Validates repository structure with actionable suggestions
- Opt-in rather than always-on (keeps `wai doctor` focused on wai health)
- Memorable name: "the wai way" = opinionated recommendations
- Reduces setup friction for new projects
- Provides a foundation for future automation (e.g., `wai way --fix`, `wai way --init`)

## What Changes

- **New command**: `wai way` to check repository best practices
- **New capability**: `repository-best-practices` spec defining recommended repository standards
- **Modular check system**: Each best practice check is independent (pass/warn/info) with fix suggestions
- **Research artifact**: Include the comprehensive 2026 best practices research document as a reference

The `wai way` command checks:
- Task runner (justfile or Makefile)
- Git hook manager (prek recommended, or pre-commit)
- Editor configuration (.editorconfig)
- Core documentation (README.md, CONTRIBUTING.md, LICENSE, .gitignore)
- AI assistant instructions (CLAUDE.md or AGENTS.md)
- AI-friendly documentation (llm.txt for broader LLM tool compatibility)
- Agent skills configuration (universal-rule-of-5-review, deliberate-commit)
- CI/CD configuration (.github/workflows/)
- Dev container configuration (.devcontainer/)

This change:
- Does NOT modify `wai doctor` (keeps it focused on wai-specific health)
- Does NOT implement automated setup commands (future: `wai way --fix`)
- Does NOT enforce or require these practices (all recommendations, opt-in command)

## Impact

- **Affected specs**:
  - NEW: `repository-best-practices` (defines standards and check requirements)
  - MODIFIED: `cli-core` (adds new `wai way` command)

- **Affected code**:
  - NEW: `src/commands/way.rs` - Repository best practices checker
  - `src/cli.rs`: Add `way` command definition
  - Shared check utilities (may extract helpers from doctor.rs)

- **User experience**:
  - `wai doctor` remains focused on wai workspace health
  - `wai way` provides opt-in repository recommendations
  - Users discover "the wai way" when ready for repository improvements
  - Clear separation of concerns: doctor=health, way=recommendations

- **Breaking changes**: None (purely additive)
