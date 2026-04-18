# Implementation Status

At-a-glance view of wai's implemented commands and their relation to formal
change proposals tracked in `openspec/`.

---

## Commands

| Command | Description |
|---------|-------------|
| `wai init` | Initialize wai in any directory |
| `wai tutorial` | Run the interactive quickstart tutorial |
| `wai status` | Project overview and next-step suggestions |
| `wai prime` | Orient the session; detect resume state |
| `wai close` | Capture session state and signal pending resume |
| `wai handoff create <project>` | Generate a handoff document explicitly |
| `wai show` | Inspect projects, areas, and resources |
| `wai ls` | List all wai workspaces under a root |
| `wai new` | Create projects, areas, and resources |
| `wai add` | Add research, plans, designs, and reviews |
| `wai search` | Full-text artifact search with tag filtering |
| `wai timeline` | Chronological artifact view across projects |
| `wai why` | Ask the LLM oracle about design decisions |
| `wai reflect` | Synthesize project patterns into CLAUDE.md |
| `wai sync` | Pull remote artifact updates |
| `wai move` | Relocate projects and areas |
| `wai import` | Import existing AI tool configs (.claude/, .cursorrules) |
| `wai phase` | Show, advance, or set the current project phase |
| `wai doctor` | Diagnose and repair workspace health |
| `wai way` | Check and scaffold repository best practices |
| `wai config` | Inspect workspace configuration |
| `wai resource` | Manage skills, rules, and context resources |
| `wai plugin` | Enable and disable workspace plugins |
| `wai pipeline` | Run structured multi-step research workflows |
| `wai project` | Session-scope a project (`WAI_PROJECT` env var) |
| `wai trace` | *(Planned)* Import session traces from agent tools |

For the authoritative list run `wai --help`.

---

## Change Proposals

Formal changes are tracked in `openspec/changes/`. Each entry links a
capability to the reasoning that motivated it. Status values come from
`openspec list`; task counts from `tasks.md` in each change directory.

| Change | Status | Related Commands |
|--------|--------|-----------------|
| `add-cross-tool-trace-ingestion` | In progress (0/10) | `wai trace`, `wai reflect` |
| `add-artifact-integrity` | Complete | `wai pipeline lock/verify`, `wai doctor` |
| `add-project-resolution` | Complete | `wai project use`, `wai ls` |

Run `openspec list` for the full list including archived changes.

---

## Capability Specs

Stable capabilities have formal specs in `openspec/specs/`. These define the
expected behavior independently of any single change.

Run `openspec list --specs` to see current specs.

---

## Release History

Releases follow calendar versioning (`vYYYY.M.N`). See the
[GitHub Releases page](https://github.com/Fission-AI/wai/releases) for the
full changelog.
