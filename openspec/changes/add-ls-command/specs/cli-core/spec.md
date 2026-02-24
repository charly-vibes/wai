## ADDED Requirements

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
