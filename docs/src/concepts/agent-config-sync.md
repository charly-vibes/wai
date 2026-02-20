# Agent Config Sync

Wai maintains a single source of truth for agent configurations and syncs them to tool-specific locations using configurable projection strategies.

## Directory Structure

Agent configs are stored in `.wai/resources/agent-config/`:

```
.wai/resources/agent-config/
├── .projections.yml          # Sync configuration
├── skills/                   # Agent skills
│   ├── code-review.md
│   └── commit.md
├── rules/                    # System rules
│   ├── security.md
│   └── style.md
└── context/                  # Context files
    └── project-context.md
```

## Projection Strategies

Configure syncs in `.wai/resources/agent-config/.projections.yml`:

### 1. Symlink Strategy

Creates target directory with symlinks (or copies on Windows) to source files.

**Note on Windows:** Windows doesn't support symlinks in all configurations (requires Developer Mode or admin privileges). Wai automatically falls back to file copies when symlinks aren't available.

**Use when:** Tools expect a directory structure with individual files.

```yaml
projections:
  - strategy: symlink
    sources:
      - skills/
    target: .claude/skills/
```

**Result:**
- `.claude/skills/code-review.md` → symlink to `.wai/resources/agent-config/skills/code-review.md`
- `.claude/skills/commit.md` → symlink to `.wai/resources/agent-config/skills/commit.md`

### 2. Inline Strategy

Concatenates multiple source files into a single target file.

**Use when:** Tools expect a single configuration file (e.g., `.cursorrules`).

```yaml
projections:
  - strategy: inline
    sources:
      - rules/base.md
      - rules/security.md
      - rules/style.md
    target: .cursorrules
```

**Result:**
```markdown
<!-- AUTO-GENERATED FILE - DO NOT EDIT DIRECTLY -->
<!-- Source: .wai/resources/agent-config/rules/base.md -->

[content of base.md]

<!-- Source: .wai/resources/agent-config/rules/security.md -->

[content of security.md]

<!-- Source: .wai/resources/agent-config/rules/style.md -->

[content of style.md]
```

### 3. Reference Strategy

Creates a markdown file listing paths to source files.

**Use when:** Tools can follow references to external files.

```yaml
projections:
  - strategy: reference
    sources:
      - context/
    target: .agents/context-refs.md
```

**Result:**
```markdown
# Agent Context References

This file references agent context files managed by wai.

## Files

- .wai/resources/agent-config/context/project-context.md
```

## Configuration Format

Full `.projections.yml` example:

```yaml
projections:
  # Symlink skills to Claude Code
  - strategy: symlink
    sources:
      - skills/
    target: .claude/skills/

  # Inline rules to Cursor
  - strategy: inline
    sources:
      - rules/base.md
      - rules/security.md
    target: .cursorrules

  # Reference context files
  - strategy: reference
    sources:
      - context/
    target: .gemini/context-refs.md
```

## Commands

### Check Sync Status

```bash
wai sync --status
```

Shows:
- Current projections
- Source files
- Target status (in sync / out of sync)
- Files that would be created/updated

### Apply Sync

```bash
wai sync
```

Applies all configured projections:
- Creates/updates symlinks
- Regenerates inline files
- Updates reference files

## Managing Configs

### Add New Config

```bash
wai config add skill my-skill.md
```

Copies `my-skill.md` to `.wai/resources/agent-config/skills/my-skill.md`.

### List Configs

```bash
wai config list
```

Shows all configs organized by type (skills, rules, context).

### Edit Config

```bash
wai config edit skills/my-skill.md
```

Opens config in `$EDITOR` (or `$VISUAL`, falls back to `vi`).

### Import Existing Configs

```bash
wai import .claude/
wai import .cursorrules
```

Imports existing tool configs into wai's single source of truth.

## Workflow

1. **Add/Edit configs** in `.wai/resources/agent-config/`
2. **Check status**: `wai sync --status`
3. **Apply sync**: `wai sync`
4. **Verify**: `wai doctor`

## Doctor Checks

The `wai doctor` command validates:
- Projection configuration syntax
- Source files exist
- Target directories are writable
- Symlinks are valid
- Inline files match sources
- Reference files are current

Auto-fix mode can repair common issues:
```bash
wai doctor --fix
```

## Conflict Resolution

**Important:** Wai always overwrites target files during sync. The source files in `.wai/resources/agent-config/` are the single source of truth.

### What Happens When You Edit Synced Files

If you manually edit a synced target file (e.g., `.cursorrules`, `.claude/skills/my-skill.md`):
1. Your changes will be **overwritten** the next time you run `wai sync`
2. No warning is given - wai assumes sources are authoritative

### Best Practices

**✅ DO:**
- Edit files in `.wai/resources/agent-config/`
- Use `wai config edit skills/my-skill.md`
- Run `wai sync --status` before syncing to preview changes
- Keep sources under version control

**❌ DON'T:**
- Edit target files directly (`.cursorrules`, `.claude/skills/`, etc.)
- Manually create files in target directories
- Expect wai to merge changes from targets back to sources

### Recovering Manual Edits

If you accidentally edited a target file:

```bash
# Copy your changes to a backup
cp .cursorrules my-changes-backup.md

# Check what the source contains
cat .wai/resources/agent-config/rules/*.md

# Manually merge your changes into the source files
wai config edit rules/base.md

# Re-sync to apply
wai sync
```

### Migration Workflow

When importing from existing tool configs:

```bash
# Import existing configs (one-time)
wai import .cursorrules
wai import .claude/

# From now on, edit only in .wai/resources/agent-config/
wai config edit rules/imported-rules.md

# Sync to propagate changes
wai sync
```

## Benefits

- **Single Source of Truth** — Edit once, sync everywhere
- **Version Control** — All configs tracked in `.wai/`
- **Consistency** — Same configs across all tools
- **Flexibility** — Different strategies for different tools
- **Auditability** — Clear projection configuration
- **No Conflicts** — Unidirectional sync prevents merge conflicts

## See Also

- [Commands Reference](../commands.md#agent-configuration) - Config management commands
- [Troubleshooting](../troubleshooting.md#sync-issues) - Sync troubleshooting guide
- [Quick Start](../quick-start.md#agent-configuration) - Getting started with config sync
