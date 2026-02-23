## 1. Workflow Detection

- [ ] 1.1 Create `src/workflows.rs` module
- [ ] 1.2 Define workflow patterns (new project, research phase, implement phase)
- [ ] 1.3 Implement state detection from `.wai/` contents

## 2. Post-Command Suggestions

- [ ] 2.1 Add suggestions after `wai new project`
- [ ] 2.2 Add suggestions after `wai add research`
- [ ] 2.3 Add suggestions after `wai phase next`

## 3. Phase-Aware Suggestions

- [ ] 3.1 Detect when project is ready for implementation
- [ ] 3.2 Suggest starting implementation
- [ ] 3.3 Detect when research might be needed

## 4. Interactive Ambiguity Resolution

- [ ] 4.1 Detect ambiguous commands (e.g., multiple matching projects)
- [ ] 4.2 Prompt for selection instead of erroring
- [ ] 4.3 Add --no-interactive flag to disable prompts

## 5. Testing

- [ ] 5.1 Test workflow detection accuracy
- [ ] 5.2 Test suggestion relevance
- [ ] 5.3 Test interactive prompts
