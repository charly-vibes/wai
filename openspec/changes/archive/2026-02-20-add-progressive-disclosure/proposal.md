# Change: Add Progressive Disclosure Help System

## Why

Current help is all-or-nothing. Users need tiered verbosity: simple success messages by default, detailed logs for debugging, full traces for experts.

## What Changes

- Three verbosity modes: beginner (default), intermediate (-v), expert (-vv)
- Beginner: simple success + 3-4 next steps + link to docs
- Intermediate: execution log + plugin hooks + files created
- Expert: full trace + state machine transitions + performance metrics
- Examples-first help pages

## Impact

- Affected specs: help-system
- Affected code: all command handlers, output formatting
