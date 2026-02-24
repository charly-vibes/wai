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
counts column is shown only when at least one workspace has beads detected; it is omitted
entirely otherwise. When no workspaces are found, the system prints a single message
indicating the root that was scanned.

The default root is `$HOME`; the default depth is 3. Both are overridable via flags.

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

#### Scenario: Beads detected — counts shown

- **WHEN** a discovered workspace has `.beads/` present and `bd` is installed
- **THEN** the line for each project in that workspace shows `N open, M ready`

#### Scenario: Beads not detected — counts omitted

- **WHEN** a discovered workspace does not have `.beads/` or `bd` is not installed
- **THEN** the counts portion of the line is omitted for projects in that workspace

#### Scenario: Multiple projects in one workspace

- **WHEN** a discovered workspace contains more than one project
- **THEN** each project appears as a separate line in the output
