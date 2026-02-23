# Rule of 5 Universal Review - Specification Files

**Review Date**: 2026-02-19
**Reviewer**: Claude Sonnet 4.5
**Specs Reviewed**:
- `cli-core/spec.md` (Way Command requirement)
- `repository-best-practices/spec.md` (8 requirements, 26 scenarios)

---

## Review Methodology

Evaluating spec quality across five dimensions:
1. **Clarity & Completeness** - Unambiguous requirements, complete scenarios
2. **Technical Correctness** - Accurate, implementable, consistent
3. **User Experience & Value** - Testable, valuable, user-centric
4. **Implementation Feasibility** - Developer-friendly, actionable
5. **Future-Proofing** - Maintainable, extensible, evolvable

---

## 1️⃣ Clarity & Completeness

### ✅ Strengths

**CLI-Core Spec (Way Command)**:
- ✅ Clear requirement statement with memorable branding
- ✅ 4 distinct scenarios covering key behaviors
- ✅ Cross-reference to repository-best-practices spec
- ✅ Explicit exit code behavior (always 0)

**Repository-Best-Practices Spec**:
- ✅ Strong purpose and problem statement
- ✅ Clear design rationale (guidance not enforcement)
- ✅ Explicit non-goals section
- ✅ Consistent pattern across all 8 requirements
- ✅ Each requirement has 2-4 scenarios (good coverage)

### ❌ Critical Issues

1. **Status::Info Not Defined** (cli-core:12, repository-best-practices:13)
   - Specs use `Status::Info` but this enum value doesn't exist in doctor.rs
   - Current doctor.rs has: `Status::Pass`, `Status::Warn`, `Status::Fail`
   - **Impact**: Implementation will fail to compile
   - **Fix**: Either add Status::Info to doctor.rs or change specs to use Status::Warn

2. **"Higher Priority" Undefined** (repository-best-practices:112)
   - Scenario says "reports Status::Info with higher priority"
   - No mechanism defined for priority levels
   - **Impact**: Implementer doesn't know how to implement priority
   - **Fix**: Define priority mechanism or remove "higher priority" language

3. **Missing Scenario: Multiple Files** (repository-best-practices:160)
   - CI/CD check says "at least one .yml or .yaml file"
   - No scenario for directory exists but has no workflow files
   - **Impact**: Edge case not covered
   - **Fix**: Add scenario for empty .github/workflows/ directory

4. **Inconsistent Message Formats** (throughout)
   - Some: `"Task runner: {filename} found"` (has colon)
   - Others: `"Git hooks configured (prek)"` (no colon)
   - Others: `"Documentation: Complete (.gitignore, ...)"`
   - **Impact**: Inconsistent UX
   - **Fix**: Standardize format: `"Category: Status"` or `"Category: Details"`

### ⚠️ Warnings

1. **Vague "Optional URLs"** (cli-core:33)
   - Says "optional URLs for each suggestion"
   - Doesn't specify when URLs are required vs optional
   - Better: "URLs for all suggestions" or list which checks include URLs

2. **Check Grouping Scope Confusion** (repository-best-practices:196)
   - Requirement says "group under Repository Standards section in doctor output"
   - But these checks are for `wai way`, not `wai doctor`
   - **Impact**: Wrong command referenced
   - **Fix**: Change to "group under Repository Standards section in way output"

3. **{missing_files} Placeholder** (repository-best-practices:121)
   - Uses `{missing_files}` but doesn't specify format
   - Is it comma-separated? One per line? With "and"?
   - Better: Specify format, e.g., "CONTRIBUTING.md and LICENSE"

### 📊 Completeness Score: 7/10

**Missing scenarios**:
- Empty directories (.github/workflows/ exists but no files)
- Invalid file formats (e.g., .editorconfig with syntax errors)
- Symbolic links to files outside repository
- Multiple AI instruction files (both CLAUDE.md and AGENTS.md exist)

---

## 2️⃣ Technical Correctness

### ✅ Strengths

- ✅ File existence checks are straightforward and reliable
- ✅ TOML/YAML validation mentioned appropriately
- ✅ Cross-references between specs use correct paths
- ✅ Exit code behavior clearly specified (always 0)
- ✅ Consistent use of SHALL for normative requirements

### ❌ Critical Issues

1. **Status::Info Doesn't Exist** (BLOCKING)
   - Used throughout: cli-core:12, repository-best-practices:45, 69, 76, 94, etc.
   - Current doctor.rs enum:
     ```rust
     enum Status {
         Pass,
         Warn,
         Fail,
     }
     ```
   - **Impact**: Code won't compile
   - **Fix Options**:
     - A) Add `Info` variant to Status enum (requires modifying doctor.rs)
     - B) Change all Status::Info to Status::Warn in specs
     - C) Create separate WayStatus enum for wai way command

2. **TOML Validation Complexity** (repository-best-practices:57)
   - Says "is valid TOML" but doesn't specify validation depth
   - Does it parse the entire file or just check syntax?
   - Does it validate prek-specific schema?
   - **Impact**: Over-specified requirement that's hard to implement
   - **Fix**: Change to "exists" or "parses as TOML" (no schema validation)

3. **Directory + File Check Ambiguity** (repository-best-practices:160, 177)
   - CI/CD: Checks `.github/workflows/` directory "with at least one .yml or .yaml file"
   - Dev container: Checks `.devcontainer/` directory "with devcontainer.json"
   - **Question**: What if directory exists but is empty? Pass or Info?
   - **Impact**: Ambiguous behavior
   - **Fix**: Add explicit scenario for empty directory case

### ⚠️ Warnings

1. **Pattern Inconsistency**: Some checks validate file format (TOML), others don't (EditorConfig)
   - Git hooks: Validates `.prek.toml` is valid TOML
   - EditorConfig: Only checks `.editorconfig` exists, no validation
   - **Question**: Why validate some but not others?
   - **Recommendation**: Either validate all or validate none (prefer none for simplicity)

2. **"Current directory" vs "Project root"** (throughout)
   - Most scenarios say "current directory"
   - But design says `wai way` can run anywhere
   - What if user runs `wai way` from subdirectory?
   - **Recommendation**: Clarify: checks current working directory, not git root

### 📊 Technical Correctness Score: 6/10

**Deductions**:
- -3 for Status::Info not existing (blocking issue)
- -1 for validation inconsistency (TOML vs no validation)

---

## 3️⃣ User Experience & Value

### ✅ Strengths

- ✅ Scenarios are user-centric (describe user actions, not implementation)
- ✅ Clear messages that users will see
- ✅ Actionable suggestions with URLs
- ✅ "Quick-start priorities" mentioned (helpful for beginners)
- ✅ Summary format specified ("X/Y best practices adopted")

### ❌ Critical Issues

1. **No Scenario for "All Info" Output** (cli-core)
   - What does output look like when all 8 checks are Status::Info?
   - Users will hit this case on fresh repos
   - **Impact**: Most common use case not demonstrated
   - **Fix**: Add scenario showing minimal repo output

2. **"Consider migrating to prek" Message** (repository-best-practices:64)
   - Status::Pass but message suggests improvement
   - Confusing: Green checkmark but "you should change this"
   - **Impact**: Mixed signal to users
   - **Fix**: Either Status::Info with migration suggestion, or Status::Pass with neutral message

3. **Unclear Priority System** (repository-best-practices:112)
   - Says "higher priority" but users won't see priority in output
   - No specification of how priority affects display order or styling
   - **Impact**: Users may miss critical files
   - **Fix**: Define visual distinction (e.g., "⚠️ Critical" vs "ℹ️ Recommended")

### ⚠️ Warnings

1. **Message Verbosity** - Some messages are long
   - "Create .devcontainer/devcontainer.json for consistent development environments (VS Code, GitHub Codespaces)"
   - May wrap awkwardly in terminal
   - **Recommendation**: Keep messages under 80 characters

2. **No Success Message** - No scenario for "all checks pass"
   - What does a perfect repo output look like?
   - Users want positive reinforcement
   - **Recommendation**: Add scenario and celebratory message

3. **URL Formatting Not Specified** (cli-core:33)
   - Says "includes... optional URLs"
   - But how? Inline? On separate line? Colored?
   - Example: `→ Create justfile (see: https://just.systems)`
   - **Recommendation**: Add to scenario: "URLs appear in parentheses after suggestion"

### 📊 UX & Value Score: 7/10

**Strengths**: User-centric, actionable
**Weaknesses**: Missing common-case scenarios, unclear priority

---

## 4️⃣ Implementation Feasibility

### ✅ Strengths

- ✅ Requirements map directly to simple file checks
- ✅ No external dependencies needed
- ✅ Scenarios provide clear test cases
- ✅ Consistent pattern makes implementation straightforward
- ✅ Cross-references enable code reuse

### ❌ Critical Issues

1. **Status::Info Implementation Gap** (BLOCKING)
   - Requires either:
     - Modifying existing doctor.rs (risky, affects other code)
     - Creating new status enum (duplication)
     - Using Status::Warn (misleading name)
   - **Impact**: Design decision needed before implementation
   - **Recommendation**: Create `WayStatus` enum separate from doctor's `Status`

2. **"At Least One File" Check** (repository-best-practices:160)
   - Requires directory traversal and extension checking
   - More complex than other checks (simple file existence)
   - Scenario: `.github/workflows/` with subdirectories or non-workflow files
   - **Impact**: Implementation complexity
   - **Recommendation**: Simplify to "directory exists" or be more specific

3. **Multiple File Tracking** (repository-best-practices:105)
   - Documentation check needs to track 4 different files
   - Distinguish "missing critical" vs "partial" vs "none"
   - More complex logic than other checks
   - **Impact**: More LOC, more test cases
   - **Current**: Acceptable, but note complexity

### ⚠️ Warnings

1. **TOML Parsing Adds Dependency**
   - Checking "is valid TOML" requires toml crate
   - But wai already uses TOML for config.toml
   - **Impact**: Minimal (dependency already exists)
   - **Note**: Ensure consistent TOML crate usage

2. **Message Formatting Consistency** (throughout)
   - Different message formats require different string building
   - Example: `"Task runner: justfile found"` vs `"Git hooks: prek config invalid"`
   - **Recommendation**: Create message builder helper: `format_check_message(category, status, detail)`

3. **Test Coverage** (implicit)
   - 26 scenarios = 26 minimum test cases
   - Plus edge cases (empty dirs, symlinks, etc.)
   - ~40-50 tests total
   - **Impact**: Significant test writing effort
   - **Mitigation**: Use table-driven tests

### 📊 Implementation Feasibility Score: 7/10

**Clear path forward, but Status::Info decision needed first**

---

## 5️⃣ Future-Proofing & Maintenance

### ✅ Strengths

- ✅ Modular requirement structure (easy to add/remove checks)
- ✅ Non-goals explicitly stated (prevents scope creep)
- ✅ URLs in suggestions enable updating links without code changes
- ✅ Tool-agnostic where possible (justfile OR Makefile, prek OR pre-commit)
- ✅ Design rationale documented (helps future maintainers)

### ❌ Critical Issues

1. **Hardcoded File Names** (throughout)
   - `justfile`, `Makefile`, `.prek.toml`, `.pre-commit-config.yaml`, etc.
   - If new tool emerges (e.g., taskfile, lefthook), specs must change
   - **Impact**: Specs tightly coupled to 2026 tooling
   - **Fix**: Consider "extensible check" pattern in future version

2. **No Versioning** (entire spec)
   - No indication these are "2026 best practices"
   - In 2027+, how do we know which recommendations are outdated?
   - **Impact**: Specs may become stale without obvious indication
   - **Fix**: Add metadata: "Based on 2026 industry standards, review annually"

3. **GitHub-Only CI/CD** (repository-best-practices:154)
   - Only checks GitHub Actions
   - GitLab, CircleCI, Jenkins, Drone, etc. not considered
   - **Impact**: Limited applicability for non-GitHub repos
   - **Fix**: Add note: "Future: support other CI systems" or make extensible

### ⚠️ Warnings

1. **Check Grouping Requirement** (repository-best-practices:194)
   - Says checks are "grouped" but doesn't specify grouping mechanism
   - What if we want to reorganize grouping in future?
   - **Recommendation**: Make grouping a display concern, not requirement

2. **Missing Deprecation Path**
   - No mechanism to mark checks as deprecated
   - Example: If pre-commit becomes fully obsolete
   - **Recommendation**: Add Status::Deprecated or note in design

3. **Tool URLs May Change**
   - Hardcoded URLs: https://just.systems, https://editorconfig.org
   - What if sites move or projects are abandoned?
   - **Mitigation**: Specs are living documents, can update URLs

### 📊 Future-Proofing Score: 7/10

**Good modular structure, but needs versioning and deprecation planning**

---

## Overall Spec Quality Assessment

| Dimension | Score | Grade | Key Issue |
|-----------|-------|-------|-----------|
| 1️⃣ Clarity & Completeness | 7/10 | B | Status::Info undefined, missing edge cases |
| 2️⃣ Technical Correctness | 6/10 | C+ | **BLOCKING**: Status::Info doesn't exist |
| 3️⃣ User Experience | 7/10 | B | Missing common scenarios, unclear priority |
| 4️⃣ Implementation Feasibility | 7/10 | B | Status enum decision needed |
| 5️⃣ Future-Proofing | 7/10 | B | Needs versioning, hardcoded tools |

**Overall Score: 6.8/10 (B-)**

**Status**: ⚠️ **CONDITIONAL APPROVAL** - Fix Status::Info before implementation

---

## Critical Fixes Required

### 🔴 BLOCKING (Must Fix Before Implementation)

1. **Status::Info Definition**
   - **Problem**: Used throughout specs but doesn't exist in codebase
   - **Options**:
     - A) Add `Info` variant to existing `Status` enum in doctor.rs
     - B) Create separate `WayStatus` enum for wai way
     - C) Change all `Status::Info` to `Status::Warn` in specs
   - **Recommendation**: Option B (WayStatus enum) - cleanest separation

2. **Fix Check Grouping Requirement** (repository-best-practices:196)
   - **Problem**: Says "in doctor output" but should be "in way output"
   - **Fix**: Change line 196: `s/doctor output/way output/`

### 🟡 HIGH PRIORITY (Should Fix)

3. **Add Empty Directory Scenarios**
   - CI/CD: What if `.github/workflows/` exists but is empty?
   - Dev container: What if `.devcontainer/` exists without devcontainer.json?
   - **Impact**: Ambiguous behavior
   - **Fix**: Add explicit scenarios for these cases

4. **Standardize Message Format**
   - **Problem**: Inconsistent formats across checks
   - **Fix**: Define standard: `"Category: Status (details)"`
   - Examples:
     - `"Task runner: justfile found"`
     - `"Git hooks: prek configured"`
     - `"Documentation: Missing .gitignore, README.md"`

5. **Clarify TOML Validation Depth** (repository-best-practices:57)
   - **Problem**: "is valid TOML" over-specifies
   - **Fix**: Change to "exists" or "parses as TOML (syntax only)"

### 🟢 NICE TO HAVE (Recommended)

6. **Add "All Pass" Scenario** - Show success case
7. **Add "All Info" Scenario** - Show fresh repo case
8. **Specify URL Format** - How URLs appear in output
9. **Add Versioning Metadata** - "Based on 2026 standards"
10. **Define Priority Mechanism** - How "higher priority" works

---

## Specific Line-by-Line Issues

### cli-core/spec.md

| Line | Issue | Severity | Fix |
|------|-------|----------|-----|
| 12 | `Status::Info` doesn't exist | 🔴 Critical | Define WayStatus enum or use Status::Warn |
| 33 | "optional URLs" vague | 🟡 Medium | Specify when URLs are included |

### repository-best-practices/spec.md

| Line | Issue | Severity | Fix |
|------|-------|----------|-----|
| 13 | Says `Status::Warn` should be `Status::Info` | 🔴 Critical | Inconsistent with rest of spec |
| 45, 69, 76, 94, etc. | `Status::Info` doesn't exist | 🔴 Critical | Define WayStatus enum |
| 57 | "is valid TOML" over-specified | 🟡 Medium | Change to "parses as TOML" |
| 64 | Confusing "consider migrating" with Status::Pass | 🟡 Medium | Use Status::Info or neutral message |
| 112 | "higher priority" undefined | 🟡 Medium | Define priority mechanism |
| 121 | `{missing_files}` format unspecified | 🟢 Low | Specify format |
| 160 | "at least one .yml or .yaml file" complex | 🟡 Medium | Add scenario for empty directory |
| 196 | Says "doctor output" should be "way output" | 🔴 Critical | Change command reference |

---

## Recommendations

### Immediate Actions (Before Implementation)

1. **Decision Required**: Choose Status enum approach
   - Recommended: Create `WayStatus` enum in way.rs
   - Keeps `wai way` isolated from `wai doctor`
   - Cleaner separation of concerns

2. **Update Specs**: Fix Status::Info references
   - If using WayStatus: Update all Status:: to WayStatus::
   - If using Status::Warn: Update all Info to Warn

3. **Fix Critical Errors**: Line 196, line 13 inconsistency

### Before Finalization

4. **Add Missing Scenarios**:
   - Empty directories
   - All checks pass
   - All checks info
   - Both CLAUDE.md and AGENTS.md exist

5. **Standardize Messages**: Create format guide

6. **Add Versioning**: Note "2026 best practices"

---

## Conclusion

**Spec Quality**: 6.8/10 (B-) - Good foundation with critical issues

**Status**: ⚠️ **CONDITIONAL APPROVAL**
- Specs are well-structured and mostly clear
- One **BLOCKING** issue: Status::Info doesn't exist
- Several medium-priority improvements needed

**Confidence**: 80% (once Status::Info is resolved)

**Next Steps**:
1. Decide on Status enum approach (WayStatus recommended)
2. Update specs to fix Status::Info references
3. Fix critical errors (lines 196, 13)
4. Add missing scenarios for completeness
5. Standardize message formats
6. Re-validate with `openspec validate --strict`

**Timeline**: ~2-3 hours to fix all critical and high-priority issues

---

**Review Completed**: 2026-02-19
**Reviewer**: Claude Sonnet 4.5
**Recommendation**: Fix Status::Info, then proceed to implementation
