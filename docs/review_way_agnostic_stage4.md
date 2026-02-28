# Rule of 5 Review: Agnostic Way Capabilities

**Work Reviewed:** OpenSpec Proposal (`openspec/changes/refactor-way-agnostic/`)
**Stage 4: EDGE CASES**

Issues Found:

[EDGE-001] [CRITICAL] - `CheckResult` JSON compatibility
Description: The `CheckResult` struct change adds two required fields (`intent` and `success_criteria`). Existing JSON consumers (scripts, webhooks) that expect the old structure *might* break if they use strict schema validation.
Scenario: A CI script uses `jq` to select fields. If the script uses `map({name, status, message, suggestion})`, it will likely be fine. If it uses a more rigid validation, it might fail.
Impact: Breaking change for strict JSON consumers.
Recommendation: Mark these fields as optional (`Option<String>`) or provide defaults during serialization/deserialization to ensure backward compatibility for consumers that don't yet expect them. (Self-correction: For a version 0.x tool, adding fields is generally considered safe if they don't rename existing ones).

[EDGE-002] [HIGH] - Custom Plugins
Description: Does this change affect how custom plugins (defined in `.wai/plugins/*.yml`) report their results? 
Scenario: A plugin provides a custom check but only uses `name`, `status`, and `message`. It won't have `intent` or `success_criteria`.
Impact: Custom plugins might show empty or default strings for these fields.
Recommendation: Ensure the plugin system provides default "Intent" and "Success Criteria" for custom plugins, or allows them to define their own in the YAML.

[EDGE-003] [LOW] - Output truncation
Description: The verbose output (`-v`) will significantly increase the lines of text. 
Scenario: A repository with 15+ checks (including plugins) is checked on a small terminal window.
Impact: Important information might scroll off-screen.
Recommendation: Ensure the human-readable output remains scannable even with verbose details enabled. (The current `render_human` looks good for this).

Edge Case Coverage: GOOD (Though EDGE-001/002 are important)

**Convergence Check (after Stage 4):**
New CRITICAL issues: 1
Total new issues: 3
New issues vs Stage 3: 0% change in count.
Status: CONTINUE
