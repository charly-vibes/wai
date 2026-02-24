# CLI Core

## Purpose

Define the core command structure and patterns for the wai CLI, including the verb-noun command hierarchy, global flags, and foundational commands for managing PARA-organized artifacts, project phases, agent config sync, handoffs, and cross-artifact search.

See also: onboarding spec for first-run and no-args welcome behavior.

## Problem Statement

For `wai` to effectively support development and research projects, it requires a **stable and predictable foundation**. Without a clearly defined and consistent core command structure and project organization, users would face a steep learning curve, inconsistent interactions, and an unstable platform for automation. This lack of a standardized and predictable interface would hinder adoption and make it difficult to build reliable workflows around `wai`.

## Design Rationale

The design of the CLI core follows a few key principles to establish a **stable foundation** that is intuitive, consistent, and extensible, making it a reliable platform for future growth.

### Command Structure: Verb-Noun

The chosen `verb-noun` pattern (e.g., `wai new project`) is a foundational **Type 1 decision** for `wai`'s grammar. It was selected for its readability and similarity to natural language, establishing a **predictable and consistent rhythm** for the user. This stable grammar makes commands easy to discover and remember, and crucially, **enables future extensibility** by providing a clear framework for applying existing verbs to new nouns. An alternative `noun-verb` pattern (e.g., `wai project new`) was considered but deemed less intuitive for `wai`'s action-oriented approach.

### Core Verbs

The primary verbs (`new`, `add`, `show`, `move`) provide a minimal, orthogonal set of operations. Additional top-level commands (`phase`, `sync`, `config`, `handoff`, `search`, `timeline`, `doctor`) provide direct access to frequently-used workflows that don't fit the verb-noun pattern naturally.

### PARA-Based Organization

Wai organizes artifacts using the PARA method (Projects, Areas, Resources, Archives). This replaces the previous bead-centric model with a proven organizational framework. Beads (`.beads/`) is an external tool that wai detects via its plugin system but does not manage directly.

## Scope and Requirements

This spec covers the foundational elements of the CLI.

### Non-Goals

- The detailed implementation of every command's functionality.
- The internal plugin execution model (covered in plugin-system spec).
- Specific output formats like JSON or YAML, beyond the standard text output.
- A graphical user interface.

### Requirement: Command Structure

The CLI SHALL use consistent verb-noun command patterns with primary verbs: `new`, `add`, `show`, `move`, plus dedicated top-level commands for `phase`, `sync`, `config`, `handoff`, `search`, `timeline`, `why`, `reflect`, and `doctor`.

#### Scenario: Create new items

- **WHEN** user runs `wai new project <name>`, `wai new area <name>`, or `wai new resource <name>`
- **THEN** the system creates the requested PARA item with appropriate directory structure

#### Scenario: Add artifacts to a project or area

- **WHEN** user runs `wai add research <content>`, `wai add plan <content>`, or `wai add design <content>`
- **THEN** the system creates a date-prefixed artifact file in the appropriate directory

#### Scenario: Show information

- **WHEN** user runs `wai show <item>`
- **THEN** the system displays the requested information

#### Scenario: Move items between PARA categories

- **WHEN** user runs `wai move <item> archives`
- **THEN** the system moves the item to the archives category

#### Scenario: Manage project phases

- **WHEN** user runs `wai phase`, `wai phase next`, `wai phase set <phase>`, or `wai phase back`
- **THEN** the system shows or transitions the current project's phase

#### Scenario: Sync agent configurations

- **WHEN** user runs `wai sync`
- **THEN** the system projects agent configs to tool-specific locations

#### Scenario: Manage agent configs

- **WHEN** user runs `wai config add skill <file>`, `wai config list`, or `wai config edit <path>`
- **THEN** the system manages agent configuration files

#### Scenario: Generate handoffs

- **WHEN** user runs `wai handoff create <project>`
- **THEN** the system generates a handoff document enriched with plugin data

#### Scenario: Search artifacts

- **WHEN** user runs `wai search <query>`
- **THEN** the system searches across all `.wai/` artifacts

#### Scenario: View timeline

- **WHEN** user runs `wai timeline <project>`
- **THEN** the system displays a chronological view of the project's artifacts

#### Scenario: Ask reasoning questions

- **WHEN** user runs `wai why <query>`
- **THEN** the system uses LLM synthesis to answer why decisions were made
- **AND** displays relevant artifacts, decision chains, and suggestions
- **AND** gracefully falls back to `wai search` if no LLM available

#### Scenario: Reflect on session history

- **WHEN** user runs `wai reflect`
- **THEN** the system reads accumulated handoffs and artifacts
- **AND** optionally accepts a conversation transcript via `--conversation <file>`
- **AND** uses LLM synthesis to surface project-specific conventions, gotchas, and patterns
- **AND** shows a unified diff of old vs proposed REFLECT block content
- **AND** requires user confirmation before writing to CLAUDE.md and/or AGENTS.md
- **AND** updates whichever AI config files exist in the repo root by default
- **AND** fails with a clear diagnostic if no LLM is available (does not fall back)

#### Scenario: Diagnose workspace health

- **WHEN** user runs `wai doctor`
- **THEN** the system runs diagnostic checks against the workspace
- **AND** reports pass/warn/fail status for each check with fix suggestions
- **AND** exits with code 0 when all checks pass, 1 when any check fails

### Requirement: Global Flags

The CLI SHALL support global verbosity and quiet flags that work with all commands.

#### Scenario: Verbose output (level 1)

- **WHEN** user passes `-v` or `--verbose`
- **THEN** output includes additional context and metadata

#### Scenario: Verbose output (level 2)

- **WHEN** user passes `-vv` or `--verbose --verbose`
- **THEN** output includes debug information

#### Scenario: Verbose output (level 3)

- **WHEN** user passes `-vvv` or `--verbose --verbose --verbose`
- **THEN** output includes trace-level details

#### Scenario: Quiet mode

- **WHEN** user passes `-q` or `--quiet`
- **THEN** only errors are shown

#### Scenario: Non-interactive mode

- **WHEN** user passes `--no-input`
- **THEN** the system disables interactive prompts and fails with a diagnostic error if input is required

#### Scenario: Auto-confirm

- **WHEN** user passes `--yes`
- **THEN** the system proceeds with default choices for confirmations

#### Scenario: Safe mode

- **WHEN** user passes `--safe`
- **THEN** the system runs in read-only mode and refuses operations that mutate state, returning a diagnostic error with a suggested non-safe command

### Requirement: JSON Output

Commands that return multi-line structured information SHALL support `--json` output for machine parsing.

#### Scenario: Status as JSON

- **WHEN** user runs `wai status --json`
- **THEN** the system outputs JSON containing phase, plugin statuses, and suggestion lists

#### Scenario: Search as JSON

- **WHEN** user runs `wai search <query> --json`
- **THEN** the system outputs JSON containing matches with file paths, line numbers, and context

#### Scenario: Timeline as JSON

- **WHEN** user runs `wai timeline <project> --json`
- **THEN** the system outputs JSON containing entries with date, type, title, and path

#### Scenario: Plugin list as JSON

- **WHEN** user runs `wai plugin list --json`
- **THEN** the system outputs JSON containing plugin name, status, and detector metadata

### Requirement: Project Initialization

The CLI SHALL provide `wai init` to initialize a project in the current directory.

#### Scenario: Interactive initialization

- **WHEN** user runs `wai init` without arguments
- **THEN** the system prompts for project name (defaulting to directory name)
- **AND** creates `.wai/` structure with PARA directories (projects, areas, resources, archives, plugins)
- **AND** creates default agent-config directory with `.projections.yml`
- **AND** auto-detects available plugins (beads, openspec, git)

#### Scenario: Named initialization

- **WHEN** user runs `wai init --name my-project`
- **THEN** the system creates the project with the specified name without prompting

#### Scenario: Already initialized

- **WHEN** user runs `wai init` in an already-initialized directory
- **THEN** the system shows a warning and suggests `wai status`

### Requirement: Status Command

The CLI SHALL provide `wai status` to show project overview and suggest next steps.

#### Scenario: Show project phase and status

- **WHEN** user runs `wai status`
- **THEN** the system displays the current project's phase
- **AND** shows plugin status summaries (beads issues, openspec changes, git status)
- **AND** shows contextual suggestions based on current phase

#### Scenario: Contextual suggestions

See [context-suggestions](../context-suggestions/spec.md) for the complete suggestion logic.

### Requirement: Doctor Command

The CLI SHALL provide `wai doctor` to diagnose workspace health and report issues with actionable fix suggestions.

#### Scenario: Directory structure check

- **WHEN** `wai doctor` runs
- **THEN** it verifies that all expected `.wai/` subdirectories exist (projects, areas, resources, archives, plugins)
- **AND** reports pass if all present, fail with `mkdir` suggestion for each missing directory

#### Scenario: Configuration validation

- **WHEN** `wai doctor` runs
- **THEN** it attempts to parse `.wai/config.toml`
- **AND** reports pass if valid, fail with the parse error and suggestion to check the file

#### Scenario: Plugin tool availability

- **WHEN** `wai doctor` runs and plugins are detected
- **THEN** it checks whether each detected plugin's CLI tool is installed (e.g., `git`, `bd`, `openspec`)
- **AND** reports pass if reachable, warn if not installed with install guidance

#### Scenario: Agent config sync status

- **WHEN** `wai doctor` runs and `.projections.yml` exists
- **THEN** it validates the projections file parses correctly
- **AND** checks whether each projection target exists and is up to date
- **AND** reports pass if synced, warn if targets are missing with `wai sync` suggestion

#### Scenario: Project state integrity

- **WHEN** `wai doctor` runs and projects exist
- **THEN** it validates each project's `.state` file parses as valid YAML with a recognized phase
- **AND** reports pass if valid, fail with the error for each invalid state file

#### Scenario: Custom plugin validation

- **WHEN** `wai doctor` runs and `.wai/plugins/` contains YAML files
- **THEN** it validates each plugin YAML parses correctly as a PluginDef
- **AND** reports pass if valid, fail with the parse error for each invalid file

#### Scenario: Summary output

- **WHEN** all diagnostic checks complete
- **THEN** the system prints a summary line with total pass, warn, and fail counts
- **AND** exits with code 0 if no failures, code 1 if any failures

#### Scenario: Not initialized

- **WHEN** user runs `wai doctor` outside a wai workspace
- **THEN** the system reports the standard not-initialized error with `wai init` suggestion
## Requirements
### Requirement: Way Command

The CLI SHALL provide `wai way` to validate repository best practices and provide opinionated recommendations ("the wai way") using a dedicated status model separate from `wai doctor`.

#### Scenario: Repository standards check

- **WHEN** `wai way` runs
- **THEN** it checks for repository best practices as defined in [repository-best-practices](../repository-best-practices/spec.md)
- **AND** includes checks for task runner, git hook manager (prek/pre-commit), EditorConfig, documentation, AI instructions, llm.txt, agent skills, CI/CD, and dev containers
- **AND** reports each check as WayStatus::Pass (✓) or WayStatus::Info (ℹ) with actionable suggestions

#### Scenario: Works without wai initialization

- **WHEN** user runs `wai way` in any directory (wai workspace or not)
- **THEN** the command succeeds and runs all repository checks
- **AND** does NOT require `.wai/` to exist
- **AND** helps users prepare repositories before running `wai init`

#### Scenario: Summary output

- **WHEN** all checks complete
- **THEN** the system prints a summary: "X/Y best practices adopted"
- **AND** suggests quick-start priorities if many checks are info status
- **AND** always exits with code 0 (recommendations never fail)

#### Scenario: Output format

- **WHEN** `wai way` runs
- **THEN** output is grouped under "The wai way" header
- **AND** uses ✓ (green) for WayStatus::Pass, ℹ (blue) for WayStatus::Info
- **AND** each check follows format: "Category: Status (details)"
- **AND** includes actionable fix text with URLs (in parentheses) for each WayStatus::Info
- **AND** supports `--json` flag for machine-readable output
- **AND** critical recommendations (missing .gitignore/README.md) display with ⚠️ marker

#### Scenario: Fix skills

- **WHEN** user runs `wai way --fix skills`
- **THEN** the system scaffolds missing recommended agent skills (rule-of-5-universal, commit)
- **AND** skips skills that already exist
- **AND** exits with code 0
- **AND** does NOT run the normal check output (fix mode is separate from check mode)

### Requirement: Session Close Command

`wai close` SHALL write a `.pending-resume` signal after every successful handoff
creation.

#### Scenario: Pending-resume written on success

- **WHEN** `wai close` successfully creates a handoff document
- **THEN** the system writes `.wai/projects/<project>/.pending-resume` containing
  the path to the new handoff, relative to the project directory
- **AND** this file is not mentioned in the command's terminal output
- **AND** the file appears in the uncommitted-changes list (it is a tracked
  workspace artifact, committed with other `.wai/` changes)

---

### Requirement: Global Workspace View

The CLI SHALL provide `wai ls` to scan for wai workspaces under a root directory and
display each project's phase and open issue count as a single line per project.

The expected terminal output shape is:

```
why-command   [review]    3 open, 2 ready
para          [plan]      7 open, 0 ready
rizomas       [implement] 1 open, 1 ready
```

Columns are left-aligned and padded to the longest project name in the result set. The
counts column is a **global toggle**: it appears for ALL rows when at least one workspace
has beads data (rows without beads show a blank cell), and is omitted entirely when no
workspace has beads. When no workspaces are found, the system prints a single message
indicating the root that was scanned. When two projects in different workspaces share the
same name, a short path suffix disambiguates each: `name (~/path/to/repo)`.

The default root is `$HOME`; the default depth is 3. Both are overridable via flags.
The filesystem walker never follows symlinks.

#### Scenario: Workspaces found — table rendered

- **WHEN** user runs `wai ls` and at least one `.wai/config.toml` exists under `$HOME` at depth ≤ 3
- **THEN** the system displays one line per (workspace, project) pair
- **AND** each line shows the project name, phase in brackets, and beads counts when available

#### Scenario: No workspaces found

- **WHEN** user runs `wai ls` and no `.wai/config.toml` is found under the root
- **THEN** the system prints `No wai workspaces found under <root>`

#### Scenario: Custom root

- **WHEN** user runs `wai ls --root <path>`
- **THEN** the system scans from `<path>` instead of `$HOME`

#### Scenario: Custom depth

- **WHEN** user runs `wai ls --depth <n>`
- **THEN** the system limits filesystem traversal to `<n>` levels below the root

#### Scenario: At least one workspace has beads — counts column shown globally

- **WHEN** at least one discovered workspace has `.beads/` present and `bd` is installed
- **THEN** the counts column appears for all rows in the output
- **AND** rows for projects in beads-enabled workspaces show `N open, M ready`
- **AND** rows for projects in workspaces without beads show a blank counts cell

#### Scenario: No workspace has beads — counts column omitted

- **WHEN** no discovered workspace has `.beads/` present or `bd` is not installed anywhere
- **THEN** the counts column is omitted entirely from the output

#### Scenario: Multiple projects in one workspace

- **WHEN** a discovered workspace contains more than one project
- **THEN** each project appears as a separate line in the output

#### Scenario: Duplicate project names across workspaces

- **WHEN** two projects from different workspaces share the same name
- **THEN** each line appends a short path suffix to disambiguate: `name (~/path/to/repo)`

#### Scenario: Invalid root path

- **WHEN** user runs `wai ls --root <path>` and `<path>` does not exist
- **THEN** the system fails with a diagnostic error naming the invalid path

### Requirement: Session Orientation Command

`wai prime` SHALL detect a `.pending-resume` signal and render a `⚡ RESUMING`
block when the referenced handoff is dated today.

#### Scenario: Resume mode — today's handoff

- **WHEN** user runs `wai prime`
- **AND** `.wai/projects/<project>/.pending-resume` exists
- **AND** the referenced handoff file exists and its frontmatter `date` equals
  today's date
- **THEN** the system renders a `⚡ RESUMING` block before the plugin status lines
- **AND** the block shows the handoff date and one-line snippet on the first line
- **AND** the block shows the contents of the handoff's `## Next Steps` section
  immediately below, rendered as described in "Resume mode — next steps rendering"
- **AND** the normal `• Handoff:` line is omitted (replaced by the RESUMING block)
- **AND** the `.pending-resume` file is NOT modified or deleted

#### Scenario: Resume mode — next steps rendering

- **WHEN** the RESUMING block is rendered
- **THEN** the `  Next Steps:` label is printed indented two spaces from the
  left margin, with no `##` heading markers and with a trailing colon
- **AND** each content line from the `## Next Steps` section is printed indented
  four spaces from the left margin
- **AND** blank lines and lines starting with `<!--` within the section are skipped

#### Scenario: Resume mode — next steps present

- **WHEN** the handoff referenced by `.pending-resume` contains a `## Next Steps`
  section with renderable content (non-blank, non-comment lines)
- **THEN** the RESUMING block shows the `  Next Steps:` label followed by the items

#### Scenario: Resume mode — no next steps section

- **WHEN** the handoff referenced by `.pending-resume` does not contain a
  `## Next Steps` section, or the section contains only blank lines and HTML
  comments
- **THEN** the RESUMING block shows only the `⚡ RESUMING: {date} — '{snippet}'`
  header line with no indented items

#### Scenario: Signal not consumed by prime

- **WHEN** user runs `wai prime` and a RESUMING block is rendered
- **THEN** the `.pending-resume` file is NOT modified or deleted
- **AND** a subsequent `wai prime` call in the same session renders the same
  RESUMING block again

#### Scenario: Stale signal — not today's handoff

- **WHEN** user runs `wai prime`
- **AND** `.wai/projects/<project>/.pending-resume` exists
- **AND** the referenced handoff's frontmatter `date` is before today, or the
  date field is missing or unparseable
- **THEN** the system ignores the signal entirely
- **AND** renders the normal `• Handoff:` line using the latest handoff

#### Scenario: Missing handoff — signal ignored

- **WHEN** user runs `wai prime`
- **AND** `.pending-resume` exists but the referenced file does not exist on disk
- **THEN** the system ignores the signal
- **AND** renders the normal `• Handoff:` line (or omits the line if no handoffs
  exist at all)

