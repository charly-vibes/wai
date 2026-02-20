# Workflow Detection

Wai automatically detects project patterns and provides context-aware suggestions based on your current state.

## How It Works

When you run `wai status`, wai analyzes:
- Current project phase
- Number and type of artifacts
- Plugin state (git status, open issues, etc.)
- Recent activity

Based on this analysis, it detects workflow patterns and suggests relevant next steps.

## Detected Patterns

### NewProject

**Condition:** Project exists but has no artifacts yet, in research phase.

**Suggestions:**
- Add initial research with `wai add research "..."`
- Import existing research with `wai add research --file notes.md`
- Run tutorial for guidance: `wai tutorial`

**Example:**
```bash
$ wai status
Project: my-app
Phase: research
Artifacts: 0

Pattern: NewProject
Suggestions:
  - Start gathering research: wai add research "Initial findings"
  - Import existing notes: wai add research --file notes.md
```

### ResearchPhaseMinimal

**Condition:** In research phase with few (≤1) research artifacts.

**Suggestions:**
- Continue gathering information
- Add more research artifacts
- Consider when you have enough to move to design

**Example:**
```bash
$ wai status
Project: my-app
Phase: research
Research artifacts: 1

Pattern: ResearchPhaseMinimal
Suggestions:
  - Add more research: wai add research "Additional findings"
  - When ready, advance: wai phase next
```

### ReadyToImplement

**Condition:** In plan or design phase with design documents created.

**Suggestions:**
- Move to implement phase
- Review designs before starting
- Set up development environment

**Example:**
```bash
$ wai status
Project: my-app
Phase: design
Design artifacts: 2
Plan artifacts: 1

Pattern: ReadyToImplement
Suggestions:
  - Ready to implement: wai phase next
  - Review designs: wai search "design" --in my-app
  - Create handoff: wai handoff create my-app
```

### ImplementPhaseActive

**Condition:** Currently in implement phase.

**Suggestions:**
- Run tests regularly
- Document implementation decisions
- Consider moving to review when done

**Example:**
```bash
$ wai status
Project: my-app
Phase: implement

Pattern: ImplementPhaseActive
Suggestions:
  - Run tests: cargo test
  - Document decisions: wai add design "Implementation notes"
  - When done, advance: wai phase next
```

## Context-Aware Commands

Suggestions include specific commands tailored to your project:

```bash
$ wai status
Project: api-server
Phase: implement
Plugin: beads (5 open issues)
Plugin: git (uncommitted changes)

Suggestions:
  - Review open issues: wai beads ready
  - Check git status: git status
  - Run tests: cargo test
  - Commit changes: git add -A && git commit
```

## Plugin Integration

Workflow detection incorporates plugin context:

**With beads plugin:**
- Suggests reviewing open issues
- Recommends closing completed work
- Highlights blockers

**With git plugin:**
- Detects uncommitted changes
- Suggests creating commits
- Recommends pushing to remote

**With openspec plugin:**
- Shows active change proposals
- Displays implementation progress
- Suggests next specification steps

## JSON Output

Get structured workflow information:

```bash
wai status --json | jq '.suggestions'
```

**Output:**
```json
[
  {
    "pattern": "ReadyToImplement",
    "suggestion": "Move to implement phase",
    "command": "wai phase next"
  },
  {
    "pattern": "ReadyToImplement",
    "suggestion": "Review designs before implementing",
    "command": "wai search \"design\" --in my-app"
  }
]
```

## Custom Patterns

While wai detects common patterns automatically, you can create custom workflow patterns through plugins.

### Example Custom Pattern Plugin

```yaml
name: my-workflow
description: Custom workflow detector
hooks:
  - type: on_status
    command: |
      if [ -f .needs-review ]; then
        echo "Ready for review - run: make review"
      fi
    inject_as: custom_workflow
```

## Best Practices

### Follow Suggestions

Wai's suggestions are designed to guide you through proven workflows:
1. Research → gather information
2. Design → make decisions
3. Plan → break down work
4. Implement → build
5. Review → validate
6. Archive → document and close

### Ignore When Needed

Suggestions are guidance, not rules. Feel free to:
- Skip phases that don't apply
- Go back when you need more research
- Add artifacts in any phase
- Move forward at your own pace

### Use with Plugins

Workflow detection works best with plugins:
- **beads** — Track concrete tasks alongside high-level workflow
- **git** — Keep version control in sync with project phases
- **openspec** — Align specifications with implementation phases

## Disable Suggestions

To hide workflow suggestions:

```bash
wai status --quiet
```

Or suppress output:

```bash
wai status -q
```

## See Also

- [Project Phases](../concepts/phases.md) - Understanding workflow stages
- [Commands Reference](../commands.md#viewing--navigating) - Status command details
- [JSON Output](./json-output.md#status) - Programmatic access to suggestions
