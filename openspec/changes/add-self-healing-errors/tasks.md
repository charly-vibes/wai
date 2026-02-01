## 1. Foundation

- [ ] 1.1 Add strsim or similar crate for typo detection
- [ ] 1.2 Create `src/suggestions.rs` module for suggestion logic

## 2. Typo Detection

- [ ] 2.1 Implement command similarity matching
- [ ] 2.2 Add "Did you mean?" suggestions to unknown commands
- [ ] 2.3 Add subcommand typo detection

## 3. Context Inference

- [ ] 3.1 Detect when user is in a subdirectory of a project
- [ ] 3.2 Suggest running from project root or using `--project` flag

## 4. Wrong Order Detection

- [ ] 4.1 Detect reversed verb-noun patterns (e.g., `bead new` → `new bead`)
- [ ] 4.2 Show corrected command in error message

## 5. Conversational Tone

- [ ] 5.1 Update error message templates to use friendly language
- [ ] 5.2 Replace "Error:" with contextual phrases

## 6. Testing

- [ ] 6.1 Add tests for typo suggestions
- [ ] 6.2 Add tests for context inference
- [ ] 6.3 Add tests for wrong-order detection
