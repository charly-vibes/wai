# Quick Start

## Interactive Tutorial

Run the interactive tutorial for a guided introduction:

```bash
wai tutorial
```

The tutorial covers:
- What is wai and why use it
- PARA method organization
- Project phase workflow
- Core commands
- Session handoffs

## Initialize

Set up wai in your project directory:

```bash
wai init
```

Or specify a custom project name:

```bash
wai init --name my-project
```

## Create a Project

```bash
wai new project my-app
```

Projects are active work items with phases, timelines, and dated artifacts.

## Check Status

Get an overview of your project with contextual suggestions:

```bash
wai status
```

Status shows:
- All projects with current phases
- Detected plugins and their status
- OpenSpec changes if applicable
- Context-aware suggestions for next steps

## Manage Phases

Projects progress through phases — research, design, plan, implement, review, archive:

```bash
wai phase              # Show current phase with history
wai phase next         # Advance to next phase
wai phase back         # Go back to previous phase
wai phase set design   # Jump to specific phase
```

## Add Artifacts

Capture research, plans, and designs as you work:

```bash
# Add inline content
wai add research "Initial API analysis"
wai add plan "Implementation approach"
wai add design "Component architecture"

# Import from files
wai add research --file notes.md

# Add tags for organization
wai add research "Security findings" --tags "security,api,auth"

# Target specific project
wai add research "Findings" --project other-project
```

## Search

Find content across all artifacts:

```bash
# Basic search
wai search "authentication"

# Search with filters
wai search --type research           # Only research artifacts
wai search --in my-app                # Specific project
wai search --regex "api.*error"       # Regex patterns
wai search "config" -n 5              # Limit results
```

## View Timeline

See a chronological view of project activity:

```bash
# Full timeline
wai timeline my-app

# Date range
wai timeline my-app --from 2026-02-01 --to 2026-02-15

# Reverse order (oldest first)
wai timeline my-app --reverse
```

## Generate Handoffs

Create a handoff document summarizing project state:

```bash
wai handoff create my-app
```

Handoffs include:
- Current project phase
- Recent artifacts
- Plugin context (git status, open issues, etc.)

Perfect for session transitions or knowledge transfer.

## Agent Configuration

### Sync Configs

Push agent config files to their tool-specific locations:

```bash
# Check sync status
wai sync --status

# Apply sync
wai sync
```

### Manage Configs

```bash
# List all configs
wai config list

# Add new config
wai config add skill my-skill.md

# Edit config
wai config edit skills/my-skill.md
```

### Import Existing Configs

```bash
wai import .claude/
wai import .cursorrules
```

## Workspace Health

Check for issues and auto-fix when possible:

```bash
# Diagnose wai workspace issues
wai doctor

# Auto-repair wai workspace
wai doctor --fix

# Check repository best practices
wai way

# Get JSON output for CI integration
wai way --json
```

**Doctor checks (wai-specific):**
- Directory structure
- Config file integrity
- Plugin availability
- Sync status
- Project state consistency

**Way checks (general repository - 10 checks):**
- Task runner (justfile, Makefile)
- Git hooks (prek, pre-commit)
- Editor config (.editorconfig)
- Documentation (README, LICENSE, CONTRIBUTING, .gitignore)
- AI instructions (CLAUDE.md, AGENTS.md)
- LLM documentation (llm.txt)
- Agent skills (.wai/resources/skills/)
- GitHub CLI (gh installed & authenticated)
- CI/CD configuration
- Dev container setup

## Work with Plugins

List and interact with detected plugins:

```bash
# List all plugins
wai plugin list

# Use plugin commands (pass-through)
wai beads list
wai beads ready
wai beads show beads-123
```

## JSON Output

Get machine-readable output for automation:

```bash
wai status --json
wai search "config" --json
wai plugin list --json
wai timeline my-app --json
```

## Safe Mode

Run commands in read-only mode:

```bash
wai status --safe
wai search "query" --safe
```

Safe mode prevents any file modifications.

## Next Steps

### Learn Core Concepts
- [PARA Method](./concepts/para-method.md) - Understand project organization
- [Project Phases](./concepts/phases.md) - Master the workflow
- [Plugin System](./concepts/plugins.md) - Extend wai's capabilities
- [Agent Config Sync](./concepts/agent-config-sync.md) - Manage AI tool configs

### Reference Documentation
- [Commands Reference](./commands.md) - Complete command list
- [Troubleshooting](./troubleshooting.md) - Common problems and solutions
- [FAQ](./faq.md) - Frequently asked questions

### Advanced Usage
- [JSON Output](./advanced/json-output.md) - Automation and scripting
- [Workflow Detection](./advanced/workflow-detection.md) - Context-aware suggestions

### Getting Help
- Run `wai tutorial` for interactive guide
- Run `wai doctor` to check workspace health
- Check [GitHub Issues](https://github.com/charly-vibes/wai/issues) for known problems
