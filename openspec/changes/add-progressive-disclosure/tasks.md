## 1. Output Framework

- [x] 1.1 Create `src/help.rs` module with verbosity-aware help rendering
- [x] 1.2 Define HelpContent struct with tiered sections (examples, advanced, env, internals)
- [x] 1.3 Map -v flags to output levels via arg pre-scanning

## 2. Beginner Mode (Default)

- [x] 2.1 Implement simple success messages
- [x] 2.2 Add "Next steps" suggestions (3-4 items)
- [x] 2.3 Add links to examples/docs

## 3. Intermediate Mode (-v)

- [x] 3.1 Add execution step logging
- [x] 3.2 Show plugin hooks that ran
- [x] 3.3 List files created/modified

## 4. Expert Mode (-vv)

- [x] 4.1 Add full execution trace
- [x] 4.2 Show state machine transitions
- [x] 4.3 Add timing/performance metrics

## 5. Help Pages

- [x] 5.1 Update --help to show examples first
- [x] 5.2 Add "Common workflows" section to main help
- [x] 5.3 Add per-command examples

## 6. Testing

- [x] 6.1 Test output at each verbosity level
- [x] 6.2 Verify help pages include examples
