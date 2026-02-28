# EPIC: Friction & Roughness Analysis

This epic documents the identified friction points, "roughness," and desired paths for the `wai` tool. As an "annoying" LLM reviewer, I have scrutinized the codebase for inconsistencies, unintuitive behaviors, and workflow dead-ends.

## 1. Command Architecture & Naming

### 1.1 Overlapping Intent: `way` vs `doctor`
- **Friction:** `wai way` (best practices) and `wai doctor` (workspace health) have significant semantic overlap. Both commands support a `--fix` flag (though `way`'s is more specific).
- **Desired Path:** Unify diagnostic commands. Perhaps `wai doctor` should include the "best practices" checks from `way` as a category, or `way` should be the primary entry point for all repository-level validation.

### 1.2 Subcommand Verbosity & Nesting
- **Friction:** Commands like `wai resource add skill` are four levels deep. 
- **Inconsistency:** `wai new resource` exists but is for PARA items, while `wai resource add skill` is for agent-ready skills. This distinction is confusing.
- **Desired Path:** Flatten the hierarchy. `wai add skill` or `wai skill new` would be more direct.

### 1.3 Command Flag Inconsistencies
- **Friction:** 
    - `wai add research` has a short flag `-p` for `--project`.
    - `wai add plan` and `wai add design` only have a long `--project` flag.
- **Desired Path:** Standardize flags across all `add` subcommands.

### 1.4 LLM Configuration Fragmentation
- **Friction:** Both `wai why` and `wai reflect` use LLMs, but they are configured under a `[why]` block in `config.toml`. Users configuring `reflect` may not think to look in `[why]`.
- **Desired Path:** Rename the configuration block to `[llm]` or `[ai]` to reflect its shared usage.

## 2. Workflow & Suggestion Engine

### 2.1 Arbitrary Thresholds & "Dead Zones"
- **Friction:** `src/workflows.rs` uses hardcoded counts for suggestions:
    - 0-1 research items: Suggests adding more.
    - 3+ research items: Suggests advancing phase.
    - **Dead Zone:** If a user has exactly 2 research items, *no* patterns are detected, and suggestions disappear or revert to generic ones.
- **Desired Path:** Replace hardcoded thresholds with more fluid logic or at least eliminate "dead zones" where the tool has nothing to say.

### 2.2 Forced Whimsy in Error Messages
- **Friction:** Many error messages in `src/error.rs` start with "Hmm,". While intended to be friendly, this can be annoying in non-interactive environments or during repetitive debugging.
- **Desired Path:** Move the "whimsy" to the output formatter so it can be toggled or removed in `--quiet` or CI modes.

### 2.3 Manual Pipeline State
- **Friction:** `wai pipeline` requires the user to manually `export WAI_PIPELINE_RUN=<id>` for `wai add` to automatically tag artifacts. This breaks the "flow" and requires the user to manage shell state.
- **Desired Path:** Use a local state file (e.g., in `.wai/`) to track the "active" pipeline run, similar to how git tracks the current branch.

## 3. Implementation Technical Debt

### 3.1 Hardcoded Validation in `mod.rs`
- **Friction:** `src/commands/mod.rs` contains a hardcoded list of `valid_commands` and `valid_patterns` for typo/order detection. This must be updated manually every time a command is added.
- **Desired Path:** Derive these lists dynamically from the `Cli` struct or a registry of commands.

### 3.2 Side-Effects in "Show" Functions
- **Friction:** `show_welcome` in `src/commands/mod.rs` automatically saves the user configuration if it's missing. "Show" functions should be read-only.
- **Desired Path:** Move configuration initialization to a dedicated "pre-run" hook or the `UserConfig::load` method itself.

### 3.3 Output Formatting Duplication
- **Friction:** `show_welcome` and `run_external` both handle their own logic for formatting suggestions and JSON output.
- **Desired Path:** Consolidate all suggestion rendering into `src/output.rs` or a dedicated `Display` trait for suggestions.

## 4. Desired Path: "Agent-Aware" Mode
- **Discovery:** Existing WIP in `openspec/changes/archive/` suggests a move toward "agent-aware" mode.
- **Friction:** When running inside an LLM (like Claude Code), `wai` should not spawn another LLM subprocess.
- **Desired Path:** Detect the parent agent and output raw context for the agent to synthesize, rather than performing its own LLM call. This is already partially planned but not fully implemented.

