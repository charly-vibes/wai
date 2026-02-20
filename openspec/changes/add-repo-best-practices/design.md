# Design: Repository Best Practices (`wai way` command)

## Context

Wai currently focuses on managing its own `.wai/` workspace structure but doesn't guide users toward repository-level best practices. Research shows that modern repositories (2026) benefit from standardized tooling: task runners (justfile), git hook managers (prek recommended over pre-commit for performance), dev containers, editor configuration, documentation standards, and CI/CD patterns.

The `wai doctor` command provides health checks for wai-specific concerns (directory structure, config validity, plugin sync). Adding repository recommendations to doctor would bloat its output and mix concerns (wai health vs general repo standards).

## Goals / Non-Goals

### Goals
- Provide actionable guidance on repository structure and tooling
- Help users discover and adopt proven 2026 best practices
- Keep checks non-invasive (warnings, not failures)
- Align with research findings on modern repository standards
- Lay groundwork for future automation (e.g., `wai setup` commands)

### Non-Goals
- Automated setup/scaffolding (future work after this change proves value)
- Enforcing or requiring any specific practices (all voluntary)
- Supporting every possible tool variation (focus on most common 2026 standards)
- Language-specific linting/formatting checks (out of scope for wai)

## Decisions

### Decision 1: New `wai way` Command vs. Extending Doctor

**Choice**: Create new `wai way` command for repository best practices

**Rationale**:
- **Separation of concerns**: Doctor checks wai health, way checks repo practices
- **Opt-in discovery**: Users run `wai way` when ready, not bombarded on every `wai doctor`
- **Memorable branding**: "the wai way" = opinionated best practices
- **Prevents bloat**: Doctor output stays focused and actionable
- **Future expansion**: Natural home for `wai way --fix`, `wai way --init` automation

**Alternatives considered**:
- Extend `wai doctor`: Would add 7-9 warnings to every doctor run, mixing wai-specific and general repo concerns
- New `wai check` or `wai repo`: Less memorable, doesn't convey opinionated guidance
- Subcommand `wai doctor --repo`: Awkward, still conflates two different purposes

### Decision 2: Check Severity Model

**Choice**: Repository best practice checks use **info/pass** model, no failures

**Rationale**:
- Best practices are recommendations, not requirements
- `wai way` should never exit with non-zero code (doesn't block CI)
- Users opt in to run the command, so output is expected
- Clear distinction from `wai doctor` which can fail on broken config

**Severity mapping**:
- **Pass**: File/configuration exists and follows best practices (green ✓)
- **Info**: File missing or improvement suggested (blue ℹ)
- **Fail**: Not used in `wai way` (reserved for `wai doctor` critical issues)

**Output format**:
```
  ◆ Repository Best Practices ("the wai way")

  ✓ Task runner: justfile found
  ℹ Git hooks: No hook manager configured
    → Create .prek.toml to automate formatting and linting
  ✓ Editor config: .editorconfig found
  ℹ Documentation: Missing CONTRIBUTING.md, LICENSE
    → Add contribution guidelines and license

  Summary: 5/8 best practices adopted
  Quick start: Focus on .gitignore, README.md, and justfile
```

### Decision 3: Check Priority and Scope

**Choice**: Focus on high-impact, language-agnostic checks first

**Priority 1 (include now)**:
- Task runner (justfile or Makefile)
- Git hook manager (prek recommended, or legacy pre-commit)
- Editor configuration (.editorconfig)
- Core documentation (README.md, CONTRIBUTING.md, LICENSE, .gitignore)
- AI assistant instructions (CLAUDE.md or AGENTS.md)
- AI-friendly documentation (llm.txt for broader LLM compatibility)
- Agent skills (universal-rule-of-5-review, deliberate-commit for AI workflows)
- CI/CD presence (.github/workflows/)
- Dev container configuration (.devcontainer/)

**Priority 2 (future work)**:
- Conventional commits (check for commit message format in git log)
- Security features (Dependabot, secret scanning)
- Lockfile presence (language-specific: package-lock.json, Cargo.lock, etc.)
- Branch naming conventions (requires git integration)

**Rationale**:
- Priority 1 checks are universally applicable and easy to verify (file existence + basic validation)
- Priority 2 requires deeper integration (git history parsing, GitHub API, language detection)

### Decision 4: Check Implementation Pattern

**Choice**: New `src/commands/way.rs` with shared check utilities

**Pattern**:
```rust
// src/commands/way.rs
pub fn run() -> Result<()> {
    let project_root = std::env::current_dir()?;
    let mut checks = Vec::new();

    checks.push(check_task_runner(&project_root));
    checks.push(check_git_hooks(&project_root));
    checks.push(check_editorconfig(&project_root));
    checks.extend(check_documentation(&project_root));
    checks.push(check_ai_instructions(&project_root));
    checks.push(check_cicd(&project_root));
    checks.push(check_devcontainer(&project_root));

    render_way_output(&checks)?;
    Ok(()) // Always exits 0
}
```

**Rationale**:
- Separate file keeps concerns isolated
- Can reuse CheckResult struct from doctor.rs
- Independent evolution (doctor vs way can diverge)
- Easy to test individual checks
- No risk of breaking existing doctor functionality

### Decision 5: Relationship to wai doctor

**Choice**: `wai way` does NOT require wai to be initialized

**Rationale**:
- Repository best practices apply to any repo, not just wai workspaces
- Users can run `wai way` before `wai init` to prepare repository
- Broader utility: helps users set up good repo structure first
- Natural workflow: `wai way` → fix issues → `wai init` → start using wai

**Difference from doctor**:
- `wai doctor`: Requires `.wai/` (checks wai-specific health)
- `wai way`: Works anywhere (checks general repo standards)

### Decision 6: Configuration and Customization

**Choice**: No configuration for check enablement in this change (run all checks)

**Rationale**:
- YAGNI - wait for user feedback before adding complexity
- Command is opt-in, so users control when they see output
- All checks are informational - no failures to disable
- Future: Could add flags like `--focus=docs` or `--skip=devcontainer`

**Future consideration**:
```bash
wai way --focus=essential  # Only README, .gitignore, justfile
wai way --skip=devcontainer,cicd  # Skip optional checks
wai way --fix  # Auto-generate missing files (future)
```

### Decision 7: AI-Friendly Documentation Standards

**Choice**: Check for llm.txt as a separate AI documentation standard

**Rationale**:
- **llm.txt standard**: Emerging specification (https://llmstxt.org) for providing AI-friendly project context
- **Broader compatibility**: Works with multiple AI tools, not just Claude or specific assistants
- **Complements CLAUDE.md**: CLAUDE.md is assistant-specific instructions, llm.txt is project documentation
- **2026 trend**: Similar to robots.txt for search engines, llm.txt is becoming standard for LLMs
- **Lightweight**: Simple markdown file that LLMs can easily parse

**Implementation**:
- Check for `llm.txt` in repository root
- Suggest creating it alongside CLAUDE.md for comprehensive AI support
- Don't require it (WayStatus::Info if missing)

### Decision 8: Agent Skills for AI-Assisted Workflows

**Choice**: Check for agent skills configuration in `.wai/resources/skills/`

**Rationale**:
- **AI development workflows**: Projects using AI assistance benefit from documented practices
- **Universal rule of 5 review**: Code review guideline (review code in chunks of 5 or fewer items)
- **Deliberate commit**: Practice of writing intentional, well-structured commit messages
- **Skill reusability**: Skills defined as SKILL.md files can be referenced by AI assistants
- **Wai integration**: Aligns with wai's resource management and agent skill system

**Recommended skills**:
- `universal-rule-of-5-review`: Limit code review scope to maintain focus and quality
- `deliberate-commit`: Structured approach to commit messages (why, what, context)

**Implementation**:
- Check for `.wai/resources/skills/` directory
- Look for recommended skill files (SKILL.md format)
- Report count of skills found
- Suggest creating skills if directory missing or key skills absent

## Risks / Trade-offs

### Risk: Check Explosion
- **Concern**: Too many checks overwhelm users with warnings
- **Mitigation**: Start with Priority 1 checks only, group related checks (e.g., "Documentation" covers README, CONTRIBUTING, LICENSE)

### Risk: False Positives
- **Concern**: Users have valid reasons not to follow certain practices
- **Mitigation**: All checks are warnings, clear fix suggestions explain the value proposition

### Risk: Maintenance Burden
- **Concern**: Best practices evolve, tools change (e.g., prek emerging as faster alternative to pre-commit)
- **Mitigation**: Reference research document (repository-structure-research-2026.md), plan annual review of recommendations, check for multiple tool variants (e.g., accept both prek and legacy pre-commit)

### Trade-off: Validation Depth
- **Current**: Check file existence, basic YAML/TOML parsing for pre-commit, TOML for prek
- **Not doing**: Deep validation (e.g., checking if git hooks are correctly configured for the language)
- **Rationale**: Shallow checks are fast, reliable, and still valuable. Deep validation adds complexity and failure modes.

## Migration Plan

N/A - purely additive change, no migration needed.

## Open Questions

1. **Should we check for both CLAUDE.md AND AGENTS.md, or prefer one?**
   - Research shows CLAUDE.md is emerging standard
   - Wai uses AGENTS.md internally via openspec
   - **Proposal**: Check for either, suggest CLAUDE.md if neither exists (more widely adopted in 2026)

2. **Should task runner check prefer justfile over Makefile?**
   - Research shows justfile is emerging as modern standard
   - Many projects still use Makefile
   - **Proposal**: Pass if either exists, suggest justfile in fix text if neither exists

3. **How to handle monorepo scenarios?**
   - Some checks (CI/CD, git hooks) might be at monorepo root
   - Wai might be initialized in a subdirectory
   - **Proposal**: Check only at project_root (where wai was run), don't walk up tree. Future enhancement if users request it.

4. **Should llm.txt be checked separately or combined with AI instructions?**
   - llm.txt is a new standard (https://llmstxt.org) for AI-friendly documentation
   - CLAUDE.md/AGENTS.md are assistant-specific, llm.txt is broader
   - **Proposal**: Separate check for llm.txt, suggest it in AI instructions check if missing

5. **Should agent skills be required or optional?**
   - Agent skills (like universal-rule-of-5-review, deliberate-commit) are workflow practices
   - Not all projects use wai for AI-assisted development
   - **Proposal**: Optional check (WayStatus::Info if missing), suggest creating `.wai/resources/skills/` with recommended SKILL.md files
