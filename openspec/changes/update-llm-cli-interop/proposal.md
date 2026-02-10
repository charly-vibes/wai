# Change: Update LLM CLI Interop

## Why

LLM-driven automation needs deterministic, machine-readable outputs and safe non-interactive behaviors. The current specs contain contradictions around no-args behavior and lack explicit requirements for structured outputs, exit codes, or safe plugin passthrough, making reliable LLM interaction risky.

## What Changes

- Reconcile no-args behavior between onboarding and help-system
- Add explicit JSON output support for status/search/timeline/plugin commands
- Define non-interactive and safe modes with consistent exit codes
- Add structured error payload expectations alongside human-readable diagnostics
- Standardize suggestion output blocks for machine parsing

## Impact

- Affected specs: cli-core, help-system, onboarding, plugin-system, error-recovery, context-suggestions, timeline-search
- Affected code: output formatters, command runners, error handling, plugin passthrough
