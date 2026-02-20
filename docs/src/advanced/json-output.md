# JSON Output

All major wai commands support `--json` flag for machine-readable output, enabling integration with scripts, automation tools, and other software.

## Usage

```bash
wai <command> --json
```

JSON output is written to stdout, errors to stderr. This allows you to separate data from errors:

```bash
# Capture JSON output only
wai status --json > status.json

# Capture errors only
wai status --json 2> errors.log

# Capture both separately
wai status --json > status.json 2> errors.log

# Pipe JSON directly to processing tools
wai status --json | jq '.projects[].name'
```

## Supported Commands

### Status

```bash
wai status --json
```

**Output:**
```json
{
  "project_root": "/path/to/project",
  "projects": [
    {
      "name": "my-app",
      "phase": "implement",
      "path": ".wai/projects/my-app"
    }
  ],
  "plugins": [
    {
      "name": "beads",
      "detected": true,
      "description": "Issue tracking"
    },
    {
      "name": "git",
      "detected": true,
      "description": "Version control"
    }
  ],
  "hook_outputs": {
    "git_status": "M src/main.rs\n",
    "beads_stats": "Open: 5, Closed: 12"
  },
  "openspec": {
    "specs": ["cli-core", "plugin-system"],
    "changes": [
      {
        "name": "add-feature-x",
        "progress": 75
      }
    ]
  },
  "suggestions": [
    {
      "pattern": "ImplementPhaseActive",
      "suggestion": "Consider running tests",
      "command": "cargo test"
    }
  ]
}
```

### Search

```bash
wai search "authentication" --json
```

**Output:**
```json
{
  "query": "authentication",
  "results": [
    {
      "path": ".wai/projects/my-app/research/2026-02-15-auth-research.md",
      "type": "research",
      "project": "my-app",
      "line": 12,
      "context": [
        "Evaluated JWT vs session-based authentication.",
        "Chose JWT for stateless API design.",
        "Will store tokens in httpOnly cookies."
      ]
    }
  ]
}
```

### Timeline

```bash
wai timeline my-app --json
```

**Output:**
```json
{
  "project": "my-app",
  "entries": [
    {
      "date": "2026-02-15",
      "type": "research",
      "title": "auth-research",
      "path": ".wai/projects/my-app/research/2026-02-15-auth-research.md"
    },
    {
      "date": "2026-02-16",
      "type": "design",
      "title": "api-design",
      "path": ".wai/projects/my-app/designs/2026-02-16-api-design.md"
    }
  ]
}
```

### Plugin List

```bash
wai plugin list --json
```

**Output:**
```json
{
  "plugins": [
    {
      "name": "beads",
      "description": "Issue tracking",
      "detected": true,
      "commands": [
        {
          "name": "list",
          "description": "List issues",
          "read_only": true
        },
        {
          "name": "show",
          "description": "Show issue details",
          "read_only": true
        }
      ],
      "hooks": [
        {
          "type": "on_status",
          "command": "bd stats"
        },
        {
          "type": "on_handoff_generate",
          "command": "bd list --status=open"
        }
      ]
    }
  ]
}
```

### Resource List

```bash
wai resource list skills --json
```

**Output:**
```json
{
  "skills": [
    {
      "name": "code-review",
      "path": ".wai/resources/agent-config/skills/code-review.md"
    },
    {
      "name": "commit",
      "path": ".wai/resources/agent-config/skills/commit.md"
    }
  ]
}
```

## Error Handling

Errors are returned as JSON when `--json` is used:

```json
{
  "code": "wai::error::project_not_found",
  "message": "Project 'missing-app' not found",
  "help": "Use 'wai show' to list available projects",
  "details": null
}
```

### JSON Schema Stability

**Current Policy:** JSON output schemas are considered **stable** within a major version.

- ✅ **Safe:** Adding new optional fields
- ✅ **Safe:** Adding new values to enums
- ❌ **Breaking:** Removing fields
- ❌ **Breaking:** Changing field types
- ❌ **Breaking:** Renaming fields

**Recommendation:** When writing automation scripts, handle unknown fields gracefully and validate expected fields exist.

**Example resilient parsing:**
```bash
# Good - checks for field existence
if jq -e '.projects[]? | select(.name == "my-app")' status.json; then
  echo "Project exists"
fi

# Good - provides defaults for missing fields
jq '.projects[]? | {name, phase: (.phase // "unknown")}' status.json
```

## Processing with jq

Use `jq` for powerful JSON processing:

```bash
# Extract project names
wai status --json | jq '.projects[].name'

# Get all research artifacts
wai search ".*" --regex --type research --json | jq '.results[].path'

# Count plugins detected
wai plugin list --json | jq '[.plugins[] | select(.detected)] | length'

# Get recent timeline entries
wai timeline my-app --json | jq '.entries | .[-5:]'

# Check if sync is needed
wai sync --status --json | jq '.projections[] | select(.status != "in_sync")'
```

## Automation Examples

### CI/CD Integration

```bash
#!/bin/bash
# Check if handoff exists for current branch
BRANCH=$(git branch --show-current)
HANDOFFS=$(wai search "$BRANCH" --type handoff --json | jq -r '.results[].path')

if [ -z "$HANDOFFS" ]; then
  echo "No handoff found for branch $BRANCH"
  exit 1
fi
```

### Slack Notifications

```bash
#!/bin/bash
# Send status to Slack
STATUS=$(wai status --json)
PROJECTS=$(echo "$STATUS" | jq -r '.projects[] | "\(.name): \(.phase)"' | tr '\n' ', ')

curl -X POST "$SLACK_WEBHOOK" \
  -H 'Content-Type: application/json' \
  -d "{\"text\":\"Project Status: $PROJECTS\"}"
```

### Dashboard Generation

```python
import json
import subprocess

# Get status
result = subprocess.run(['wai', 'status', '--json'], capture_output=True, text=True)
status = json.loads(result.stdout)

# Generate HTML dashboard
html = "<h1>Project Status</h1>"
for project in status['projects']:
    html += f"<div><strong>{project['name']}</strong>: {project['phase']}</div>"

print(html)
```

## Non-Interactive Mode

Combine `--json` with `--no-input` for fully automated scripts:

```bash
wai status --json --no-input
wai search "query" --json --no-input
```

This prevents any interactive prompts that might block automation.

## Safe Mode

Use `--safe` with `--json` for read-only automation:

```bash
wai status --json --safe
```

Ensures no modifications occur during automated queries.

## See Also

- [Commands Reference](../commands.md#global-flags) - Global flags documentation
- [Troubleshooting](../troubleshooting.md#json-output-issues) - JSON troubleshooting
- [Quick Start](../quick-start.md#json-output) - Getting started with JSON
