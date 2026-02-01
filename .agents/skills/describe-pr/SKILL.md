---
name: describe-pr
description: Generate comprehensive pull request descriptions from actual code changes. Analyzes diffs, explains the "why", and produces structured PR documentation.
---

# Generate PR Description

Create a comprehensive pull request description based on the actual changes in the PR.

## Process

### Step 1: Identify the PR

```bash
gh pr view --json url,number,title,state,baseRefName
```

### Step 2: Gather PR Information

```bash
gh pr diff {number}
gh pr view {number} --json commits --jq '.commits[] | "\(.oid[0:7]) \(.messageHeadline)"'
gh pr view {number} --json url,title,number,state,baseRefName,additions,deletions
```

### Step 3: Analyze Changes Deeply

Review all changes and understand:

**What changed:** Files modified, added, removed; key functions affected
**Why it changed:** What problem this solves
**Impact:** User-facing changes, API changes, breaking changes, security considerations

### Step 4: Generate Description

```markdown
## Summary

[2-3 sentence overview of what this PR does and why it's needed]

## Changes

**Key changes:**
- [Specific change 1 with reasoning]
- [Specific change 2 with reasoning]

**Files changed:**
- `path/to/file1.ext` - [What changed and why]

## Motivation

[Explain the problem this PR solves]

## Implementation Details

[Explain key implementation decisions and trade-offs]

## Testing

**Automated tests:**
- [ ] Unit tests pass
- [ ] Integration tests pass

**Manual testing:**
- [ ] [Specific manual test 1]

## Breaking Changes

[If any breaking changes, list them here with migration guidance]

## Follow-up Work

- [ ] [Follow-up task 1]
```

### Step 5: Update PR

Upon approval:

```bash
gh pr edit {number} --body-file /tmp/pr-description.md
```

## Guidelines

1. **Focus on "why" not just "what"** - Diff shows what, description explains why
2. **Be specific** - Vague descriptions aren't helpful
3. **Highlight breaking changes** - Make them impossible to miss
