# EPIC: Friction & Roughness Analysis

This epic documents the identified friction points, "roughness," and desired paths for the `wai` tool. As an "annoying" LLM reviewer, I have scrutinized the codebase for inconsistencies, unintuitive behaviors, and workflow dead-ends.

_Written: 2026-02-25. Status tags updated: 2026-03-04._

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
- **Status: OPEN**
- **Friction:** Commands like `wai resource add skill` are four levels deep.
- **Inconsistency:** `wai new resource` exists but is for PARA items, while `wai resource add skill` is for agent-ready skills. This distinction is confusing.
- **Desired Path:** Flatten the hierarchy. `wai add skill` or `wai skill new` would be more direct.

### 1.3 Command Flag Inconsistencies
- **Status: OPEN**
- **Friction:**
    - `wai add research` has a short flag `-p` for `--project`.
    - `wai add plan` and `wai add design` only have a long `--project` flag.
- **Desired Path:** Standardize flags across all `add` subcommands.

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
- **Status: OPEN**
- **Friction:** `src/commands/mod.rs:229` contains a hardcoded `valid_commands` list and `valid_patterns` list for typo/order detection. Must be updated manually when commands are added.
- **Desired Path:** Derive dynamically from the `Cli` struct or a command registry.

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

- **Status: OPEN**
- **Discovery:** WIP in `openspec/changes/` points toward agent-aware mode.
- **Friction:** When running inside Claude Code, `wai why`/`wai reflect` should not spawn a second LLM subprocess.
- **Note:** `add-claude-code-projection` (11/11, complete) addresses skill file projection to Claude Code format — a related but distinct concern. The runtime detection problem (suppress LLM calls when parent is an agent) is not yet addressed.
- **Desired Path:** Detect parent agent (e.g., via `CLAUDE_CODE` env var or similar) and output raw context for the agent to synthesize rather than calling the LLM directly.

---

## Top 5 Friction Points by Impact

Priority order for design/plan phase work:

1. **§1.2 Subcommand verbosity** — `wai resource add skill` is the most common agent task and four levels deep. High daily friction. Low implementation risk (CLI refactor).

2. **§4 Agent-aware mode** — When wai spawns an LLM inside Claude Code, it burns tokens and latency for every `wai why` call. Compounds with usage frequency.

3. **§1.1 `wai way` vs `wai doctor`** — Partial fix exists but the UX split still confuses new users. Medium effort to unify under a single entry point.

4. **§3.1 Hardcoded `valid_commands`** — Silent maintenance trap. Every new command risks outdated typo suggestions until someone notices. Low effort to fix with a derive macro or registry.

5. **§1.3 Flag inconsistencies** — Minor daily friction but easy to fix; `-p` standardization across all `add` subcommands is a few-line change.
