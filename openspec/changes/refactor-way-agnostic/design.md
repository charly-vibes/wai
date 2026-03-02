# OpenSpec Design: Agnostic Way Capabilities

**Change ID:** `refactor-way-agnostic`
**Status:** `draft`
**Author:** `Gemini CLI`

## Architectural Reasoning

The `wai way` command's current architecture is a direct mapping of hardcoded tool-check functions to a `CheckResult` structure. While simple, it's brittle and lacks the "why" that AI agents need to work autonomously.

The proposed design shifts the focus from **Discovery** (finding a file) to **Validation of Capability** (ensuring the project has standardized commands, quality gates, etc.).

### 1. Unified Data Model

The `CheckResult` structure will be expanded to support this, using `Option<String>` for the new fields to ensure backward compatibility for JSON consumers:

```rust
struct CheckResult {
    name: String,            // Capability name (e.g., "Command standardization")
    status: Status,
    message: String,         // Tool-specific discovery result (e.g., "justfile detected")
    intent: Option<String>,          // The "Why" (agnostic)
    success_criteria: Option<String>, // The "What" (agnostic)
    suggestion: Option<String>,
}
```

### 2. Check Renaming (Tool to Capability)

The existing 11 checks will be mapped to agnostic capabilities. The **Message** field continues to report tool-specific discovery results, while **Intent** and **Success Criteria** provide the agnostic context.

| Existing Check | Agnostic Capability | Intent | Success Criteria |
| :--- | :--- | :--- | :--- |
| **Task runner** | **Command standardization** | Provide a single, tool-agnostic entry point for common repository tasks (build, test, deploy). | A standard interface (justfile, Makefile, npm scripts) exists for common tasks. |
| **Git hooks** | **Pre-commit quality gates** | Prevent low-quality commits by running automated checks before code is saved to history. | Automated checks (linters, tests) run automatically before code is committed. |
| **Editor config** | **Consistent formatting** | Ensure consistent code formatting across different editors and IDEs. | Project-wide style rules are enforced by a shared configuration file. |
| **Documentation** | **Project documentation** | Provide essential project identity, onboarding, and legal/contribution guidance. | Essential files (README, .gitignore, LICENSE) provide project context and rules. |
| **AI instructions** | **AI-agent context** | Provide persistent "rules of the road" and project context for AI collaborators. | Persistent instructions define coding standards and context for AI assistants. |
| **LLM documentation** | **LLM-friendly context** | Provide machine-readable project context and navigation for LLMs. | Machine-readable project documentation (llm.txt) exists for AI tools. |
| **Agent skills** | **Extended agent capabilities** | Enhance agent functionality with specialized iterative review and commit workflows. | Specialized agent workflows (Rule of 5, Deliberate Commits) are active. |
| **GitHub CLI** | **Integration & automation** | Streamline repository interactions (PRs, issues, releases) from the CLI. | CLI tools are configured for seamless integration with the hosting provider. |
| **CI/CD** | **Automated verification** | Ensure code quality and correctness through automated builds and tests on every change. | Every change is automatically validated by a remote build/test pipeline. |
| **Dev container** | **Reproducible environments** | Provide a standardized, containerized environment for all contributors. | A configuration exists to spin up a consistent, reproducible dev environment. |
| **Release pipeline** | **Automated delivery** | Automate the process of building, packaging, and publishing software releases. | Software releases and distribution (packages, binaries) are fully automated. |

### 3. Custom Plugin Support

The plugin system will be updated to allow plugins to provide their own agnostic context. The `PluginDef` struct gains `intent: Option<String>` and `success_criteria: Option<String>` with `#[serde(default)]` so they are optional in any plugin file.

Plugin definitions use **TOML** (`.wai/plugins/*.toml`), consistent with every other config file in the project (`config.toml`, `Cargo.toml`, `prek.toml`). The `toml` crate is already a dependency. The original proposal used YAML — that was corrected here before any users existed so there is no migration burden.

Example plugin file (`.wai/plugins/my-check.toml`):

```toml
name = "my-check"
description = "Custom check"
intent = "Verify domain-specific rule X"
success_criteria = "Rule X is satisfied according to tool Y"
```

If these fields are missing from a plugin definition, `wai way` will provide sensible default "Generic check" intent/criteria.

### 3. Progressive Disclosure (Human Output)

To keep the standard output clean, the `intent` and `success_criteria` will only be rendered in the human-readable output when `context.verbose > 0`. 

**Example Verbose Output:**
```
✓ Command standardization: justfile detected (recipes: test, lint, fmt)
  Intent: Provide a single, tool-agnostic entry point for common repository tasks (build, test, deploy).
  Success: A task runner (justfile, Makefile, etc.) is present and defines standard recipes.
```

### 4. Machine-Readable Context (JSON)

The `--json` output will always include the `intent` and `success_criteria` fields, making them immediately available to AI agents for reasoning.

## Trade-offs

- **Verbosity:** Adding these fields increases the size of the JSON payload.
- **Maintenance:** The "intent" and "success_criteria" strings must be kept up-to-date as repository best practices evolve.
- **Complexity:** The `CheckResult` struct becomes slightly larger, but the logic for checking remains simple.

## Decision

We will proceed with the unified data model and renaming of checks to better serve both human users and AI agents. Plugin files use TOML for consistency with the rest of the project.

## Implementation Status

As of 2026-03-02, review of the codebase shows most structural work is already in place:

- `CheckResult` struct already has `intent`/`success_criteria` fields (`src/commands/way.rs`)
- `render_human` already renders them under `--verbose`
- All 11 original check functions already use agnostic names with intent/success criteria
- `PluginDef` already has `intent`/`success_criteria` with `#[serde(default)]`
- `openspec validate --strict` passes

**Remaining work:**

1. **Migrate plugin loader from YAML to TOML** (`src/plugin.rs`): change file extension filter from `yml`/`yaml` to `toml`, swap `serde_yml::from_str` for `toml::from_str`.

2. **Add tests for new fields:**
   - `way_verbose_shows_intent` — `wai way -v` output contains `"Intent:"`
   - `way_verbose_shows_success_criteria` — `wai way -v` output contains `"Success:"`
   - `way_json_includes_intent` — `wai way --json` payload contains `"intent"` key
   - `way_plugin_toml_parsed` — a `.toml` plugin file with `intent`/`success_criteria` is loaded correctly

3. **Update spec**: add TOML plugin format scenario to `Plugin Agnostic Context Support` requirement.

4. **Validate and archive**: run `openspec validate refactor-way-agnostic --strict`, then archive the change.
