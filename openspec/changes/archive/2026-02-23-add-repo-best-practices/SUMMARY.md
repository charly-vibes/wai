# OpenSpec Proposal: `wai way` Command

**Change ID**: add-repo-best-practices  
**Status**: ✅ Ready for Implementation  
**Review Score**: 9.4/10

---

## What is `wai way`?

A new command that validates repository best practices and provides opinionated recommendations based on 2026 industry standards.

```bash
$ wai way

  ◆ The wai way — Repository Best Practices

  ✓ Task runner: justfile found
  ℹ Git hooks: Not configured
    → Create .prek.toml (https://github.com/pcarrier/prek)
  ✓ Editor config: .editorconfig found
  ℹ Documentation: Missing .gitignore, CONTRIBUTING.md
    → Start with .gitignore and README.md (essential)
  ✓ AI instructions: CLAUDE.md found
  ℹ CI/CD: Not configured
  ℹ Dev container: Not configured

  Summary: 3/8 best practices adopted
  Quick start: Focus on .gitignore, README.md, and justfile
```

---

## Key Features

✅ **Opt-in recommendations** - Run when you want guidance  
✅ **Never fails** - Always exits 0 (info/pass only)  
✅ **Works anywhere** - No `.wai/` initialization required  
✅ **Memorable branding** - "The wai way" = opinionated guidance  
✅ **Research-backed** - Based on 2026 industry best practices  
✅ **Future-ready** - Foundation for `wai way --fix` automation  

---

## Checks Performed (8 total)

| Check | Files | Recommendation |
|-------|-------|----------------|
| 1. Task runner | `justfile` or `Makefile` | justfile (modern standard) |
| 2. Git hooks | `.prek.toml` or `.pre-commit-config.yaml` | prek (Rust-based, faster) |
| 3. Editor config | `.editorconfig` | Standard for 40+ editors |
| 4. Documentation | `.gitignore`, `README.md`, `CONTRIBUTING.md`, `LICENSE` | Essential files |
| 5. AI instructions | `CLAUDE.md` or `AGENTS.md` | CLAUDE.md (wider adoption) |
| 6. CI/CD | `.github/workflows/*.yml` | GitHub Actions |
| 7. Dev container | `.devcontainer/` or `.devcontainer.json` | For environment consistency |
| 8. Summary | - | Quick-start priorities |

---

## Why Not Extend `wai doctor`?

**Original idea**: Add repository checks to `wai doctor`  
**Problem**: Would add 7-9 warnings to every doctor run, mixing concerns

**Better solution**: Separate `wai way` command
- Keeps `wai doctor` focused on wai-specific health
- Makes recommendations opt-in (no warning fatigue)
- Memorable branding ("the wai way")
- Enables future features (`wai way --fix`, `wai way --init`)

---

## Design Highlights

### Prek over Pre-commit
- **Prek**: Rust-based, faster, modern (2026 trend)
- **Backward compatible**: Still accepts `.pre-commit-config.yaml`
- 4 scenarios: prek, pre-commit legacy, invalid, none

### .gitignore as Critical
- Elevated from "nice to have" to "essential"
- Prioritized with README.md in suggestions
- 4 scenarios: complete, missing critical, partial, none

### Info Status (not Warn)
- **Pass** (✓): Practice adopted
- **Info** (ℹ): Recommendation/suggestion  
- No failures: Command always exits 0

### URLs in Suggestions
All recommendations include reference links:
- "Create justfile (see: https://just.systems)"
- "Create .prek.toml (https://github.com/pcarrier/prek)"
- "Create .editorconfig (https://editorconfig.org)"

---

## Implementation Scope

### New Files
- `src/commands/way.rs` (~300 LOC)
- `tests/way_command_test.rs` (unit + integration)

### Modified Files
- `src/cli.rs` (add `way` subcommand)
- `src/commands/mod.rs` (export `way` module)

### Specs
- **NEW**: `repository-best-practices` (8 requirements, 26 scenarios)
- **ADDED**: `cli-core` Way Command requirement (4 scenarios)

---

## Validation

✅ `openspec validate add-repo-best-practices --strict` **PASSING**  
✅ 30 scenarios, all properly formatted  
✅ Cross-references valid  
✅ No modifications to `wai doctor` (clean separation)  

---

## Rule of 5 Review Results

| Dimension | Score | Status |
|-----------|-------|--------|
| Clarity & Completeness | 9/10 | ✅ .gitignore added |
| Technical Correctness | 10/10 | ✅ Prek + clean design |
| User Experience | 9/10 | ✅ Opt-in, no fatigue |
| Implementation Feasibility | 10/10 | ✅ Straightforward |
| Future-Proofing | 9/10 | ✅ Extensible |

**Overall**: 9.4/10 - **Ready for implementation**

---

## Research Foundation

Based on comprehensive 2026 research document (2,587 lines):
- 40+ source references from official docs and industry articles
- Developer workflow standardization (justfile, prek)
- Development environment consistency (devcontainers, EditorConfig)
- CI/CD automation patterns (GitHub Actions, act tool)
- Documentation standards (README, ADRs, CLAUDE.md)
- Code quality tools (Ruff, cargo fmt, Prettier)

---

## Future Enhancements

🔮 **Automation**: `wai way --fix` to generate missing files  
🔮 **Filtering**: `wai way --essential` (only critical files)  
🔮 **Multi-platform CI**: Support GitLab CI, CircleCI  
🔮 **Lockfiles**: Check `package-lock.json`, `Cargo.lock`  
🔮 **Config-driven**: Custom tool detection via `.wai/config.toml`  

---

## Next Steps

1. ✅ **Approve proposal** (ready now)
2. 🔨 **Implement** following `tasks.md` (35 tasks)
3. 🧪 **Test** on wai repo + minimal repos
4. 📚 **Document** with examples and screenshots
5. 🚀 **Release** as part of next wai version

---

## Files in This Proposal

```
openspec/changes/add-repo-best-practices/
├── SUMMARY.md              ← This file
├── REVIEW.md              ← Rule of 5 review results
├── proposal.md            ← Full proposal (why, what, impact)
├── design.md              ← Design decisions and rationale
├── tasks.md               ← 35 implementation tasks
└── specs/
    ├── cli-core/
    │   └── spec.md        ← ADDED: Way Command requirement
    └── repository-best-practices/
        └── spec.md        ← NEW: 8 requirements, 26 scenarios
```

---

**Confidence**: High (95%)  
**Recommendation**: ✅ Proceed with implementation  
**Timeline**: ~1-2 days for complete implementation and testing
