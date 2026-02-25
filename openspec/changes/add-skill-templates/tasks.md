## Prerequisites

The `create` template uses `wai search --latest`. This flag is introduced by
`add-artifact-tags`. Implement `add-artifact-tags` first, or stub the `--latest`
flag with a placeholder and complete the template body after that change lands.

## 1. Add --template flag to skill creation

- [ ] 1.1 Add `template: Option<String>` field to `ResourceAddCommands::Skill` in
        `src/cli.rs`
- [ ] 1.2 Add template lookup in `src/commands/resource.rs`: match on the template
        name and return the appropriate stub string

## 2. Implement built-in templates

- [ ] 2.1 Implement `gather` template: stub with `wai search "$ARGUMENTS"`, codebase
        exploration section, and `wai add research` instructions
- [ ] 2.2 Implement `create` template: stub with artifact retrieval via
        `wai search "$ARGUMENTS" --type plan --latest`, item loop, and
        `bd create` for output tracking
- [ ] 2.3 Implement `tdd` template: RED/GREEN/REFACTOR loop stub with `just check`
        (or `cargo test`) and commit instructions
- [ ] 2.4 Implement `rule-of-5` template: 5-pass review stub with convergence check
        and APPROVED / NEEDS_CHANGES / NEEDS_HUMAN verdict format
- [ ] 2.5 All templates MUST use `$ARGUMENTS`, `$PROJECT`, and `$REPO_ROOT` placeholders;
        no hardcoded project names

## 3. Tests

- [ ] 3.1 Test that `wai resource add skill my-skill --template gather` creates a
        SKILL.md containing the gather template body
- [ ] 3.2 Test that an unknown template name produces a clear error listing valid names
- [ ] 3.3 Test that omitting `--template` still creates the bare stub (no regression)

## 4. Documentation

- [ ] 4.1 Update `wai resource add skill --help` to document `--template` and list
        valid template names with one-line descriptions
