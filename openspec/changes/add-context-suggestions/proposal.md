# Change: Add Context-Aware Suggestions

## Why

Users often don't know the next logical step. The CLI should detect common workflows and suggest what to do next based on current project state.

## What Changes

- Post-command suggestions based on what just happened
- Workflow detection (after creating project → suggest adding research)
- Phase-aware suggestions (implement phase → start implementing)
- Command chaining suggestions after success or error

## Impact

- Affected specs: context-suggestions
- Affected code: all command handlers, new `src/workflows.rs`
