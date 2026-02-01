## 1. Output Framework

- [ ] 1.1 Create `src/output.rs` module with verbosity-aware printing
- [ ] 1.2 Define OutputLevel enum (Beginner, Intermediate, Expert)
- [ ] 1.3 Map -v flags to output levels

## 2. Beginner Mode (Default)

- [ ] 2.1 Implement simple success messages
- [ ] 2.2 Add "Next steps" suggestions (3-4 items)
- [ ] 2.3 Add links to examples/docs

## 3. Intermediate Mode (-v)

- [ ] 3.1 Add execution step logging
- [ ] 3.2 Show plugin hooks that ran
- [ ] 3.3 List files created/modified

## 4. Expert Mode (-vv)

- [ ] 4.1 Add full execution trace
- [ ] 4.2 Show state machine transitions
- [ ] 4.3 Add timing/performance metrics

## 5. Help Pages

- [ ] 5.1 Update --help to show examples first
- [ ] 5.2 Add "Common workflows" section to main help
- [ ] 5.3 Add per-command examples

## 6. Testing

- [ ] 6.1 Test output at each verbosity level
- [ ] 6.2 Verify help pages include examples
