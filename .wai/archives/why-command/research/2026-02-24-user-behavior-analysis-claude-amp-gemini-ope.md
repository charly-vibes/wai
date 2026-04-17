# User Behavior Analysis: Claude, Amp, Gemini, OpenCode

> **Scope note:** This research crosses wai broadly (prime, status, close, sync, global view).
> It belongs in a new project (e.g. `wai-ux`) rather than `why-command`. Filed here as input
> for that future project — do not extend `why-command` to cover these items.

## Data Sources

| Tool | Volume | Limitation |
|------|--------|------------|
| Claude Code | 576 JSONL files, 2161 history entries, 20+ projects | Full history available |
| Amp | 282 threads (up to 473 messages), 26 skills | Full history available |
| Gemini | ~15 sessions | **One week only (Dec 5–11, 2025); not representative of ongoing usage** |
| OpenCode | 40 prompt history entries | **Minimal usage; most entries are 1–3 words. Excluded from analysis.** |
| Goose | 10 LLM request logs | Limited sample |
| Global AGENTS.md | Full user workflow config at `~/AGENTS.md` | Ground truth for preferences |

## Observations

### 1. Session Start Friction (highest frequency)

**Context loss and re-orientation consume significant effort across every session.**

Evidence:
- **233 `/clear` commands** in Claude — constantly starting fresh sessions
- **189 `/rate-limit-options` hits** — forced breaks interrupt flow mid-session
- Common session openers: `"work"`, `"start working"`, `"continue"`, `"what's the status?"`, `"how do we continue?"`
- Heavy investment in handoff + resume skills to bridge sessions (Amp: `create-handoff`, `resume-handoff`; Claude: `/resume`)

The `bd prime` Claude Code startup hook (configured in `~/.claude/settings.json` under `SessionStart` and `PreCompact`) auto-runs at session start and after context compaction, providing beads-specific orientation. No equivalent exists for wai or the full toolchain.

The ideal: say `"work"` → agent reads all state and proposes the next action. Currently requires: `wai status` + `bd ready` + `openspec list` + reading last handoff = 4 separate commands.

### 2. Delegation Mode

> **Definition:** The user sets an intent and approves or steers agent actions rather than directing each step individually.

Majority of user messages in Amp are empty strings (approve tool use), `"y"`, `"ok"`, `"commit and push"` (100× in Claude history), `"fix all"`, `"continue"`. The user monitors by interrupting (`[Request interrupted by user for tool use]`) but doesn't micromanage.

This pattern means: tools that require explicit commands to capture context go against the grain. `wai add research "..."` demands a full command and content at a moment when the user is approving-not-directing.

### 3. Multi-Tool Skill Duplication

User maintains parallel skill sets across tools:
- **Amp**: 26 skills in `~/.config/amp/skills/`
- **Claude Code**: skills in `~/.claude/skills/`, commands in `.claude/commands/`
- **Gemini**: separate config in `~/.gemini/`

Evidence from Amp threads of manual duplication:
- `"add https://github.com/humanlayer/.../commit.md but adapt them to this repo"` — adapting the same skill per tool
- `"move the commands to .agents/commands and create a just command to symlink them to .claude/commands"` — attempted unification

wai's `resources/agent-config/` + `wai sync` were built exactly for this, but `wai sync` requires manual invocation and only targets project-relative paths (`.claude/`, `.cursorrules`, etc.), not global tool configs.

### 4. Terse, Intent-First Communication

User messages are short and assume the agent reads state from files, not prompts:
- Abbreviations: `"mvh"`, `"mkr"`, `"1 2 and 3"`
- Mixes Spanish/English: `"revisa las gh actions porque siempre fallaron"`
- Rarely explains context — expects agents to discover it

### 5. Spec-Driven, Research-Heavy Process

Configured Amp skills include: `design-practice`, `plan-review`, `research-review`, `research-codebase`. Pattern of work matches wai phases: research → plan → design → implement. But **artifact capture is reactive** — it happens when the agent suggests it, not as a user habit.

### 6. Session End Neglect

`CLAUDE.md` contains a manual session-close checklist. Evidence of incomplete follow-through:
- Many Amp threads end with `"commit and push"` — no handoff, no artifact
- Across **all** repos that use wai, there are 6 handoff files total (3 in wai, 2 in rizomas, 1 in fotos) over several months
- `wai add research` is typically agent-initiated, not user-initiated

The low handoff count reflects a habit gap, not a tooling gap — `wai handoff create` already exists. The trigger is missing.

### 7. Tool-Specific Usage Niches

The three main tools serve distinct roles, not overlapping ones:
- **Claude Code**: wai repo development (Rust, specs, openspec)
- **Amp**: frontend/data projects (fabbro, hipervinculos, scientific notebooks, sxAct)
- **Gemini**: quick content edits (Phorma website HTML, Dec 2025 only)

Improvements to wai for session orientation matter most in **Claude Code and Amp** where long multi-phase projects run. Gemini usage is too thin and project-specific to generalize from.

## What Already Exists

Before proposing new features, mapping pain points to current capabilities:

| Pain Point | Existing Capability | Actual Gap |
|-----------|--------------------|----|
| Session orientation | `wai status`, `bd prime` hook | No unified single command; no handoff summary in status |
| Artifact capture | `wai add research`, `wai add --file`, `wai import` | No low-friction path; no agent-side trigger |
| Skill projection | `wai sync` (project-relative targets) | Doesn't target global tool dirs (`~/.config/amp/`, `~/.gemini/`) |
| Session close | `wai handoff create <project>` | No trigger; no reminder; not integrated with git status / bd sync |
| AGENTS.md / CLAUDE.md | `wai init` injects managed block into repo-local files | Doesn't touch global `~/AGENTS.md` |
| Cross-project view | None | Genuine gap |
| Tool health | `wai doctor` (repo-scoped) | Doesn't check whether skills in `agent-config/` are in sync with tool dirs |

**Key finding:** The habit gap is as large as the feature gap. Several pain points are solved by existing commands that aren't being triggered at the right moment.

## Gaps

> **Value rubric:** HIGH = daily friction point, MEDIUM = weekly friction, LOW = occasional / nice-to-have

### A. No unified session orientation (HIGH)

`bd prime` orients for beads. `wai status` shows project/phase. Nothing combines both with a handoff summary and openspec progress into a single "what's the situation?" answer.

Relates to: openspec onboarding spec — covers first-run but not per-session orientation.

### B. Session close has no trigger (HIGH)

`wai handoff create` exists but is never triggered automatically. The manual checklist in CLAUDE.md isn't followed consistently — 6 handoffs across months of multi-project work confirms this.

Relates to: handoff-system spec — explicitly marks "automatic handoff generation" as a non-goal. That non-goal may need revisiting for hook-based triggering.

### C. Agent config sync doesn't reach global tool dirs (HIGH)

`wai sync` reads `.projections.yml` and writes to project-relative targets. It never touches `~/.config/amp/`, `~/.gemini/`, or global `~/AGENTS.md`. This is confirmed by source (`src/commands/sync.rs`).

Relates to: agent-config-sync spec — scope is currently per-project projections; global sync is out of scope.

### D. Artifact capture requires an active command (MEDIUM)

`wai add research "..."` requires intent + content at the moment of capture. During delegation mode, this is too high a bar. `wai add --file` and `wai import` exist but still require a file to exist first.

### E. No cross-project global view (MEDIUM)

`wai status` is per-repo. No way to see all projects using wai and their phase/open issues at a glance. Genuine gap with no existing solution.

### F. Tool health not checked end-to-end (LOW)

`wai doctor` is repo-scoped. It doesn't verify that skills defined in `agent-config/` are actually projected to `~/.config/amp/skills/`, `~/.claude/skills/`, etc.

## Improvement Opportunities

> For each HIGH improvement, the implementation path is evaluated: (a) wai CLI change,
> (b) shell alias/script, (c) hook or AGENTS.md instruction — cheapest path first.

### HIGH VALUE

**1. Unified session orientation — addresses Gap A**

Single command (or startup hook) that shows: current project/phase, last handoff date + first line, `bd ready` output, openspec % complete, suggested next action.

Implementation paths:
- **(c) Hook (cheapest):** Add `wai status` alongside `bd prime` in `~/.claude/settings.json` `SessionStart` hook. Zero Rust changes. Partial solution — doesn't include handoff summary.
- **(b) Shell alias:** `alias work="wai status && bd ready && openspec list"`. Quick and transparent. Still doesn't surface handoff summary.
- **(a) `wai prime` CLI command:** Full solution. Reads last handoff, calls plugin hooks for bd/openspec output, formats into a single oriented view. Requires new subcommand in Rust.

**Recommended path:** Implement (c) immediately (add `wai status` to startup hook), then design (a) `wai prime` as a wai-ux project item.

Success metric: Reduce session-opening messages of `"work"` / `"what's the status?"` pattern; fewer `/clear` + immediate re-orientation sequences.

---

**2. Session close trigger — addresses Gap B**

Surface `wai handoff create` at the right moment so it gets used.

Implementation paths:
- **(c) CLAUDE.md / hook (cheapest):** Add `wai handoff create` to the existing session-close checklist in CLAUDE.md or as a `PreCompact` hook alongside `bd prime`. Cost: editing one config file.
- **(a) `wai close` CLI command:** Wraps `wai handoff create <project>` + prints `bd sync --from-main` reminder + shows `git status`. Delta over bare `wai handoff create`: automation of the checklist into one command. Requires new subcommand.

**Recommended path:** (c) first — add `wai handoff create` to the session-close checklist immediately. Design (a) `wai close` as a wai-ux project item if (c) proves insufficient.

Note: The handoff-system spec marks automatic generation as a non-goal. Hook-based triggering is different (user still decides) and compatible with the spec.

Success metric: Handoff count across active projects increases from ~1/month to ~1/session.

---

**3. Global tool dir sync — addresses Gap C**

`wai sync` should optionally target global tool installations, not just project-relative paths.

Implementation paths:
- **(b) Shell script (cheapest):** A script that copies/symlinks from `.wai/resources/agent-config/skills/` to `~/.config/amp/skills/` and `~/.claude/skills/`. No Rust changes.
- **(a) Extend `wai sync`:** Add a `--global` flag that reads tool installation paths from config and projects to them. Requires extending the projections model and tool detection logic.

**Recommended path:** (b) first to validate the workflow. Then extend the agent-config-sync spec to include global projections before implementing (a).

Note: The agent-config-sync spec currently scopes to per-project projections. Extending to global requires a spec update first.

### MEDIUM VALUE

**4. Smarter `wai status` — addresses Gap A (partial)**

Add to existing status output: last handoff date + first line, open beads issue count, openspec % complete. This is a simpler precursor to `wai prime`.

Implementation: wai status already calls plugin hooks. Extending the beads and openspec plugins to surface a one-line summary is a contained change. Cross-reference: onboarding spec owns in-project no-args behavior — this extends status, not the no-args path.

**5. Low-friction capture — addresses Gap D**

The problem is the trigger, not the command. `wai add --file /tmp/notes.md` already works. The gap is that agents don't produce a file to import — they summarize inline.

Options:
- Add `wai add research` to AGENTS.md instructions so agents do it automatically at session end
- A `wai capture` command that takes a session description and auto-classifies it (research/design/plan) — reduces the cognitive load of choosing artifact type

**6. Global project view — addresses Gap E**

`wai ls` (or extending `wai status` with a `--global` flag) scans parent directories for `.wai/` and reports phase + open issue count. Genuine new capability with no existing equivalent.

### LOW VALUE

**7. Tool health in `wai doctor` — addresses Gap F**

Extend `wai doctor` to check whether skills in `agent-config/` are projected to global tool dirs. Blocked on Gap C being solved first (no point checking something that can't be synced yet).

**8. Stream capture**

`wai note "quick thought"` creates a lightweight artifact without forcing a type choice. Lower friction than `wai add research`. Nice-to-have once the habit gap (session close trigger) is addressed.

## Summary of Recommended Actions (in order)

| Priority | Action | Type | Effort |
|----------|--------|------|--------|
| 1 | Add `wai status` to `SessionStart` hook in `~/.claude/settings.json` | Config change | Minutes |
| 2 | Add `wai handoff create` to CLAUDE.md session-close checklist | Doc change | Minutes |
| 3 | Create a shell script for global skill sync (amp + claude) | Script | Hours |
| 4 | Design `wai prime` and `wai close` as a new `wai-ux` project | New project | Days |
| 5 | Extend beads + openspec plugins to emit one-line status summary | Code change | Hours |
| 6 | Design `wai ls --global` for cross-project view | New project | Days |
