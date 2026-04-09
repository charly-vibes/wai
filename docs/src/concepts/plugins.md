# Plugin System

> **Why plugins?** Every team uses different tools — issue trackers, spec managers, version control. Building all of these into wai would make it opinionated and brittle. Instead, wai auto-detects what's already in your workspace and integrates through a plugin architecture. You get seamless context (git status in handoffs, open issues in `wai status`) without wai prescribing your toolchain.

Wai auto-detects and integrates with external tools through a flexible plugin architecture.

## Built-in Plugins

### Beads
- **Detection**: `.beads/` directory
- **Description**: Issue tracking with tasks, bugs, and dependencies
- **Commands**: `list`, `show`, `ready` (read-only)
- **Hooks**:
  - `on_handoff_generate` — Includes open issues in handoffs
  - `on_status` — Shows issue statistics

Example:
```bash
wai beads list        # Pass-through to beads plugin
wai beads ready       # Show issues ready to work on
```

### Git
- **Detection**: `.git/` directory
- **Description**: Version control integration
- **Hooks**:
  - `on_handoff_generate` — Includes git status
  - `on_status` — Shows recent commits

### OpenSpec
- **Detection**: `openspec/` directory
- **Description**: Specification and change management
- **Integration**: Status display shows active specs and change proposals with progress

## Plugin Commands

Pass-through commands allow direct access to plugin functionality:

```bash
wai <plugin> <command> [args...]
```

Example:
```bash
wai beads list --status=open
wai beads show beads-123
```

## Plugin Hooks

Plugins can inject context into wai workflows through hooks:

### Available Hooks

| Hook | When Triggered | Purpose |
|------|---------------|---------|
| `on_status` | `wai status` called | Add plugin context to status output |
| `on_handoff_generate` | `wai handoff create` called | Include plugin state in handoffs |
| `on_phase_transition` | Phase changes | React to project phase changes |

## Custom Plugins

Wai scans the `.wai/plugins/` directory at your workspace root for any files with a `.toml` extension.

### Example Plugin Definition

```toml
name = "my-tool"
description = "Custom tool integration"

[detector]
type = "directory"
path = ".mytool"   # Relative to workspace root

[[commands]]
name = "list"
description = "List items"
passthrough = "mytool list"
read_only = true

[[commands]]
name = "sync"
description = "Sync data"
passthrough = "mytool sync"
read_only = false

[hooks.on_status]
command = "mytool stats"
inject_as = "mytool_stats"

[hooks.on_handoff_generate]
command = "mytool status --format=summary"
inject_as = "mytool_context"
```

### Detector Types

- **directory** — Detect by directory presence. The `path` attribute is relative to the workspace root.
- **file** — Detect by file presence. The `path` attribute is relative to the workspace root.
- **command** — Detect by command availability. The `path` (or `command`) attribute is the shell command to execute (must return exit code 0).

> **Note:** For the `command` detector in TOML, use the `path` field to specify the command to run (e.g. `path = "mytool --version"`).

### Command Attributes

- **name** — Command name (e.g., `list`)
- **description** — Human-readable description
- **command** — Shell command to execute
- **read_only** — Whether command modifies state (respects `--safe` mode)

## Managing Plugins

### List All Plugins

```bash
wai plugin list
```

Shows:
- Plugin name and description
- Detection status (detected/not found)
- Available commands
- Configured hooks

### Enable/Disable Plugins

```bash
wai plugin enable my-tool
wai plugin disable my-tool
```

Note: Built-in plugins cannot be disabled.

## JSON Output

Get structured plugin information:

```bash
wai plugin list --json
```

Returns:
```json
{
  "plugins": [
    {
      "name": "beads",
      "description": "Issue tracking",
      "detected": true,
      "commands": ["list", "show", "ready"],
      "hooks": ["on_status", "on_handoff_generate"]
    }
  ]
}
```

## Safe Mode

Plugin commands that modify state are blocked in safe mode:

```bash
wai beads list --safe         # OK - read-only
wai my-tool sync --safe       # Blocked if not read_only
```

## Plugin Troubleshooting

### Plugin Not Detected

**Symptom:** `wai plugin list` shows plugin as "not detected"

**Common Causes:**
1. **Detector file/directory missing**
   ```bash
   # Check if detector path exists
   ls -la .beads/     # for beads plugin
   ls -la .git/       # for git plugin
   ls -la openspec/   # for openspec plugin
   ```

2. **Plugin tool not installed**
   ```bash
   # Verify tool is available
   which beads
   which mytool

   # Install if missing
   cargo install beads
   ```

3. **Custom plugin TOML syntax error**
   ```bash
   # Validate TOML
   cat .wai/plugins/my-plugin.toml

   # Check for common issues:
   # - Missing required fields (name, description, detector)
   # - Wrong detector type (must be: directory, file, or command)
   ```

### Plugin Command Fails

**Symptom:** `wai <plugin> <command>` returns error

**Debugging Steps:**
```bash
# 1. Verify plugin is detected
wai plugin list | grep myplugin

# 2. Test command directly
mytool command args

# 3. Check command definition
wai plugin list --json | jq '.plugins[] | select(.name=="myplugin") | .commands'

# 4. Verify passthrough is in PATH
which mytool

# 5. Check for command output issues
mytool command 2>&1 | head
```

### Custom Plugin Not Loading

**Symptom:** Custom plugin in `.wai/plugins/` doesn't appear

**Checklist:**
- ✅ File has `.toml` extension
- ✅ File is in `.wai/plugins/` directory
- ✅ TOML syntax is valid
- ✅ Required fields present: `name`, `description`, `detector`
- ✅ Detector path exists (for directory/file detectors)

**Example Valid Plugin:**
```toml
name = "example-tool"
description = "Example tool integration"

[detector]
type = "directory"
path = ".example"

[[commands]]
name = "status"
description = "Show status"
passthrough = "example status"
read_only = true

[[commands]]
name = "sync"
description = "Sync data"
passthrough = "example sync"
read_only = false

[hooks.on_status]
command = "example stats"
inject_as = "example_stats"
```

### Hook Output Not Showing

**Symptom:** Plugin hooks defined but output doesn't appear in status/handoffs

**Debugging:**
```bash
# 1. Check if plugin is detected
wai plugin list | grep myplugin

# 2. Verify hook command runs successfully
example stats  # Run hook command directly

# 3. Check hook definition
wai plugin list --json | jq '.plugins[] | select(.name=="example") | .hooks'

# 4. Look for hook output in status
wai status -v  # Verbose mode shows more detail

# 5. Check JSON output for hook_outputs
wai status --json | jq '.hook_outputs'
```

### Permission Errors

**Symptom:** "Permission denied" when running plugin commands

**Solutions:**
```bash
# 1. Check plugin tool permissions
ls -l $(which mytool)

# 2. Verify tool is executable
chmod +x $(which mytool)

# 3. For safe mode issues, check read_only flag
wai plugin list --json | jq '.plugins[] | select(.name=="myplugin") | .commands[] | {name, read_only}'
```

### Detector Not Working

**Symptom:** Plugin should be detected but shows as "not detected"

**Detector Types and Requirements:**

**Directory Detector:**
```toml
[detector]
type = "directory"
path = ".mytool"  # Directory must exist at workspace root
```
Check: `ls -la .mytool/`

**File Detector:**
```toml
[detector]
type = "file"
path = "mytool.config"  # File must exist at workspace root
```
Check: `ls -la mytool.config`

**Command Detector:**
```toml
[detector]
type = "command"
path = "mytool --version"  # Command must exit 0 and be in PATH
```
Check: `which mytool && mytool --version`

### Getting Help

If you can't resolve plugin issues:

```bash
# Create debug report
{
  echo "=== Plugin List ==="
  wai plugin list

  echo -e "\n=== Plugin JSON ==="
  wai plugin list --json

  echo -e "\n=== Custom Plugins ==="
  ls -la .wai/plugins/

  echo -e "\n=== Doctor Check ==="
  wai doctor
} > plugin-debug.txt
```

File an issue at https://github.com/charly-vibes/wai/issues with `plugin-debug.txt` attached.

## See Also

- [Commands Reference](../commands.md#plugin-management) - Plugin management commands
- [Troubleshooting](../troubleshooting.md#plugin-issues) - Plugin-specific troubleshooting
- [JSON Output](../advanced/json-output.md#plugin-list) - Plugin JSON schema
