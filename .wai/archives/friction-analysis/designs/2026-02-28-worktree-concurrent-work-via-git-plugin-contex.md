Worktree concurrent work via git plugin

**Status: RESOLVED (2026-03-05)** — `wai sync --from-main` implemented in `src/commands/sync.rs`. `detect_main_worktree_root()` in `src/plugin/`. Integrated into `wai prime` (suggests sync when in worktree) and `wai close` (adds sync step to checklist). PARA-guided sharing model implemented: areas/ and resources/ sync, projects/ isolated per-branch.

## Context
Research on how to handle git worktrees for concurrent work, framed as a git plugin capability (parallel to how beads uses --from-main).

## Key Insight: Beads vs Wai Storage Model

Beads (.beads/) is gitignored → exists once, shared across all worktrees via redirect mechanism.
Wai (.wai/) is committed to git → each worktree gets its own copy from whatever branch it checks out.

This is not a bug — .wai/ artifacts ON a branch represent the reasoning that led to that branch's code.

## PARA-Guided Sharing Model

The PARA structure already encodes what should be shared vs isolated:

| PARA dir     | Scope       | Rationale |
|---|---|---|
| projects/    | Per-worktree | Feature-specific work state, handoffs, phase |
| areas/       | Shared       | Ongoing responsibilities, cross-cutting concerns |
| resources/   | Shared       | Skills, rules, templates — universal |
| archives/    | Historical   | Merged already, doesn't matter |

This mirrors beads exactly: issues are global state, working changes are per-branch.

## Proposed Design: Git Plugin Gains Worktree Awareness

### 1. Worktree Detection (git plugin)
The git plugin compares git rev-parse --git-dir vs --git-common-dir.
If they differ → we're in an additional worktree.
Main worktree root = dirname of git rev-parse --git-common-dir.

Note: the git plugin's directory detector (.git exists) already works in worktrees because
.git is a FILE in worktrees (not a dir) but .exists() returns true for both.

### 2.  command
New command, parallel to bd sync --from-main:
- Requires: git plugin detected + currently in a worktree
- Finds: main worktree root via git-common-dir
- Copies: .wai/areas/** and .wai/resources/** from main worktree's checkout
- Skips: .wai/projects/** (per-branch, intentionally isolated)
- Reports: what files were synced

### 3. 
  [2m○[0m No projections configured.
  [2m→[0m Edit .wai/resources/agent-config/.projections.yml to add projections
Non-destructive check: are areas/resources behind main worktree?
(compares file mtimes or content hashes, does not modify)

### 4. Session integration (parallel to beads checklist injection)
- wai prime: when in worktree, show '→ wai sync --from-main' suggestion
- wai close: add sync step to checklist when git plugin detects worktree
  (same pattern as how beads plugin adds 'bd sync --from-main' to the checklist)

## Implementation Notes

### find_project_root() — No Change Needed
Currently walks up from cwd looking for .wai/ — this already works in worktrees since
.wai/ is committed and present in each checkout. No modification needed.

### Git Plugin Enhancement
Add a helper function in plugin.rs or a new git.rs module:
  fn detect_worktree_info(project_root: &Path) -> Option<WorktreeInfo>
  
  struct WorktreeInfo {
    is_worktree: bool,
    main_worktree_root: PathBuf,  // dirname(git-common-dir)
    branch: String,               // git rev-parse --abbrev-ref HEAD
  }

### Hook Injection
on_status hook could run: git worktree list --porcelain
→ inject_as: 'worktree_info'

This gives agents visibility into which worktrees exist and what they're working on —
useful for coordinating concurrent work.

### wai sync Implementation
Simple file-copy with conflict detection:
1. Run detect_worktree_info() → get main_worktree_root
2. For each file in main's .wai/areas/ and .wai/resources/:
   a. If not in current worktree: copy
   b. If in current worktree but identical: skip  
   c. If in current worktree and different: report conflict, skip (user resolves)
3. Report summary

## Comparison Table

| Concern              | Beads approach        | Wai approach (proposed) |
|---|---|---|
| Storage location     | Gitignored, once      | Committed, per-checkout |
| Shared data          | All issues            | areas/ + resources/     |
| Per-branch data      | (none, all shared)    | projects/               |
| Sync mechanism       | bd sync --from-main   | wai sync --from-main    |
| Worktree detection   | git hooks (.git dir vs file) | git plugin (rev-parse) |
| Session checklist    | In close.rs when beads detected | In close.rs when worktree detected |

## Open Questions
1. Should wai sync --from-main be part of wai prime (auto) or explicit only?
   Recommendation: explicit only (mirrors beads, gives user control)
2. Conflict resolution: skip-and-warn or prompt?
   Recommendation: skip-and-warn (non-destructive default)
3. Should wai status show worktree context (which worktrees exist, their branches)?
   Recommendation: yes, via git plugin hook injection (git worktree list --porcelain)

