# EPIC: Friction & Roughness Analysis

This epic documents the identified friction points, "roughness," and desired paths for the `wai` tool. As an "annoying" LLM reviewer, I have scrutinized the codebase for inconsistencies, unintuitive behaviors, and workflow dead-ends.

_Written: 2026-02-25. Status tags updated: 2026-03-05._

---

## Status Legend

- **RESOLVED** — Fixed; verified against current source or completed spec.
- **PARTIALLY RESOLVED** — Mitigated but not fully addressed.
- **OPEN** — Still valid; no known fix.

---

## 1. Command Architecture & Naming

### 1.1 Overlapping Intent: `wai way` vs `wai doctor`
- **Status: RESOLVED**
- **Friction:** `wai way` (best practices) and `wai doctor` (workspace health) have significant semantic overlap. Both commands support a `--fix` flag (though `way`'s is more specific).
- **What changed:** The distinction is now documented and cross-referenced in help text. `wai doctor` = workspace health (.wai/ structure, config, projections, plugins). `wai way` = repo hygiene and agent workflow conventions (skills, rules — works without a wai workspace).
- **Decision (2026-03-04):** A unified `wai check` entry point was designed and proposed, then deliberately rejected. The domains are genuinely distinct and a third entry point would worsen discovery. The help text cross-references (73286cd) are sufficient. No further action.

### 1.2 Subcommand Verbosity & Nesting
- **Status: RESOLVED**
- **Friction:** Commands like `wai resource add skill` are four levels deep.
- **What changed:** `wai add skill` is now the primary entry point under `AddCommands::Skill`. `wai resource add skill` still routes to the same implementation but emits a deprecation warning directing users to `wai add skill`.

### 1.3 Command Flag Inconsistencies
- **Status: RESOLVED**
- **Friction:** `wai add research` had `-p` for `--project`; plan and design lacked the short flag.
- **What changed:** All three `AddCommands` variants (Research, Plan, Design) now use `#[arg(short, long)]` for `project`, giving `-p` consistently across all subcommands.

### 1.4 LLM Configuration Fragmentation
- **Status: RESOLVED**
- **Friction:** Both `wai why` and `wai reflect` used LLMs but were configured under `[why]` in `config.toml`.
- **What changed:** Config is now accessed via `config.llm_config()`. The `[why]` TOML section still deserializes for backwards compatibility but emits a deprecation warning.

---

## 2. Workflow & Suggestion Engine

### 2.1 Arbitrary Thresholds & "Dead Zones"
- **Status: RESOLVED**
- **Friction:** `src/workflows.rs` used hardcoded counts with a dead zone: 0–1 items → suggest more; 3+ items → suggest advancing; exactly 2 items → silence.
- **What changed:** Thresholds are now adjacent (`<= 1` and `>= 2`) with no dead zone. Reflection note: "Maintain adjacency when adding new threshold-based suggestions."

### 2.2 Forced Whimsy in Error Messages
- **Status: RESOLVED**
- **Friction:** Error messages in `src/error.rs` started with "Hmm," — annoying in CI or repetitive debugging.
- **What changed:** "Hmm," no longer appears in the current source tree.

### 2.3 Manual Pipeline State
- **Status: RESOLVED**
- **Friction:** `wai pipeline` required manually exporting `WAI_PIPELINE_RUN=<id>`.
- **What changed:** Active run ID is now stored in `.wai/.pipeline-run` (not committed). `wai add` reads env var first, then falls back to the state file. `wai pipeline advance` clears it on last stage.

---

## 3. Implementation Technical Debt

### 3.1 Hardcoded Validation in `mod.rs`
- **Status: PARTIALLY RESOLVED**
- **Friction:** `src/commands/mod.rs` contained a hardcoded `valid_commands` list and `valid_patterns` list for typo/order detection.
- **What changed:** `valid_commands` is now dynamically derived via `wai_subcommand_names()` (uses `Cli::command().get_subcommands()`). A test in `cli.rs` verifies the derived list contains all expected commands.
- **Remaining:** `valid_patterns` (the (verb, noun) pair list for wrong-order detection) is still hardcoded in `run_external`. Must be updated manually when multi-word subcommands are added. Tracked in beads: wai-st9q.

### 3.2 Side-Effects in "Show" Functions
- **Status: RESOLVED**
- **Friction:** `show_welcome` automatically saved user config if missing.
- **What changed:** First-run initialization is now handled inside `UserConfig::load`, keeping `show_welcome` read-only. (Comment at `src/commands/mod.rs:101`.)

### 3.3 Output Formatting Duplication
- **Status: PARTIALLY RESOLVED**
- **Friction:** `show_welcome` and `run_external` had separate suggestion-rendering logic.
- **What changed:** `print_suggestions()` is now a shared `pub(crate)` function in `mod.rs:460`. Both callers use it.
- **Remaining:** JSON output path inside `show_welcome` still builds its suggestion list inline rather than using a shared builder.

---

## 4. Agent-Aware Mode

- **Status: RESOLVED**
- **Friction:** When running inside Claude Code, `wai why`/`wai reflect` would spawn a second LLM subprocess — burning tokens and latency.
- **What changed:** `src/llm.rs` implements `in_agent_session()` which detects `CLAUDECODE`, `WAI_AGENT`, and `CURSOR_AGENT` env vars. The auto-detect path in `detect_backend()` returns `AgentBackend` when in an agent session. `AgentBackend` outputs context via `AGENT_SENTINEL` for the parent agent to synthesize — no subprocess spawned. Both `wai why` and `wai reflect` are wired through this path.

---

## Top 5 Friction Points by Impact

_Updated 2026-03-05: §1.2, §1.3, §4, and the worktree design are all RESOLVED. Only §3.1 (valid_patterns hardcoded) remains open._

1. ~~**§1.2 Subcommand verbosity**~~ — **RESOLVED**: `wai add skill` is now the primary path; `wai resource add skill` deprecated.

2. ~~**§4 Agent-aware mode**~~ — **RESOLVED**: `in_agent_session()` detects Claude Code / Cursor / WAI_AGENT; AgentBackend used automatically.

3. **§1.1 `wai way` vs `wai doctor`** — RESOLVED (cross-references added; `wai check` unification deliberately rejected).

4. **§3.1 Hardcoded `valid_patterns`** — PARTIALLY RESOLVED. `valid_commands` is now dynamic; `valid_patterns` (verb/noun pairs) still hardcoded. Tracked: wai-st9q (P4 backlog).

5. ~~**§1.3 Flag inconsistencies**~~ — **RESOLVED**: `-p` now consistent across research/plan/design.
