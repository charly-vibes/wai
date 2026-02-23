# Rule of 5 Universal Review - Repository Best Practices (`wai way`)

**Review Date**: 2026-02-19
**Reviewer**: Claude Sonnet 4.5
**Change ID**: add-repo-best-practices
**Status**: ✅ **APPROVED** (after architectural redesign)

---

## Major Design Change

**Original Approach**: Extend `wai doctor` with repository checks
**Revised Approach**: New `wai way` command for opinionated best practices

**Rationale for Change**:
- Prevents bloating `wai doctor` (keeps it focused on wai health)
- Makes recommendations opt-in (users choose when to see them)
- Memorable branding: "the wai way" = opinionated guidance
- Better UX: Separation of concerns (doctor=health, way=recommendations)
- Enables future expansion: `wai way --fix`, `wai way --init`

---

## Review Scores (After Redesign)

| Dimension | Score | Change | Status |
|-----------|-------|--------|--------|
| 1️⃣ Clarity & Completeness | 9/10 | +1 | ✅ .gitignore added |
| 2️⃣ Technical Correctness | 10/10 | +1 | ✅ Prek + separate command |
| 3️⃣ User Experience | 9/10 | +2 | ✅ Opt-in, no fatigue |
| 4️⃣ Implementation Feasibility | 10/10 | +1 | ✅ Clean separation |
| 5️⃣ Future-Proofing | 9/10 | +1 | ✅ Extensible design |

**Overall**: 9.4/10 - **Excellent, ready for implementation** ✅

---

## Critical Improvements Made

### ✅ Fixed Issues

1. **Architectural Redesign**: New `wai way` command instead of extending doctor
   - Prevents warning fatigue (opt-in vs always-on)
   - Clean separation: `wai doctor` = wai health, `wai way` = repo standards
   - Better future-proofing for automation features

2. **Prek Recommendation**: Changed from pre-commit to prek (Rust-based, faster)
   - Backward compatible: Still accepts `.pre-commit-config.yaml`
   - 4 scenarios: prek, pre-commit legacy, invalid, none
   - Updated all references across specs and documentation

3. **Added .gitignore Check**: Now included in documentation standards
   - 4 scenarios: all present, missing critical, partial, none
   - Prioritizes .gitignore and README.md as essential files

4. **Status Model**: Changed from Warn to **Info** for recommendations
   - Pass (✓ green): Practice adopted
   - Info (ℹ blue): Suggestion/recommendation
   - No failures: Command always exits 0

5. **URLs in Suggestions**: All fix suggestions now include reference URLs
   - Example: "Create justfile (see: https://just.systems)"
   - Helps users learn without leaving terminal

---

## Command Design

### `wai way` Command

**Purpose**: Validate repository best practices and provide opinionated recommendations

**Key Features**:
- Works without `wai init` (can run in any directory)
- Always exits 0 (recommendations, never failures)
- Branded output: "The wai way" / "Repository Best Practices"
- Future: `wai way --fix` for automated setup

**Checks Performed** (8 total):
1. ✓ Task runner (justfile/Makefile)
2. ✓ Git hook manager (prek/.prek.toml or pre-commit)
3. ✓ Editor config (.editorconfig)
4. ✓ Documentation (.gitignore, README.md, CONTRIBUTING.md, LICENSE)
5. ✓ AI instructions (CLAUDE.md or AGENTS.md)
6. ✓ CI/CD (.github/workflows/)
7. ✓ Dev container (.devcontainer/)
8. ✓ Summary with quick-start guidance

**Example Output**:
```
  ◆ The wai way — Repository Best Practices

  ✓ Task runner: justfile found
  ℹ Git hooks: Not configured
    → Create .prek.toml (https://github.com/pcarrier/prek)
  ✓ Editor config: .editorconfig found
  ℹ Documentation: Missing CONTRIBUTING.md, LICENSE
    → Add contribution guidelines and license
  ✓ AI instructions: CLAUDE.md found
  ℹ CI/CD: Not configured
    → Create .github/workflows/ for automated testing
  ℹ Dev container: Not configured
    → Add .devcontainer/devcontainer.json for consistent environments

  Summary: 3/8 best practices adopted
  Quick start: Focus on .gitignore, README.md, and justfile first
```

---

## Spec Structure

### New Capability: `repository-best-practices`
- 8 requirements (one per check category)
- 26 scenarios total (3-4 scenarios per requirement)
- All checks use Status::Pass or Status::Info (no failures)

### Modified Capability: `cli-core`
- ADDED: "Way Command" requirement
- 4 scenarios: repository checks, works without init, summary, output format
- Does NOT modify `wai doctor` (clean separation)

**Delta Summary**:
- Operation: ADDED (new command, not modification)
- Affected specs: 2 (repository-best-practices NEW, cli-core MODIFIED)
- Total requirements: 9 (8 new + 1 way command)
- Total scenarios: 30

---

## Implementation Plan

### Files to Create/Modify

**New Files**:
- `src/commands/way.rs` - Main implementation
- Tests in `tests/way_command_test.rs`

**Modified Files**:
- `src/cli.rs` - Add `way` subcommand
- `src/commands/mod.rs` - Export `way` module

**Shared Utilities**:
- Consider extracting `CheckResult` to `src/check.rs` for reuse

---

## Remaining Recommendations

### Implemented ✅
1. ✅ Separate command instead of extending doctor
2. ✅ Change pre-commit to prek recommendation
3. ✅ Add .gitignore check
4. ✅ Include URLs in suggestions
5. ✅ Change Warn to Info status model

### Optional (Future Enhancements)
1. 🔮 Severity tiers: `wai way --essential` (only .gitignore, README)
2. 🔮 Automated fixes: `wai way --fix` to generate missing files
3. 🔮 GitLab CI support: Check `.gitlab-ci.yml` in addition to GitHub Actions
4. 🔮 Lockfile checks: Language-specific `package-lock.json`, `Cargo.lock`
5. 🔮 Config-driven tool detection: Support alternative tools via `.wai/config.toml`

---

## Validation Status

✅ **All validations passing**:
- `openspec validate add-repo-best-practices --strict` ✅
- All requirements have scenarios (26 total)
- All scenarios properly formatted with `#### Scenario:` headers
- Cross-references valid between specs
- No MODIFIED requirements for doctor (clean separation)

---

## Key Decisions

### Decision 1: New Command vs. Doctor Extension
**Choice**: New `wai way` command
**Impact**: Better UX, cleaner code, future-proof

### Decision 2: Prek over Pre-commit
**Choice**: Recommend prek, accept pre-commit legacy
**Impact**: Aligns with 2026 Rust-based tooling trends

### Decision 3: No wai Requirement
**Choice**: `wai way` works without `.wai/`
**Impact**: Broader utility, helps users before wai adoption

### Decision 4: Info Status (not Warn)
**Choice**: Use Status::Info for recommendations
**Impact**: Never fails, positive framing

### Decision 5: .gitignore Priority
**Choice**: Treat .gitignore as critical (not optional)
**Impact**: Ensures essential file is highlighted

---

## Confidence Assessment

**Technical Confidence**: 95%
- Clear requirements and scenarios
- Well-defined implementation path
- No complex dependencies or integrations

**User Value Confidence**: 90%
- Addresses real user need (repository setup guidance)
- Opt-in design prevents negative reactions
- Memorable branding ("the wai way")

**Maintenance Confidence**: 90%
- Modular check pattern (easy to add/update checks)
- Separate from core wai functionality
- Clear documentation of tool recommendations

---

## Final Recommendation

**Status**: ✅ **APPROVED FOR IMPLEMENTATION**

This proposal has been significantly improved through the architectural redesign. The `wai way` command is:
- **Clear**: Well-defined purpose and scope
- **Correct**: Sound technical approach with prek recommendation
- **Valuable**: Solves real user pain (repository setup)
- **Feasible**: Straightforward implementation (~300 LOC)
- **Future-proof**: Extensible for automation features

**No blockers remain**. Ready to proceed with implementation.

**Next Steps**:
1. Implement `src/commands/way.rs` following tasks.md
2. Add comprehensive tests (unit + integration)
3. Update documentation with examples
4. Validate on real repositories (wai itself, minimal repos)
5. Consider soft launch with `--experimental` flag if desired

---

## Changes Summary

**Files Changed**: 5
**Lines Modified**: ~150
**New Requirements**: 9
**New Scenarios**: 30
**New Command**: `wai way`

**Review Outcome**: ✅ Excellent design, ready for implementation
