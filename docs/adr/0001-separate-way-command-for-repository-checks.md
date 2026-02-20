# ADR 0001: Separate `wai way` Command for Repository Best Practices

## Status

Accepted

## Context

Wai currently provides health checks through the `wai doctor` command, which focuses on wai-specific concerns:
- Workspace directory structure (`.wai/` directories)
- Configuration validity (`config.toml`)
- Plugin synchronization
- Agent config projections

However, there is value in guiding users toward modern repository-level best practices that go beyond wai-specific health. Research on 2026 repository standards shows benefits from standardized tooling:
- Task runners (justfile)
- Git hook managers (prek recommended for performance)
- Dev containers
- Editor configuration (.editorconfig)
- Documentation standards (README, CONTRIBUTING, LICENSE)
- AI assistant instructions (CLAUDE.md, AGENTS.md)
- AI-friendly documentation (llm.txt)
- Agent skills for AI workflows
- CI/CD patterns

The question is: should we extend `wai doctor` to include these repository checks, or create a separate command?

## Decision

We will create a new `wai way` command for repository best practice checks, separate from `wai doctor`.

### Rationale

1. **Separation of concerns**: `wai doctor` checks wai workspace health, `wai way` checks repository practices. These serve different purposes and audiences.

2. **Opt-in discovery**: Users run `wai way` when they're ready for guidance, rather than being bombarded with 7-9 additional warnings on every `wai doctor` run.

3. **Memorable branding**: "the wai way" conveys opinionated best practices and creates a clear mental model for users.

4. **Prevents output bloat**: `wai doctor` output stays focused and actionable for wai-specific issues.

5. **Future expansion**: Creates a natural home for future automation features like `wai way --fix` or `wai way --init`.

6. **No initialization required**: Unlike `wai doctor`, `wai way` can run in any repository, even before `wai init`. This allows users to prepare their repository structure before adopting wai.

### Alternatives Considered

1. **Extend `wai doctor`**: Would add 7-9 warnings to every doctor run, mixing wai-specific and general repository concerns. Rejected due to output bloat and conflation of concerns.

2. **New `wai check` or `wai repo`**: Less memorable names that don't convey the opinionated guidance nature. "The wai way" is more distinctive and suggests best practices.

3. **Subcommand `wai doctor --repo`**: Awkward syntax and still conflates two different purposes under the doctor umbrella.

## Consequences

### Positive

- Clear separation between wai health checks and repository best practices
- Users can discover repository guidance at their own pace
- Distinctive branding ("the wai way") that communicates opinionated guidance
- Foundation for future automation features
- Can be used before wai initialization, broadening utility

### Negative

- Additional command for users to learn
- Potential confusion about when to use `doctor` vs `way`
- Two separate check systems to maintain

### Mitigation

- Clear documentation explaining the difference between `doctor` (wai health) and `way` (repo practices)
- Reference `wai way` in `wai doctor` output for users who want broader guidance
- Consistent check pattern and output format between both commands where appropriate

## References

- Design document: `openspec/changes/add-repo-best-practices/design.md`
- Related issue: wai-fb9
