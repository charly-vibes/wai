# Change: Add Self-Healing Error Patterns

## Why

Current errors suggest fixes but don't detect common user mistakes like typos, wrong command order, or missing context. Self-healing errors should detect these patterns and offer exact fixes.

## What Changes

- Add typo detection with "did you mean?" suggestions
- Add context inference (guess project from current directory)
- Add wrong-order detection (e.g., `wai bead new` → `wai new bead`)
- Add sync conflict resolution suggestions
- Make errors conversational ("Let's fix this" not "Command failed")

## Impact

- Affected specs: error-recovery
- Affected code: `src/error.rs`, command handlers
