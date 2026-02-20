# Change: Add First-Run Experience

## Why

First impressions matter. When users first encounter wai, they should get a guided, welcoming experience that teaches the core workflow without overwhelming them.

## What Changes

- Quickstart tutorial on first run
- Guided project creation flow
- Example workflows shown, not just syntax
- "Getting started" link in all early outputs

## Impact

- Affected specs: onboarding
- Affected code: `src/commands/mod.rs` (welcome), new `src/tutorial.rs`
