# Troubleshooting

Common issues and solutions for wai.

## Commands Not Working

### "wai: command not found"

**Problem:** Wai is not in your PATH after installation.

**Solution:**
```bash
# Check if cargo bin is in PATH
echo $PATH | grep cargo

# If not, add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.bashrc  # or ~/.zshrc
```

### "Project not found" or "Not initialized"

**Problem:** Running wai commands outside of an initialized directory.

**Solution:**
```bash
# Check if you're in a wai workspace
ls -la .wai/

# If not, initialize or navigate to the correct directory
wai init
# or
cd /path/to/your/wai/workspace
```

**Error example:**
```
Error: wai::error::not_initialized
  ╭─[command]
  │ wai status
  ╰────
  help: Run 'wai init' to initialize wai in this directory
```

## Sync Issues

### Sync conflicts - target file manually edited

**Problem:** You edited a synced file (e.g., `.cursorrules`) directly and wai overwrote it.

**Solution:**
```bash
# Always edit source files, not targets
wai config edit rules/my-rule.md

# If you accidentally edited the target, copy changes to source first
cp .cursorrules .wai/resources/agent-config/rules/backup.md
# Then edit the source and re-sync
wai sync
```

**Prevention:** Never edit synced target files. Edit sources in `.wai/resources/agent-config/` instead.

### Symlinks not working on Windows

**Problem:** Symlink strategy fails on Windows.

**Solution:**
Wai automatically falls back to file copies on Windows. If you see errors:

```bash
# Check sync status
wai sync --status

# Run doctor to diagnose
wai doctor

# Try auto-fix
wai doctor --fix
```

Alternatively, switch to inline or reference strategy in `.projections.yml`.

### "Permission denied" during sync

**Problem:** Wai can't write to target directory.

**Solution:**
```bash
# Check target directory permissions
ls -ld .claude/

# Fix permissions
chmod u+w .claude/

# Or run sync with appropriate permissions
sudo wai sync  # Last resort, not recommended
```

## Plugin Issues

### Plugin not detected

**Problem:** Plugin shows as "not detected" in `wai plugin list`.

**Solution:**
```bash
# Check if detector file/directory exists
ls -la .beads/    # for beads
ls -la .git/      # for git
ls -la openspec/  # for openspec

# Install the plugin tool
cargo install beads  # or appropriate installation

# Verify detection
wai plugin list
```

### Custom plugin not loading

**Problem:** YAML plugin in `.wai/plugins/` not appearing.

**Solution:**
```bash
# Check YAML syntax
cat .wai/plugins/my-plugin.yml

# Verify file extension (.yml or .yaml)
ls .wai/plugins/

# Check for YAML errors
wai doctor

# Example valid plugin:
cat > .wai/plugins/example.yml << 'EOF'
name: example
description: Example plugin
detector:
  type: directory
  path: .example
commands:
  - name: status
    description: Show status
    command: example status
    read_only: true
EOF
```

### Plugin command fails

**Problem:** `wai <plugin> <command>` returns an error.

**Solution:**
```bash
# Verify plugin tool is installed and in PATH
which beads
which mytool

# Test command directly
beads list
mytool status

# Check plugin definition
wai plugin list --json | jq '.plugins[] | select(.name=="myplugin")'
```

## Search & Timeline Issues

### Search returns no results

**Problem:** Can't find artifacts you know exist.

**Solution:**
```bash
# Check if you're searching the right project
wai search "query" --in project-name

# Try without filters
wai search "query"

# Use case-insensitive search (regex mode)
wai search "(?i)query" --regex

# Verify artifacts exist
wai timeline my-project
wai show my-project
```

### Timeline shows wrong dates

**Problem:** Artifacts appear in unexpected order.

**Solution:**
Timeline is sorted by filename date (YYYY-MM-DD prefix). Artifacts are created with the current date by default.

```bash
# Check actual filenames
ls .wai/projects/my-project/research/

# Files are named: YYYY-MM-DD-slug.md
# Timeline sorts by this date, not file modification time
```

## Doctor Command Issues

### Doctor reports issues but --fix doesn't work

**Problem:** `wai doctor --fix` doesn't repair all issues.

**Solution:**
Some issues require manual intervention:

```bash
# Run doctor to see all issues
wai doctor

# Check which issues can't be auto-fixed
# Doctor will show suggestions for manual fixes

# Common manual fixes:
# 1. Corrupted config - restore from backup or re-init
# 2. Missing directories - create manually or re-init
# 3. Permission issues - fix with chmod/chown
```

### Doctor false positives

**Problem:** Doctor reports issues that aren't actually problems.

**Solution:**
```bash
# Check specific validation
wai doctor --json | jq .

# Some "warnings" are informational
# Only "error" severity items need fixing

# If you believe it's a false positive, file an issue:
# https://github.com/charly-vibes/wai/issues
```

## Phase Management Issues

### Can't advance to next phase

**Problem:** `wai phase next` doesn't work or gives unexpected results.

**Solution:**
```bash
# Check current phase
wai phase show

# Phases are: research → design → plan → implement → review → archive
# Can't advance past archive

# If stuck, set phase directly
wai phase set implement

# Check phase history
wai phase show  # Shows all transitions
```

### Phase history looks wrong

**Problem:** Phase history shows unexpected transitions.

**Solution:**
Phase history is stored in `.wai/projects/<name>/.state`. This file tracks all transitions with timestamps.

```bash
# View state file
cat .wai/projects/my-project/.state

# If corrupted, you can manually edit (advanced)
# Or reset to a phase:
wai phase set research  # Starts fresh from research
```

## Artifact Issues

### Tags not working

**Problem:** Tags don't appear or can't search by tags.

**Solution:**
Tags are stored in YAML frontmatter for research artifacts:

```bash
# Add tags correctly (comma-separated, no spaces around commas)
wai add research "Finding" --tags "api,security,auth"

# NOT: --tags "api, security, auth"  # Extra spaces cause issues

# Check frontmatter
head -10 .wai/projects/my-project/research/*.md

# Should see:
# ---
# tags: [api, security, auth]
# ---
```

### Can't import file

**Problem:** `wai add research --file path.md` fails.

**Solution:**
```bash
# Check file exists and is readable
ls -la path.md
cat path.md

# Use absolute or relative path
wai add research --file ./notes/research.md
wai add research --file /full/path/to/research.md

# Check file isn't binary
file notes/research.md  # Should say "text"
```

## JSON Output Issues

### JSON output malformed

**Problem:** `--json` flag produces invalid JSON.

**Solution:**
```bash
# Ensure no extra output
wai status --json --quiet

# Check for errors on stderr
wai status --json 2>&1 | jq .

# Validate JSON
wai status --json | jq empty
```

### Can't parse JSON in scripts

**Problem:** Automation scripts fail to parse wai JSON output.

**Solution:**
```bash
# Capture only stdout
wai status --json 2>/dev/null | jq .

# Handle errors separately
if ! output=$(wai status --json 2>&1); then
  echo "Error: $output" >&2
  exit 1
fi
echo "$output" | jq .
```

## Performance Issues

### Commands are slow

**Problem:** Wai commands take a long time to execute.

**Solution:**
```bash
# Check project size
du -sh .wai/

# Large artifact counts can slow search/timeline
find .wai/projects -name "*.md" | wc -l

# Use filters to limit scope
wai search "query" --in specific-project -n 10
wai timeline project --from 2026-02-01

# For very large projects, consider archiving old work
wai move old-project archives
```

## Getting Help

### Where to find more help

**Resources:**
- **Built-in help:** `wai <command> --help`
- **Verbose help:** `wai --help -v` (shows advanced options)
- **Status check:** `wai doctor` (diagnoses workspace issues)
- **GitHub Issues:** https://github.com/charly-vibes/wai/issues
- **Documentation:** Check `docs/` directory

### Filing a bug report

Include:
1. Wai version: `wai --version`
2. Operating system: `uname -a`
3. Command that failed: Full command with flags
4. Error message: Complete error output
5. Doctor output: `wai doctor`

```bash
# Create comprehensive debug report
{
  echo "=== Version ==="
  wai --version

  echo -e "\n=== System ==="
  uname -a

  echo -e "\n=== Doctor ==="
  wai doctor

  echo -e "\n=== Config ==="
  cat .wai/config.toml
} > wai-debug-report.txt
```

Then attach `wai-debug-report.txt` to your issue.
