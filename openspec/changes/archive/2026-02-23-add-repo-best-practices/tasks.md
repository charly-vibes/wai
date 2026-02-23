# Implementation Tasks

## 1. Documentation and Research
- [ ] 1.1 Move research document to resources (repository-structure-research-2026.md → .wai/resources/repo-best-practices-research.md or docs/)
- [ ] 1.2 Create ADR documenting the decision to add repository checks to wai doctor
- [ ] 1.3 Update main README or documentation to mention repository best practices guidance

## 2. Spec Delta Creation
- [ ] 2.1 Create `repository-best-practices` spec with requirements for each check category
- [ ] 2.2 Add `wai way` command requirement to `cli-core` spec
- [ ] 2.3 Validate specs with `openspec validate add-repo-best-practices --strict`

## 3. Core Implementation
- [ ] 3.1 Create new file `src/commands/way.rs` for repository best practices checker
- [ ] 3.2 Add `way` command to `src/cli.rs` CLI definition
- [ ] 3.3 Define CheckResult struct or reuse from doctor.rs (consider extracting to shared module)
- [ ] 3.4 Implement task runner check (justfile or Makefile)
- [ ] 3.5 Implement git hook manager check (prek .prek.toml or pre-commit .pre-commit-config.yaml)
- [ ] 3.6 Implement EditorConfig check (.editorconfig)
- [ ] 3.7 Implement documentation checks (README.md, CONTRIBUTING.md, LICENSE, .gitignore)
- [ ] 3.8 Implement AI assistant instructions check (CLAUDE.md or AGENTS.md - prefer CLAUDE.md)
- [ ] 3.9 Implement CI/CD check (.github/workflows/ directory)
- [ ] 3.10 Implement dev container check (.devcontainer/ or .devcontainer.json)
- [ ] 3.11 Implement output renderer with "the wai way" branding and ✓/ℹ icons

## 4. Testing
- [ ] 4.1 Add unit tests for each individual check function (test: present, absent, invalid)
- [ ] 4.2 Add integration test for `wai way` command on minimal repository
- [ ] 4.3 Add integration test for `wai way` command on complete repository
- [ ] 4.4 Test that `wai way` works without `.wai/` initialization
- [ ] 4.5 Test JSON output format includes all check results
- [ ] 4.6 Test that `wai way` always exits with code 0 (never fails)
- [ ] 4.7 Verify fix suggestions include URLs and are actionable

## 5. Documentation
- [ ] 5.1 Add `wai way --help` text explaining "the wai way" concept
- [ ] 5.2 Add examples of `wai way` output to documentation
- [ ] 5.3 Document which checks are performed and why
- [ ] 5.4 Link to research document for users wanting deeper context
- [ ] 5.5 Update main README to mention `wai way` for repository best practices

## 6. Validation and Polish
- [ ] 6.1 Run `wai way` on wai repository itself and verify output
- [ ] 6.2 Run `wai way` on empty/minimal repository and verify helpful suggestions
- [ ] 6.3 Test performance (should be <100ms for file existence checks)
- [ ] 6.4 Ensure output is visually appealing with proper icons and formatting
- [ ] 6.5 Test that `wai doctor` is unaffected (no changes to its output)
- [ ] 6.6 Run `cargo fmt` and `cargo clippy` to ensure code quality

## 7. Final Validation
- [ ] 7.1 Validate all specs pass strict validation
- [ ] 7.2 Run full test suite (`cargo test`)
- [ ] 7.3 Manual testing across different repository scenarios
- [ ] 7.4 Update CHANGELOG.md with new feature
- [ ] 7.5 Ready for review and approval
