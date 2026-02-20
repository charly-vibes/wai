# Frequently Asked Questions

## General

### What is wai?

Wai (pronounced "why") is a command-line workflow manager for AI-driven development. It helps you capture and organize the research, reasoning, and decisions behind your code - the "why" that's often lost between sessions.

### How is wai different from other documentation tools?

Most tools focus on *what* you built. Wai captures *why* you built it that way:
- Research findings and trade-offs
- Design decisions and alternatives considered
- Implementation plans and reasoning
- Session handoffs with context

### Do I need to use git with wai?

No, wai works independently. However, wai integrates nicely with git through the plugin system, including git status in handoffs and status output.

### Can I use wai with existing projects?

Yes! Run `wai init` in any directory to start using wai. You can import existing documentation with `wai import` or `wai add research --file`.

## Usage

### Which phase should I be in?

Phases are guidelines, not gates. Use what makes sense:
- **Research** - Exploring, learning, gathering information
- **Design** - Making architectural decisions
- **Plan** - Breaking down implementation steps
- **Implement** - Writing code
- **Review** - Testing, validating, refining
- **Archive** - Wrapping up, documenting outcomes

You can skip phases or go backward anytime.

### How do I choose between projects, areas, and resources?

**PARA method:**
- **Projects** - Active work with a goal and end date (e.g., "user-auth-feature")
- **Areas** - Ongoing responsibilities without end date (e.g., "performance-monitoring")
- **Resources** - Reference material (e.g., agent configs, templates)
- **Archives** - Completed or inactive items

### Should I use wai for every project?

Wai is most valuable for:
- Complex features requiring research and design
- Work spanning multiple sessions
- Collaboration requiring knowledge transfer
- Projects where "why" matters (architecture decisions, trade-offs)

For quick fixes or trivial changes, standard git commits may be enough.

### How often should I create handoffs?

Create handoffs when:
- Ending a work session (especially multi-hour sessions)
- Switching context to another project
- Handing work to another developer
- Taking a break and might forget context

Think of handoffs as "save points" for your reasoning.

## Agent Configuration

### What's the difference between skills, rules, and context?

- **Skills** - Task-specific instructions (e.g., "how to review code", "how to commit")
- **Rules** - System-wide constraints (e.g., "always add tests", "follow security practices")
- **Context** - Project-specific background (e.g., "architecture overview", "team conventions")

### Which sync strategy should I use?

| Strategy | Use When | Example |
|----------|----------|---------|
| Symlink | Tool expects directory of files | Claude Code, Aider |
| Inline | Tool expects single concatenated file | Cursor (.cursorrules) |
| Reference | Tool can follow paths | Custom scripts |

See [Agent Config Sync](./concepts/agent-config-sync.md) for details.

### Can I use multiple sync strategies?

Yes! You can have different projections for different tools:

```yaml
projections:
  - strategy: symlink
    sources: [skills/]
    target: .claude/skills/
  - strategy: inline
    sources: [rules/]
    target: .cursorrules
```

### What happens if I edit synced files directly?

Wai will **overwrite** them on the next sync. Always edit source files in `.wai/resources/agent-config/`, never the synced targets.

## Plugins

### How do I create a custom plugin?

Create a YAML file in `.wai/plugins/`:

```yaml
name: my-tool
description: My tool integration
detector:
  type: directory
  path: .mytool
commands:
  - name: status
    description: Show status
    command: mytool status
    read_only: true
hooks:
  - type: on_status
    command: mytool stats
    inject_as: mytool_stats
```

See [Plugin System](./concepts/plugins.md) for complete guide.

### Can I disable built-in plugins?

No, built-in plugins (git, beads, openspec) cannot be disabled. They're only active when their detector finds the relevant files (e.g., `.git/` for git).

### Why isn't my plugin detected?

Common reasons:
1. Detector path doesn't exist (e.g., `.mytool/` directory missing)
2. YAML syntax error in plugin definition
3. Plugin tool not installed or not in PATH

Run `wai doctor` to diagnose plugin issues.

## Workflow & Patterns

### What are workflow patterns?

Wai automatically detects your project state and suggests next steps:
- **NewProject** - No artifacts yet → suggests adding research
- **ResearchPhaseMinimal** - Few artifacts → suggests gathering more info
- **ReadyToImplement** - Designs complete → suggests moving to implementation
- **ImplementPhaseActive** - Currently implementing → suggests testing

### Can I customize workflow suggestions?

Not yet, but it's planned. For now, you can ignore suggestions that don't fit your workflow.

### How does wai know what to suggest?

Wai analyzes:
- Current project phase
- Number and type of artifacts
- Plugin state (git status, open issues, etc.)
- Recent activity

Based on this, it matches patterns and suggests relevant commands.

## Technical

### Where is data stored?

Everything is in `.wai/` directory:
```
.wai/
├── config.toml           # Project configuration
├── projects/             # Active projects with artifacts
├── resources/            # Agent configs, templates
└── plugins/              # Custom plugin definitions
```

Safe to version control with git!

### Is wai data portable?

Yes! The entire `.wai/` directory can be:
- Committed to git
- Copied to another machine
- Backed up
- Shared with team members

### Can I use wai in CI/CD?

Yes! Use non-interactive mode with JSON output:

```bash
wai status --json --no-input
wai search "query" --json --no-input
wai doctor --json --no-input
```

### Does wai work offline?

Yes, wai is completely local. No network required.

### What's the performance impact?

Minimal. Wai is a lightweight CLI tool. Most commands complete in milliseconds. Large projects (hundreds of artifacts) may see slower search/timeline commands, but typically still under 1 second.

## Troubleshooting

### Command not found after installation

Add cargo bin to PATH:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add to `~/.bashrc` or `~/.zshrc` to make permanent.

### Doctor reports issues I don't understand

Run with verbose mode:
```bash
wai doctor -v
```

Check [Troubleshooting](./troubleshooting.md) for common issues.

### My synced files keep getting overwritten

This is expected behavior. Wai syncs from source files in `.wai/resources/agent-config/` to target locations. Always edit source files, never the synced targets.

See [Agent Config Sync - Conflict Resolution](./concepts/agent-config-sync.md#conflict-resolution).

### Search returns no results but files exist

Check:
1. Are you searching the right project? Use `--in project-name`
2. Is the search term exact? Try regex mode: `--regex`
3. Do artifacts exist? Check: `wai timeline project-name`

## Getting More Help

- **Documentation:** Read full docs in `docs/` directory
- **Tutorial:** Run `wai tutorial` for interactive guide
- **Command help:** `wai <command> --help`
- **Workspace health:** `wai doctor`
- **GitHub Issues:** https://github.com/charly-vibes/wai/issues

## See Also

- [Quick Start](./quick-start.md) - Get started in 5 minutes
- [Commands Reference](./commands.md) - Complete command list
- [Troubleshooting](./troubleshooting.md) - Common problems and solutions
- [Concepts](./concepts/para-method.md) - Deep dive into wai's design
