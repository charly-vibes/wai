# How to Adopt Wai in an Existing Repository

This guide walks through adding wai to a repository that already has code, history, and an established workflow.

> **Prerequisite**: Install wai first — see the [Installation](../installation.md) page.

## 1. Initialize the Workspace

From your repository root:

```bash
wai init
```

This creates a `.wai/` directory. Commit it — wai artifacts are designed to live alongside your code.

## 2. Create a Project for Your Current Work

If you're actively working on something, create a project for it:

```bash
wai new project my-current-feature
```

If you have multiple workstreams, create one project per feature or initiative.

## 3. Capture What You Already Know

Don't start from scratch — import the reasoning that already exists:

```bash
# Record decisions you've already made
wai add research "Chose PostgreSQL over SQLite for concurrent write support"
wai add design "REST API with versioned endpoints (/v1/, /v2/)"

# Import existing notes or docs
wai add research --file docs/architecture-notes.md
```

Set the phase to match where you actually are:

```bash
wai phase set implement    # if you're already writing code
```

## 4. Set Up Agent Config Sync (Optional)

If you use AI assistants (Claude Code, Cursor, etc.), consolidate your configs:

```bash
# Import existing configs into wai's single source of truth
wai import .cursorrules
wai import .claude/

# Check what was imported
wai config list

# Sync back to tool locations
wai sync
```

## 5. Start Using the Session Loop

From now on, use `wai prime` at the start of each session and `wai close` at the end:

```bash
# Beginning of session
wai prime                  # Shows status, phase, suggestions

# ... do your work, capture decisions as you go ...
wai add research "Discovered edge case with auth tokens"

# End of session
wai close                  # Creates handoff for next session
```

## 6. Check Your Setup

Verify everything is healthy:

```bash
wai doctor                 # Workspace health check
wai way                    # Repository best practices
```

## What Not to Do

- **Don't try to backfill all history.** Capture decisions going forward. The value compounds over time.
- **Don't skip phases.** Even if you set the phase to `implement`, record the reasoning that got you there.
- **Don't forget to commit `.wai/`.** It's part of your repository.
