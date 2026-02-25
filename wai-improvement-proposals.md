# wai Improvement Proposals

> Generated from direct use during bichos session (2026-02-25).
> Context: implementing a 3-stage agent pipeline pattern (gather → create/implement → review)
> where each stage is an independent subagent and wai artifacts are the shared memory between stages.

---

## Summary

wai is the right abstraction for multi-session, multi-agent coordination. The PARA structure,
phase tracking, and artifact semantics (research / plan / design) map cleanly onto how agents
actually think. The proposals below are not about changing the model — they are about closing
the gap between the intended pattern and the friction encountered when using it.

Eight friction points are documented. They are ordered by impact.

---

## P1 — Skill names should support category/action hierarchy

**Status**: Blocking for Claude Code users

### Problem

Claude Code invokes skills via the path of their command file:
`.claude/commands/issue/gather.md` → invoked as `/issue:gather`

wai skill names reject `:` and `/`:
```
Error: Invalid character ':' at position 5. Only lowercase letters, digits, and hyphens allowed.
```

This forces a naming mismatch:
- wai name: `issue-gather`
- Claude Code invocation: `/issue:gather`
- File must live at: `.claude/commands/issue/gather.md`

The result is two separate naming systems with no mechanical link between them.

### Proposed Fix

Allow skill names with a single `/` separator for category grouping:

```bash
wai resource add skill issue/gather
wai resource add skill impl/run
```

Storage: `.wai/resources/agent-config/skills/issue/gather/SKILL.md`
Claude Code projection target: `.claude/commands/issue/gather.md`

Flat names (`my-skill`) continue to work unchanged. Only `/` (one level) is added.

### Benefit

Category becomes first-class. `wai resource list skills` can group by category.
The wai name directly predicts the Claude Code invocation path.

---

## P2 — wai sync projections must work without pre-created directories

**Status**: Blocking — projections are effectively unusable today

### Problem

With `strategy: symlink`, `wai sync` reports `Symlinked → <target>` but:

1. If the parent directory does not exist → creates an empty **directory** at the target path
   instead of a symlink. Silent, no error.
2. If the parent directory is created manually first → still creates an empty directory.
3. No `--dry-run` flag to preview what would happen before committing.

Workaround: write files directly to `.claude/commands/`, maintaining two copies.

### Proposed Fix

1. `wai sync` should create missing parent directories automatically (like `mkdir -p`).
2. `strategy: symlink` must create an actual symlink (not a directory).
3. Add `wai sync --dry-run` to preview operations.
4. Add `strategy: copy` as a fallback for systems where symlinks are unavailable.

### Reproducible Steps

```yaml
# .wai/resources/agent-config/.projections.yml
projections:
  - source: skills/issue-gather/SKILL.md
    target: ../../../.claude/commands/issue/gather.md
    strategy: symlink
```

```bash
# Before mkdir: wai sync creates a directory at .claude/commands/issue/gather.md
# After mkdir .claude/commands/issue/: wai sync still creates an empty directory
wai sync   # reports success, creates nothing useful
```

---

## P3 — Add a built-in Claude Code projection target

**Status**: High value, blocks portability across projects

### Problem

Every project that wants to use wai skills in Claude Code must:
1. Manually create the right directory structure under `.claude/commands/`
2. Maintain two copies of each skill (SKILL.md and the command file)
3. Keep frontmatter in sync between both formats

wai SKILL.md frontmatter:
```yaml
---
name: issue-gather
description: "..."
---
```

Claude Code command frontmatter:
```yaml
---
name: Issue: Gather
description: "..."
category: Issue Pipeline
tags: [issues, research, wai]
---
```

Two formats, different field names, manual sync.

### Proposed Fix

Add a built-in `claude-code` projection target that:
1. Maps `skills/<category>/<action>/SKILL.md` → `.claude/commands/<category>/<action>.md`
2. Translates wai frontmatter to Claude Code frontmatter automatically
3. Creates all parent directories

```yaml
projections:
  - source: skills/
    target: claude-code     # built-in target, no path needed
    strategy: generate      # generates correct format, not a raw copy
```

Or as a first-class command:

```bash
wai sync --to claude-code   # sync all skills to .claude/commands/
```

---

## P4 — Pipeline resource type

**Status**: Medium — today encoded in CLAUDE.md prose, not machine-readable

### Problem

The 3-stage pattern (gather → create/implement → review) had to be documented entirely
in CLAUDE.md as unstructured prose. wai has no concept of:
- An ordered sequence of skills that form a unit
- What each stage produces (artifact type) and what the next stage expects
- The status of a pipeline run ("stage 2 of 3 complete for topic X")

This means:
- No validation that stages are connected correctly
- No runtime guidance (agent must re-read CLAUDE.md to understand the pipeline)
- Cannot be reused across projects without copy-pasting CLAUDE.md

### Proposed Fix

Add a `pipeline` resource type:

```bash
wai pipeline create issue-pipeline \
  --stages="issue/gather:research,issue/create:tickets,issue/review:audit"

wai pipeline create impl-pipeline \
  --stages="impl/gather:plan,impl/run:code,impl/review:verdict"
```

Each stage declaration: `<skill-name>:<output-artifact-type>`

Runtime:
```bash
wai pipeline run issue-pipeline --topic="ant-forager"
# → creates a pipeline run ID
# → stage 1 output artifact is tagged with run ID
# → stage 2 skill receives run ID and can fetch stage 1's artifact precisely
```

Status:
```bash
wai pipeline status issue-pipeline
# Stage 1 (issue/gather): ✓ complete — artifact: research/2026-02-25-ant-forager.md
# Stage 2 (issue/create): ✓ complete — artifact: none (output is bd tickets)
# Stage 3 (issue/review): ○ pending
```

---

## P5 — Precise artifact retrieval between pipeline stages

**Status**: Medium — current workaround works but is fragile

### Problem

Stage N saves an artifact, Stage N+1 must find it. Current pattern:

```bash
# Stage 1 saves:
wai add research "TOPIC: ant-forager\n<findings>"

# Stage 2 retrieves:
wai search "ant-forager"   # fuzzy, might match unrelated artifacts
```

Problems:
- `wai search` is full-text, not structured. A topic word like "performance" could match
  dozens of unrelated artifacts.
- No way to say "give me the most recent research artifact for this pipeline run"
- No artifact IDs exposed in CLI output (can't reference by ID in the next command)

### Proposed Fix

**Option A — Tags on all artifact types** (minimal change)

Today `--tags` only works on `wai add research`. Extend to all artifact types:

```bash
wai add plan "..." --tag="topic:ant-forager" --tag="pipeline:impl"
wai add design "..." --tag="topic:ant-forager"
```

Then retrieve precisely:
```bash
wai search "ant-forager" --tag="pipeline:impl" --type=plan --latest
```

**Option B — Artifact IDs in CLI output** (small change)

Make every `wai add` command print the artifact ID:
```
✓ Created research artifact: bichos-res-20260225-ant-forager (2026-02-25)
```

So Stage 2 can reference it explicitly without searching.

**Option C — Pipeline run IDs** (requires P4)

If pipelines are implemented, run IDs tie stages together automatically.

Recommendation: implement Option A now (low effort), Option C when P4 lands.

---

## P6 — Skill templates

**Status**: Low — friction mainly on initial setup

### Problem

`wai resource add skill <name> --yes` creates:

```markdown
---
name: issue-gather
description: ""
---

# Issue Gather

Instructions go here.
```

Writing 6 skills from scratch with consistent structure takes significant effort.
A new project adopter would need to know the $ARGUMENTS convention, the wai artifact
patterns, and the convergence check structure just to get started.

### Proposed Fix

```bash
wai resource add skill issue/gather --template=gather
wai resource add skill issue/create --template=create
wai resource add skill impl/run --template=tdd
wai resource add skill impl/review --template=rule-of-5
```

Available templates:
- `gather` — research-focused stub with `wai search`, codebase exploration, `wai add research`
- `create` — creation stub with `wai search` for artifact retrieval, item loop, dependency wiring
- `tdd` — test-first implementation stub with RED/GREEN/REFACTOR steps and `just check`
- `rule-of-5` — 5-pass review stub with convergence check and APPROVED/NEEDS_CHANGES/NEEDS_HUMAN verdict

Templates should be installable from external sources (see P7).

---

## P7 — Cross-project skill sharing

**Status**: Low — friction only when setting up a new project with the same patterns

### Problem

The 6 pipeline skills created today are generic (not bichos-specific) but are stored
inside this repo's `.wai/`. To use the same pattern in another project:
1. Manually copy 6 SKILL.md files
2. Recreate the `.claude/commands/` structure
3. Re-write the CLAUDE.md orchestration section

No mechanism for sharing reusable skills across projects or repositories.

### Proposed Fix

**Global skill library** at `~/.wai/resources/skills/`:

```bash
# Install a skill globally (available in all projects)
wai resource install issue/gather --global
wai resource install issue/gather --from-repo ./other-project

# Use from a registry (e.g., incitaciones)
wai resource install incitaciones/rule-of-5-review
wai resource install https://charly-vibes.github.io/incitaciones/manifest.json

# Project-local override still takes priority
```

**Export / import**:
```bash
wai resource export issue/gather impl/run --output pipeline-skills.tar.gz
wai resource import pipeline-skills.tar.gz
```

**Portability requirement**: skill SKILL.md files must not contain hardcoded project names
or paths. Use `$PROJECT`, `$REPO_ROOT`, and `$ARGUMENTS` placeholders instead.

---

## P8 — wai add tags on plan and design artifact types

**Status**: Low — minor inconsistency

### Problem

The `--tags` flag works only on `wai add research`:

```bash
wai add research "..." --tags=ant-forager,pipeline-stage-1   # works
wai add plan "..." --tags=ant-forager                        # flag not available
wai add design "..." --tags=ant-forager                      # flag not available
```

This makes the tagging strategy from P5-A incomplete.

### Proposed Fix

Add `--tags` (or `--tag`, accepting multiple uses) to all `wai add` subcommands.
Consistent with how search already supports `--type` filtering across all artifact types.

---

## Appendix: What Works Well

These patterns felt natural and should be preserved as-is:

- **Phase tracking** (`wai phase show / next`) — clean mental model, low friction
- **Artifact semantics** (research / plan / design / handoff) — the three-way split is
  meaningful and agents use it correctly without extra instruction
- **`wai prime`** — excellent session start orientation; surfacing the last handoff is exactly right
- **`wai search --type=<type>`** — sufficient for single-project use today
- **`wai resource list skills --json`** — machine-readable output enables tooling
- **`wai resource add skill`** creating the directory structure — correct foundation
- **PARA structure** (projects / areas / resources / archives) — maps well to how work evolves
- **`wai reflect`** synthesizing CLAUDE.md — closes the loop between artifacts and instructions
- **Separation of concerns**: wai (why) / beads (what) / openspec (what it should look like)
  is a genuinely useful three-way split. Don't collapse these.

---

## Implementation Priority

| Proposal | Impact | Effort | Priority |
|----------|--------|--------|----------|
| P2 — Fix wai sync symlinks | Blocking | Low | P0 |
| P1 — Hierarchical skill names | High | Low | P1 |
| P3 — Claude Code projection target | High | Medium | P1 |
| P8 — Tags on all artifact types | Medium | Low | P2 |
| P5 — Precise artifact retrieval | Medium | Low | P2 |
| P6 — Skill templates | Medium | Medium | P3 |
| P4 — Pipeline resource type | High | High | P3 |
| P7 — Cross-project skill sharing | Medium | High | P4 |

P0 and P1 unblock the core pattern. P2+P3 together eliminate dual-format maintenance.
P4 is the long-term target that makes pipelines first-class in wai.
